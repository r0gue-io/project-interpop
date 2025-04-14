#![cfg_attr(not(feature = "std"), no_std, no_main)]

use api::xcm::{
    Junctions::{self, Here},
    Location,
};
use ink::{
    env::debug_println,
    prelude::vec::Vec,
    xcm::{
        prelude::{
            Asset, AssetId,
            Instruction::DepositAsset,
            Junction::{GeneralIndex, PalletInstance, Parachain},
            Reanchorable,
            WildAsset::All,
        },
        v4::{Instruction::WithdrawAsset, Xcm},
        VersionedXcm,
    },
};
use pop_api::{
    messaging::{self as api, MessageId},
    StatusCode,
};
use xcm::{
    fee_amount, local_account, native_asset, DepositedLocation, XcmMessageBuilder, ASSET_HUB,
    HYDRATION, POP,
};

mod xcm;

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod hydration_swapping {

    use xcm::{get_global_context, para};

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
        pub fn swap_usdt_on_hydration(
            &mut self,
            amount_out: u128,
            max_amount_in: u128,
            fee_amount: u128,
            dest: DepositedLocation,
        ) -> Result<()> {
            let fee = Asset {
                id: AssetId(Location {
                    parents: 1,
                    interior: Here,
                }),
                fun: fee_amount.into(),
            };
            // Relaychain native token - PASEO
            let give = Asset {
                id: AssetId(Location {
                    parents: 1,
                    interior: Here,
                }),
                fun: max_amount_in.into(),
            };
            // USDT
            let want = Asset {
                id: AssetId(Location {
                    parents: 1,
                    interior: Junctions::from((
                        Parachain(1000),
                        PalletInstance(50),
                        GeneralIndex(1984),
                    ))
                    .into(),
                }),
                fun: amount_out.into(),
            };
            self.transfer_and_swap_on_hydra(POP, ASSET_HUB, give, want, false, fee, dest)
        }

        #[ink(message, payable)]
        pub fn transfer_and_swap_on_hydra(
            &mut self,
            from_para: u32,
            intermediary_hop: u32,
            give_asset: Asset,
            want_asset: Asset,
            is_sell: bool,
            fee: Asset,
            dest: DepositedLocation,
        ) -> Result<()> {
            let amount_out = self.env().transferred_value();

            // Swap tokens on `HYDRATION` and then reserve transfer to `intermediary_hop`.
            let swap_on_hydration = XcmMessageBuilder::default()
                .set_max_weight_limit()
                .exchange_asset(give_asset, want_asset.clone(), is_sell, fee);

            let deposit_xcm = match dest {
                DepositedLocation::ParachainAccount(para_id, beneficiary) => {
                    let origin_context = get_global_context(HYDRATION);
                    // Deposit the destination account on the local `to_para`.
                    let destination_fee = want_asset
                        .clone()
                        .reanchored(&para(para_id), &origin_context)
                        .expect("should reanchor");
                    XcmMessageBuilder::default()
                        .set_next_hop(HYDRATION)
                        .send_to(para_id)
                        .set_max_weight_limit()
                        .deposit_to_account(beneficiary, false)
                        .reserve_transfer(
                            All.into(),
                            fee_amount(&destination_fee, 2).into(),
                            Xcm::default(),
                        )
                }
                DepositedLocation::Account(beneficiary) => Xcm([DepositAsset {
                    assets: All.into(),
                    beneficiary: local_account(beneficiary),
                }]
                .to_vec()),
                _ => panic!("Unsupported deposited location"),
            };

            // Transfer from `from_para` to `intermediary_hop` and deposit to `HYDRATION`.
            let message = XcmMessageBuilder::default()
                .set_next_hop(from_para)
                .set_max_weight_limit()
                .send_to(intermediary_hop)
                .deposit_to_parachain(HYDRATION)
                .reserve_transfer(
                    native_asset(amount_out).into(),
                    fee_amount(&native_asset(amount_out), 2),
                    Xcm([swap_on_hydration.0, deposit_xcm.0].concat()),
                );

            api::xcm::execute(&VersionedXcm::V4(Xcm([
                [WithdrawAsset(native_asset(amount_out).into())].to_vec(),
                message.0,
            ]
            .concat()
            .to_vec())))
            .unwrap();
            Ok(())
        }

        #[ink(message, payable)]
        pub fn transfer_and_execute_on_hydra(
            &mut self,
            from_para: u32,
            intermediary_hop: u32,
            xcm: Xcm<()>,
        ) -> Result<()> {
            let amount_out = self.env().transferred_value();

            // Transfer from `from_para` to `intermediary_hop` and deposit to `HYDRATION`.
            let message = XcmMessageBuilder::default()
                .set_next_hop(from_para)
                .send_to(intermediary_hop)
                .set_max_weight_limit()
                .deposit_to_parachain(HYDRATION)
                .reserve_transfer(
                    native_asset(amount_out).into(),
                    fee_amount(&native_asset(amount_out), 2),
                    xcm,
                );

            api::xcm::execute(&VersionedXcm::V4(Xcm([
                [WithdrawAsset(native_asset(amount_out).into())].to_vec(),
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
                .reserve_transfer(
                    native_asset(amount).into(),
                    fee_amount(&native_asset(amount), 2),
                    Xcm::default(),
                );
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
                .reserve_transfer(
                    native_asset(amount).into(),
                    fee_amount(&native_asset(amount), 2),
                    fund_intermediary_xcm,
                );
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
