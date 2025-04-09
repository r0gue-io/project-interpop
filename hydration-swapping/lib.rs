#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
    prelude::vec::Vec,
    xcm::{
        prelude::{Asset, Junction, Junctions, Location, Xcm},
        VersionedXcm,
    },
};
use pop_api::{
    messaging::{self as api, xcm::QueryId, MessageId},
    StatusCode,
};
use xcm::{XcmMessageBuilder, ASSET_HUB, HYDRATION, POP, UNITS};

mod xcm;

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod hydration_swapping {
    use ink::{H160, U256};

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
        pub fn swap(&mut self, account: H160, hashed: bool) -> Result<Option<QueryId>> {
            let amount = self.env().transferred_value();
            let give = Asset::from((
                Location::new(
                    1,
                    Junctions::from([Junction::Parachain(HYDRATION), Junction::GeneralIndex(0)]),
                ),
                50 * UNITS,
            ));
            let want = Asset::from((
                Location::new(
                    1,
                    Junctions::from([Junction::Parachain(HYDRATION), Junction::GeneralIndex(0)]),
                ),
                50 * UNITS,
            ));
            let swap_on_hydration = XcmMessageBuilder::default()
                .set_account(account, hashed)
                .swap(give, want, false);
            let fund_hydration_xcm = XcmMessageBuilder::default()
                .set_next_hop(ASSET_HUB)
                .send_to(HYDRATION)
                .set_max_weight_limit()
                .set_account(account, true)
                .reserve_transfer(amount, swap_on_hydration);
            let fund_asset_hub_xcm = XcmMessageBuilder::default()
                .set_next_hop(ASSET_HUB)
                .set_max_weight_limit()
                .set_account(account, true)
                .reserve_transfer(amount, fund_hydration_xcm);
            api::xcm::execute(&VersionedXcm::V4(fund_asset_hub_xcm)).unwrap();
            Ok(None)
        }

        #[ink(message, payable)]
        pub fn fund_parachain(
            &mut self,
            hop: u32,
            account: H160,
            para_id: u32,
            hashed: bool,
            xcm: Option<Xcm<()>>,
        ) {
            let amount = self.env().transferred_value();
            let message = XcmMessageBuilder::default()
                .set_next_hop(hop)
                .send_to(para_id)
                .set_max_weight_limit()
                .set_account(account, hashed)
                .reserve_transfer(amount, xcm.unwrap_or_default());
            api::xcm::execute(&VersionedXcm::V5(message)).unwrap();
            self.env().emit_event(ReserveTransferred {
                account,
                amount,
                from: hop,
                to: para_id,
            });
        }

        #[ink(message, payable)]
        pub fn fund_hydration(&mut self, account: H160, hashed: bool) -> Result<()> {
            let amount = self.env().transferred_value();
            let message = XcmMessageBuilder::default()
                .set_next_hop(POP)
                .send_to(ASSET_HUB)
                .set_max_weight_limit()
                .set_account(account, true)
                .reserve_transfer(
                    amount,
                    XcmMessageBuilder::default()
                        .set_next_hop(ASSET_HUB)
                        .send_to(HYDRATION)
                        .set_max_weight_limit()
                        .set_account(account, hashed)
                        .reserve_transfer_no_withdraw(amount, Xcm::default()),
                );
            api::xcm::execute(&VersionedXcm::V5(message)).unwrap();
            self.env().emit_event(ReserveTransferred {
                account,
                amount,
                from: POP,
                to: ASSET_HUB,
            });
            self.env().emit_event(ReserveTransferred {
                account,
                amount,
                from: ASSET_HUB,
                to: HYDRATION,
            });
            Ok(())
        }

        #[ink(message, payable)]
        pub fn fund_asset_hub(&mut self, account: H160, hashed: bool) -> Result<()> {
            let amount = self.env().transferred_value();
            let message = XcmMessageBuilder::default()
                .set_next_hop(POP)
                .send_to(ASSET_HUB)
                .set_max_weight_limit()
                .set_account(account, hashed)
                .reserve_transfer(amount, Xcm::default());
            api::xcm::execute(&VersionedXcm::V5(message)).unwrap();
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
            api::get((self.env().caller(), id))
        }

        #[ink(message)]
        pub fn remove(&mut self, id: MessageId) -> Result<()> {
            api::remove([id].to_vec())?;
            Ok(())
        }
    }

    #[ink::event]
    pub struct ReserveTransferred {
        #[ink(topic)]
        pub account: H160,
        pub amount: U256,
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
