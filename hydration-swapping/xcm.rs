use ink::{
    env::hash::{Blake2x256, CryptoHash},
    primitives::AccountId,
    scale::{Compact, Encode},
    xcm::prelude::*,
};
use pop_api::messaging::xcm::Location;

pub(crate) const ASSET_HUB: u32 = 1000;
pub(crate) const HYDRATION: u32 = 2034;
pub(crate) const POP: u32 = 4001;

pub enum DepositedLocation {
    Account(AccountId),
    Parachain(u32),
}

pub(crate) struct XcmMessageBuilder {
    dest_chain: Option<u32>,
    current_hop: Option<u32>,
    weight_limit: WeightLimit,
    deposited_location: Option<DepositedLocation>,
}

impl Default for XcmMessageBuilder {
    fn default() -> Self {
        Self {
            dest_chain: None,
            current_hop: None,
            weight_limit: Limited(Weight::MAX),
            deposited_location: None,
        }
    }
}

impl XcmMessageBuilder {
    pub fn set_next_hop(&mut self, current_hop: u32) -> &mut Self {
        self.current_hop = Some(current_hop);
        self
    }

    pub fn send_to(&mut self, dest_chain: u32) -> &mut Self {
        self.dest_chain = Some(dest_chain);
        self
    }

    pub fn set_max_weight_limit(&mut self) -> &mut Self {
        self.weight_limit = WeightLimit::Unlimited;
        self
    }

    pub fn set_weight_limit(&mut self, ref_time: u64, proof_size: u64) -> &mut Self {
        self.weight_limit = WeightLimit::Limited(Weight::from_parts(ref_time, proof_size));
        self
    }

    pub fn deposit_to_account(&mut self, account: AccountId, hashed: bool) -> &mut Self {
        self.deposited_location = Some(DepositedLocation::Account(if hashed {
            hashed_account(self.current_hop(), account)
        } else {
            account
        }));
        self
    }

    pub fn deposit_to_parachain(&mut self, para: u32) -> &mut Self {
        self.deposited_location = Some(DepositedLocation::Parachain(para));
        self
    }

    pub fn deposit_asset(&mut self, fee_asset: Asset) -> Xcm<()> {
        match self.deposited_location {
            Some(DepositedLocation::Account(account)) => Xcm::builder_unsafe()
                .buy_execution(fee_asset, self.weight_limit.clone())
                .deposit_asset(All.into(), local_account(account))
                .build(),
            _ => panic!("No deposited location set"),
        }
    }

    pub fn on_reserve_asset_deposited(&mut self, fee_asset: Asset, xcm: Xcm<()>) -> Xcm<()> {
        if xcm.is_empty() {
            self.deposit_asset(fee_asset)
        } else {
            let builder = Xcm::builder_unsafe().buy_execution(fee_asset, self.weight_limit.clone());
            match self.deposited_location {
                Some(DepositedLocation::Account(account)) => builder
                    .deposit_reserve_asset(All.into(), local_account(account), xcm)
                    .build(),
                Some(DepositedLocation::Parachain(id)) => builder
                    .deposit_reserve_asset(All.into(), para(id), xcm)
                    .build(),
                _ => panic!("No deposited location set"),
            }
        }
    }

    pub fn reserve_transfer(&mut self, amount: u128, xcm: Xcm<()>) -> Xcm<()> {
        let origin_context = Junctions::from([
            Junction::GlobalConsensus(NetworkId::Polkadot),
            Junction::Parachain(self.current_hop()),
        ]);
        // Balance of the contract caller.
        let asset = native_asset(amount);
        let reserve_fees = asset
            .clone()
            .reanchored(&self.source_chain(), &origin_context)
            .expect("should reanchor");

        Xcm::builder_unsafe()
            .initiate_reserve_withdraw(
                asset.clone().into(),
                self.dest_chain(),
                self.on_reserve_asset_deposited(reserve_fees, xcm),
            )
            .build()
    }

    pub fn exchange_asset(
        &mut self,
        give_asset: Asset,
        want_asset: Asset,
        is_sell: bool,
    ) -> Xcm<()> {
        let fee = fee_amount(&give_asset, 3);
        let give: AssetFilter = Definite(give_asset.into());
        let want: Assets = want_asset.into();
        Xcm::builder_unsafe()
            // Purchase execition using the native asset HDX.
            // .withdraw_asset(fee.clone().into())
            .buy_execution(fee.into(), self.weight_limit.clone())
            .exchange_asset(give, want, is_sell)
            .build()
    }

    fn dest_chain(&self) -> Location {
        self.dest_chain.map(para).unwrap_or(Location::parent())
    }

    fn source_chain(&self) -> Location {
        self.current_hop.map(para).unwrap_or(Location::parent())
    }

    fn current_hop(&self) -> u32 {
        self.current_hop.unwrap()
    }
}

pub(crate) fn hashed_account(para_id: u32, account_id: AccountId) -> AccountId {
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

/// Returns amount if `asset` is fungible, or zero.
pub(crate) fn fungible_amount(asset: &Asset) -> u128 {
    if let Fungible(amount) = &asset.fun {
        *amount
    } else {
        0
    }
}

pub(crate) fn fee_amount(asset: &Asset, div_by: u128) -> Asset {
    let amount = fungible_amount(asset)
        .checked_div(div_by)
        .expect("div 2 can't overflow; qed");
    Asset {
        fun: Fungible(amount),
        id: asset.clone().id,
    }
}

pub(crate) fn para(id: u32) -> Location {
    Location::new(1, Parachain(id))
}

pub(crate) fn native_asset(amount: u128) -> Asset {
    (Location::parent(), amount).into()
}

pub(crate) fn local_account(account: AccountId) -> Location {
    Location::new(
        0,
        AccountId32 {
            network: None,
            id: account.0,
        },
    )
}
