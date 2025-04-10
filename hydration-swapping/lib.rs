#![cfg_attr(not(feature = "std"), no_std, no_main)]

use api::xcm::{Junction, Junctions, NetworkId};
use ink::xcm::v4::{Asset, Instruction::WithdrawAsset, Reanchorable};
use ink::{
    env::debug_println,
    prelude::vec::Vec,
    xcm::{v4::Xcm, VersionedXcm},
};
use pop_api::{
    messaging::{self as api, MessageId},
    StatusCode,
};
use xcm::{fee_amount, native_asset, para, HYDRATION};
use xcm::{XcmMessageBuilder, ASSET_HUB, POP};

mod xcm;

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod hydration_swapping {

    use ink::xcm::v4::WeightLimit;

    use super::*;

    #[ink(storage)]
    #[derive(Default)]
    pub struct HydrationSwapping;

    impl HydrationSwapping {
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            Default::default()
        }

        /// Swap PASEO tokens on Hydration.
        /// - Message 1: Reserve transfer from Pop Network to Asset Hub.
        /// - Message 2: Reserve transfer from Asset Hub to Hydration.
        /// - Message 3: Swap tokens..
        #[ink(message, payable)]
        pub fn swap_on_hydration(
            &mut self,
            give_asset: Asset,
            want_asset: Asset,
            from_para: u32,
            intermediary_hop: u32,
            to_para: u32,
            is_sell: bool,
            account: AccountId,
            hashed: bool,
        ) -> Result<()> {
            let amount = self.env().transferred_value();

            let from_parachain = para(from_para);
            let to_parachain = para(to_para);
            let swap_parachain = para(HYDRATION);

            let origin_context = Junctions::from([
                Junction::GlobalConsensus(NetworkId::Polkadot),
                Junction::Parachain(from_para),
            ]);
            let give = give_asset
                .clone()
                .reanchored(&to_parachain, &origin_context)
                .expect("should reanchor give");
            let fees = give_asset
                .clone()
                .reanchored(&swap_parachain, &from_parachain.interior)
                .expect("should reanchor");

            let local_dest_fee = fee_amount(&native_asset(amount), 2);
            let deposit_dest_account = XcmMessageBuilder::default()
                .set_next_hop(to_para)
                .set_max_weight_limit()
                .deposit_to_account(account, hashed)
                .deposit_asset(local_dest_fee);
            // Reserve transfer to `intermediary_hop` and deposit to the `dest_parachain`.
            let reserve_transfer_to_intermediary_hop = XcmMessageBuilder::default()
                .set_next_hop(HYDRATION)
                .send_to(intermediary_hop)
                .set_max_weight_limit()
                .deposit_to_parachain(to_para)
                .reserve_transfer(amount, deposit_dest_account);

            // Swap tokens on `HYDRATION` and then reserve transfer to `intermediary_hop`.
            let swap_on_hydration = Xcm::builder_unsafe()
                .buy_execution(fees, WeightLimit::Unlimited)
                .exchange_asset(give.into(), want_asset.into(), is_sell)
                .build();

            // Transfer from `from_parachain` to `intermediary_hop` and deposit to `HYDRATION`.
            let message = XcmMessageBuilder::default()
                .set_next_hop(from_para)
                .send_to(intermediary_hop)
                .set_max_weight_limit()
                .deposit_to_parachain(HYDRATION)
                .reserve_transfer(
                    amount,
                    Xcm([swap_on_hydration.0, reserve_transfer_to_intermediary_hop.0].concat()),
                );

            api::xcm::execute(&VersionedXcm::V4(Xcm([
                [WithdrawAsset(native_asset(amount).into())].to_vec(),
                message.0,
            ]
            .concat()
            .to_vec())))
            .unwrap();
            Ok(())
        }

        #[ink(message, payable)]
        pub fn fund_direct(
            &mut self,
            account: AccountId,
            from_para: u32,
            to_para: u32,
            hashed: bool,
        ) -> Result<()> {
            let amount = self.env().transferred_value();
            let message = XcmMessageBuilder::default()
                .set_next_hop(from_para)
                .send_to(to_para)
                .set_max_weight_limit()
                .deposit_to_account(account, hashed)
                .reserve_transfer(amount, Xcm::default());
            api::xcm::execute(&VersionedXcm::V4(Xcm([
                [WithdrawAsset(native_asset(amount).into())].to_vec(),
                message.0,
            ]
            .concat()
            .to_vec())))
            .unwrap();
            self.env().emit_event(ReserveTransferred {
                account,
                amount,
                from: from_para,
                to: to_para,
            });
            Ok(())
        }

        #[ink(message, payable)]
        pub fn fund_indirect(
            &mut self,
            account: AccountId,
            from_para: u32,
            intermediary_hop: u32,
            to_para: u32,
            hashed: bool,
        ) -> Result<()> {
            let amount = self.env().transferred_value();
            let local_intermerdiary_fee = fee_amount(&native_asset(amount), 2);
            let fund_intermediary_xcm = XcmMessageBuilder::default()
                .set_next_hop(to_para)
                .set_max_weight_limit()
                .deposit_to_account(account, hashed)
                .deposit_asset(local_intermerdiary_fee);
            let message = XcmMessageBuilder::default()
                .set_next_hop(from_para)
                .send_to(intermediary_hop)
                .set_max_weight_limit()
                .deposit_to_parachain(to_para)
                .reserve_transfer(amount, fund_intermediary_xcm);
            api::xcm::execute(&VersionedXcm::V4(Xcm([
                [WithdrawAsset(native_asset(amount).into())].to_vec(),
                message.0,
            ]
            .concat()
            .to_vec())))
            .unwrap();
            self.env().emit_event(ReserveTransferred {
                account,
                amount,
                from: from_para,
                to: intermediary_hop,
            });
            self.env().emit_event(ReserveTransferred {
                account,
                amount,
                from: intermediary_hop,
                to: to_para,
            });
            Ok(())
        }

        #[ink(message, payable)]
        pub fn fund_hydration(&mut self, account: AccountId, hashed: bool) -> Result<()> {
            self.fund_indirect(account, POP, ASSET_HUB, HYDRATION, hashed)
        }

        #[ink(message, payable)]
        pub fn fund_asset_hub(&mut self, account: AccountId, hashed: bool) -> Result<()> {
            self.fund_direct(account, POP, ASSET_HUB, hashed)
        }

        #[ink(message)]
        pub fn get(&self, id: MessageId) -> Result<Option<Vec<u8>>> {
            debug_println!("messaging::get id={id}");
            api::get((self.env().account_id(), id))
        }

        #[ink(message)]
        pub fn remove(&mut self, id: MessageId) -> Result<()> {
            debug_println!("messaging::remove id={id}");
            api::remove([id].to_vec())?;
            Ok(())
        }
    }

    #[ink::event]
    pub struct ReserveTransferred {
        #[ink(topic)]
        pub account: AccountId,
        pub amount: u128,
        pub from: u32,
        #[ink(topic)]
        pub to: u32,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn default_works() {
            HydrationSwapping::new();
        }
    }
}
