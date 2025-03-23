#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod xcm;
use ink::scale::{Encode, Compact};
use ink::env::hash::{Blake2x256, CryptoHash};
use ink::xcm::{
    prelude::{Asset, Junction::Parachain, Location, OriginKind, Weight, Xcm, XcmHash},
    VersionedXcm,
};
use pop_api::{
    messaging::{self as api, ismp, ismp::Get, MessageId},
    StatusCode,
};

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod execute_on_hydra {
    use super::*;
    use ink::{prelude::vec::Vec, xcm::prelude::*};
    use pop_api::messaging::{ismp::StorageValue, Callback};

    const UNAUTHORIZED: u32 = u32::MAX;

    #[ink::event]
    pub struct GetCompleted {
        #[ink(topic)]
        pub id: MessageId,
        pub values: Vec<StorageValue>,
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct ExecuteOnHydra {
        query_id: u32,
    }

    impl ExecuteOnHydra {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { query_id: 0 }
        }

        #[ink(message, payable)]
        pub fn fund(&mut self) -> Result<()> {
            let dest = Location::new(1, Parachain(1000));

            // Reserve transfer specified assets to contract account on destination.
            let asset: Asset = (Location::parent(), self.env().transferred_value()).into();
            let beneficiary = hashed_account(4_001, self.env().account_id());
            let message: Xcm<()> = Xcm::builder_unsafe()
                .withdraw_asset(asset.clone().into())
                .initiate_reserve_withdraw(
                    asset.clone().into(),
                    dest.clone(),
                    Xcm::builder_unsafe()
                        .buy_execution(asset.clone(), WeightLimit::Unlimited)
                        .deposit_asset(
                            All.into(),
                            Location::new(0, AccountId32 { network: None, id: beneficiary.0 }),
                        )
                        .build(),
                )
                .build();
            api::xcm::execute(&VersionedXcm::V4(message)).unwrap();

            Ok(())
        }

        #[ink(message)]
        pub fn execute_on_hydra(
            &mut self,
            encoded_extrinsic: Vec<u8>,
            fee_max: Balance,
            ref_time: u64,
            proof_size: u64,
        ) -> Result<XcmHash> {
            let asset: Asset = (Location::parent(), fee_max).into();
            let hydra = Junctions::from([Parachain(2034)]);
            let dest: Location = Location {
                parents: 1,
                interior: hydra,
            };

            let message: Xcm<()> = Xcm::builder()
                .withdraw_asset(asset.clone().into())
                .buy_execution(asset.clone(), Unlimited)
                .transact(
                    OriginKind::SovereignAccount,
                    Weight::from_parts(ref_time, proof_size),
                    encoded_extrinsic.into(),
                )
                .build();

            Ok(api::xcm::send(&VersionedLocation::V4(dest), &VersionedXcm::V4(message)).unwrap())
        }

        #[ink(message)]
        pub fn query_storage_on_hydra(&mut self, key: Vec<u8>, height: u32) -> Result<()> {
            ismp::get(
                self.query_id as MessageId,
                Get::new(2034, height, 0, Vec::default(), Vec::from([key.clone()])),
                // 1 HDX
                1000000000000,
                Some(Callback::to(
                    0x57ad942b,
                    Weight::from_parts(800_000_000, 500_000),
                )),
            )?;
            self.query_id = self.query_id.saturating_add(1);

            Ok(())
        }

        #[ink(message)]
        pub fn create_pop_to_hydra_xcm(
            &mut self,
            amount: u128,
            ref_time: u64,
            proof_size: u64,
        ) {
            let hydra = Junctions::from([Parachain(2034)]);
            let dest: Location = Location {
                parents: 1,
                interior: hydra,
            };
            let asset: Asset = (Location::parent(), amount).into();
            let beneficiary = hashed_account(4_001, self.env().account_id());
            let fee_asset: Asset = (Location::parent(), 10000000000u128).into();

            // Define the chain locations
            let asset_hub = Location::new(
                1,
                Junctions::from([Parachain(1000)]),
            );
            let hydra = Location::new(
                1,
               Junctions::from([Parachain(2034)]),
            );
            let beneficiary = Location::new(
                0,
                Junctions::from([AccountId32 {
                    id: beneficiary.0,
                    network: None,
                }]),
            );

            // Create the asset to transfer
            let assets: Assets = Asset {
                id: asset.id.clone(),
                fun: Fungible(amount),
            }
                .into();

            // Create the weight limit
            let weight_limit = Limited(Weight::from_parts(ref_time, proof_size));

            // Executed on Asset Hub
            let xcm_to_hydra = Xcm([
                BuyExecution {
                    fees: fee_asset.clone(),
                    weight_limit: weight_limit.clone(),
                },
                DepositReserveAsset {
                    assets: Wild(AllCounted(1)),
                    dest: hydra,
                    xcm: Xcm([
                        BuyExecution {
                            fees: fee_asset.clone(),
                            weight_limit: weight_limit.clone(),
                        },
                        DepositAsset {
                            assets: Wild(AllCounted(1)),
                            beneficiary,
                        },
                    ].to_vec()),
                },
            ].to_vec());

            // Executed on Pop (local chain)
            let message: Xcm<()> = Xcm([
                WithdrawAsset(asset.into()),
                InitiateReserveWithdraw {
                    assets: All.into(),
                    reserve: asset_hub,
                    xcm: xcm_to_hydra,
                },
            ].to_vec());

            api::xcm::execute(&VersionedXcm::V4(message)).unwrap();
        }
    }

    fn hashed_account(para_id: u32, account_id: AccountId) -> AccountId {
        let location = (
            b"SiblingChain",
            Compact::<u32>::from(para_id),
            (b"AccountId32", account_id.0).encode(),
        )
            .encode();
        let mut output = [0u8; 32];
        Blake2x256::hash(&location, &mut output);
        AccountId::from(output)
    }

    impl api::ismp::OnGetResponse for ExecuteOnHydra {
        #[ink(message)]
        fn on_response(&mut self, id: MessageId, values: Vec<StorageValue>) -> pop_api::Result<()> {
            if self.env().caller() != self.env().account_id() {
                return Err(UNAUTHORIZED.into());
            }
            self.env().emit_event(GetCompleted { id, values });
            Ok(())
        }
    }
}
