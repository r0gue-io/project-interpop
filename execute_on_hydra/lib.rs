#![cfg_attr(not(feature = "std"), no_std, no_main)]

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
