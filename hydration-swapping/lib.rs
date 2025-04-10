#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::xcm::v4::Instruction::WithdrawAsset;
use ink::{
    env::debug_println,
    prelude::vec::Vec,
    xcm::{v4::Xcm, VersionedXcm},
};
use pop_api::{
    messaging::{self as api, MessageId},
    StatusCode,
};
use xcm::{fee_amount, native_asset, HYDRATION};
use xcm::{XcmMessageBuilder, ASSET_HUB, POP};

mod xcm;

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod hydration_swapping {
    use core::u128;

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
        #[ink(message, payable)]
        pub fn swap_on_hydration(
            &mut self,
            asset_in: u128,
            asset_out: u128,
            to: u32,
            beneficiary: AccountId,
            hashed: bool,
        ) -> Result<()> {
            self.multi_hop_swap(
                POP,
                ASSET_HUB,
                to,
                asset_in,
                asset_out,
                u128::MAX,
                beneficiary,
                hashed,
            )
        }

        #[ink(message, payable)]
        pub fn multi_hop_swap(
            &mut self,
            from_para: u32,
            intermediary_hop: u32,
            to_para: u32,
            asset_in: u128,
            asset_out: u128,
            max_amount_out: u128,
            beneficiary: AccountId,
            hashed: bool,
        ) -> Result<()> {
            let amount_in = self.env().transferred_value();

            // Deposit the destination account on the local `to_para`.
            let local_dest_fee = fee_amount(&native_asset(amount_in), 2);
            let deposit_dest_account = XcmMessageBuilder::default()
                .set_next_hop(to_para)
                .set_max_weight_limit()
                .deposit_to_account(beneficiary, hashed)
                .deposit_asset(local_dest_fee);

            // Reserve transfer to `intermediary_hop` and deposit to the `to_para`.
            let reserve_transfer_to_intermediary_hop = XcmMessageBuilder::default()
                .set_next_hop(HYDRATION)
                .send_to(intermediary_hop)
                .set_max_weight_limit()
                .deposit_to_parachain(to_para)
                .reserve_transfer(amount_in, deposit_dest_account);

            // Swap tokens on `HYDRATION` and then reserve transfer to `intermediary_hop`.
            let swap_on_hydration =
                XcmMessageBuilder::default().buy(asset_in, asset_out, amount_in, max_amount_out);

            // Transfer from `from_para` to `intermediary_hop` and deposit to `HYDRATION`.
            let message = XcmMessageBuilder::default()
                .set_next_hop(from_para)
                .send_to(intermediary_hop)
                .set_max_weight_limit()
                .deposit_to_parachain(HYDRATION)
                .reserve_transfer(
                    amount_in,
                    Xcm([swap_on_hydration.0, reserve_transfer_to_intermediary_hop.0].concat()),
                );

            api::xcm::execute(&VersionedXcm::V4(Xcm([
                [WithdrawAsset(native_asset(amount_in).into())].to_vec(),
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
