#![cfg_attr(not(feature = "std"), no_std, no_main)]

use api::xcm::{Junction, Junctions, Location};
use ink::{
    env::debug_println,
    prelude::vec::Vec,
    xcm::{
        v4::{Asset, Xcm},
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
        pub fn swap(&mut self) -> Result<Option<QueryId>> {
            let account = self.env().caller();
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
            let swap_on_hydration = XcmMessageBuilder::default().swap(account, give, want, false);
            let fund_hydration_xcm = XcmMessageBuilder::default()
                .set_next_hop(ASSET_HUB)
                .send_to(HYDRATION)
                .set_max_weight_limit()
                .reserve_transfer(account, amount, swap_on_hydration);
            let fund_asset_hub_xcm = XcmMessageBuilder::default()
                .set_next_hop(ASSET_HUB)
                .set_max_weight_limit()
                .reserve_transfer(account, amount, fund_hydration_xcm);
            api::xcm::execute(&VersionedXcm::V4(fund_asset_hub_xcm)).unwrap();
            Ok(None)
        }

        #[ink(message, payable)]
        pub fn fund_parachain(&mut self, hop: u32, para_id: u32, xcm: Option<Xcm<()>>) {
            let account = self.env().account_id();
            let amount = self.env().transferred_value();
            let message = XcmMessageBuilder::default()
                .set_next_hop(hop)
                .send_to(para_id)
                .set_max_weight_limit()
                .reserve_transfer(account, amount, xcm.unwrap_or_default());
            api::xcm::execute(&VersionedXcm::V4(message)).unwrap();
        }

        #[ink(message, payable)]
        pub fn fund_hydration(&mut self) -> Result<()> {
            let fund_hydration_xcm = XcmMessageBuilder::default()
                .set_next_hop(ASSET_HUB)
                .send_to(HYDRATION)
                .set_max_weight_limit()
                .reserve_transfer(
                    self.env().account_id(),
                    self.env().transferred_value(),
                    Xcm::default(),
                );
            let message = XcmMessageBuilder::default()
                .set_next_hop(POP)
                .send_to(ASSET_HUB)
                .set_max_weight_limit()
                .reserve_transfer(
                    self.env().account_id(),
                    self.env().transferred_value(),
                    fund_hydration_xcm,
                );
            api::xcm::execute(&VersionedXcm::V4(message)).unwrap();
            Ok(())
        }

        #[ink(message, payable)]
        pub fn fund_asset_hub(&mut self) -> Result<()> {
            let message = XcmMessageBuilder::default()
                .set_next_hop(POP)
                .send_to(ASSET_HUB)
                .set_max_weight_limit()
                .reserve_transfer(
                    self.env().account_id(),
                    self.env().transferred_value(),
                    Xcm::default(),
                );
            api::xcm::execute(&VersionedXcm::V4(message)).unwrap();
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

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn default_works() {
            HydrationSwapping::new();
        }
    }
}
