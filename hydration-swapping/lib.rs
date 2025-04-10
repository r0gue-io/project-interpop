#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
    env::debug_println,
    prelude::vec::Vec,
    xcm::{v4::Xcm, VersionedXcm},
};
use pop_api::{
    messaging::{self as api, xcm::QueryId, MessageId},
    StatusCode,
};
use xcm::{XcmMessageBuilder, ASSET_HUB, POP};

mod xcm;

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod hydration_swapping {
    use ink::xcm::v4::Instruction::WithdrawAsset;
    use xcm::{fee_amount, native_asset};

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
        pub fn swap(&mut self, account: AccountId, hashed: bool) -> Result<Option<QueryId>> {
            unimplemented!()
        }

        #[ink(message, payable)]
        pub fn fund_direct(&mut self, account: AccountId, para_id: u32, hop: u32, hashed: bool) {
            let amount = self.env().transferred_value();
            let message = XcmMessageBuilder::default()
                .set_next_hop(hop)
                .send_to(para_id)
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
                from: hop,
                to: para_id,
            });
        }

        #[ink(message, payable)]
        pub fn fund_indirect(
            &mut self,
            account: AccountId,
            para_id: u32,
            starting_hop: u32,
            intermediary_hop: u32,
            hashed: bool,
        ) -> Result<()> {
            let amount = self.env().transferred_value();
            let local_hydration_fee = fee_amount(&native_asset(amount), 2);
            let fund_hydration_xcm = XcmMessageBuilder::default()
                .set_next_hop(intermediary_hop)
                .set_max_weight_limit()
                .deposit_to_account(account, hashed)
                .deposit_asset(local_hydration_fee);
            let message = XcmMessageBuilder::default()
                .set_next_hop(starting_hop)
                .send_to(intermediary_hop)
                .set_max_weight_limit()
                .deposit_to_parachain(para_id)
                .reserve_transfer(amount, fund_hydration_xcm);
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
                from: starting_hop,
                to: intermediary_hop,
            });
            self.env().emit_event(ReserveTransferred {
                account,
                amount,
                from: intermediary_hop,
                to: para_id,
            });
            Ok(())
        }

        #[ink(message, payable)]
        pub fn fund_asset_hub(&mut self, account: AccountId, hashed: bool) -> Result<()> {
            let amount = self.env().transferred_value();
            let message = XcmMessageBuilder::default()
                .set_next_hop(POP)
                .send_to(ASSET_HUB)
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
                from: POP,
                to: ASSET_HUB,
            });
            Ok(())
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
