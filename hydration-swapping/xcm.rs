use ink::{
    env::hash::{Blake2x256, CryptoHash},
    primitives::H160,
    scale::{Compact, Encode},
    xcm::prelude::*,
};
use pop_api::primitives::Balance;

pub(crate) const UNITS: Balance = 1_000_000_000_000;
pub(crate) const ASSET_HUB: u32 = 1000;
pub(crate) const HYDRATION: u32 = 2034;
pub(crate) const POP: u32 = 4001;

pub(crate) fn para(id: u32) -> Location {
    Location::new(1, Parachain(id))
}

pub(crate) fn native_asset(amount: u128) -> Assets {
    (Location::parent(), amount).into()
}

fn local_account(account: H160) -> Location {
    Location::new(
        0,
        AccountKey20 {
            network: None,
            key: account.0,
        },
    )
}

pub(crate) struct XcmMessageBuilder {
    dest_chain: Option<u32>,
    current_hop: Option<u32>,
    weight_limit: WeightLimit,
    account: Option<H160>,
}

impl Default for XcmMessageBuilder {
    fn default() -> Self {
        Self {
            dest_chain: None,
            current_hop: None,
            weight_limit: Limited(Weight::MAX),
            account: None,
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
        self.weight_limit = Limited(Weight::MAX);
        self
    }

    pub fn set_account(&mut self, account: H160, hashed: bool) -> &mut Self {
        self.account = if hashed {
            Some(hashed_account(self.current_hop(), account))
        } else {
            Some(account)
        };
        self
    }

    pub fn deposit_asset(&mut self, asset: Asset, beneficiary: H160) -> Xcm<()> {
        Xcm::builder_unsafe()
            .buy_execution(asset, WeightLimit::Unlimited)
            .deposit_asset(
                AssetFilter::Wild(WildAsset::All),
                local_account(beneficiary),
            )
            .build()
    }

    pub fn on_reserve_asset_deposited(
        &mut self,
        asset: Asset,
        beneficiary: H160,
        xcm: Xcm<()>,
    ) -> Xcm<()> {
        if xcm.is_empty() {
            self.deposit_asset(asset, beneficiary)
        } else {
            Xcm::builder_unsafe()
                .buy_execution(asset.clone(), WeightLimit::Unlimited)
                .deposit_reserve_asset(
                    AssetFilter::Wild(WildAsset::All),
                    local_account(beneficiary),
                    xcm,
                )
                .build()
        }
    }

    pub fn reserve_transfer_no_withdraw(&mut self, amount: u128, xcm: Xcm<()>) -> Xcm<()> {
        let beneficiary = self.account.unwrap();
        // Balance of the contract caller.
        let asset: Asset = (Location::parent(), amount).into();
        // Construct a message to initiate a reserve withdraw.
        Xcm::builder_unsafe()
            .buy_execution(asset.clone(), WeightLimit::Unlimited)
            .initiate_reserve_withdraw(
                Assets::from(asset.clone()),
                self.dest(),
                self.on_reserve_asset_deposited(asset, beneficiary, xcm),
            )
            .build()
    }

    pub fn reserve_transfer(&mut self, amount: u128, xcm: Xcm<()>) -> Xcm<()> {
        let beneficiary = self.account.unwrap();
        // Balance of the contract caller.
        let asset: Asset = (Location::parent(), amount).into();
        // Construct a message to initiate a reserve withdraw.
        Xcm::builder_unsafe()
            .withdraw_asset(Assets::from(asset.clone()))
            .initiate_reserve_withdraw(
                Assets::from(asset.clone()),
                self.dest(),
                self.on_reserve_asset_deposited(asset, beneficiary, xcm),
            )
            .build()
    }

    pub fn swap(&mut self, give: Asset, want: Asset, is_sell: bool) -> Xcm<()> {
        let beneficiary = self.account.unwrap();
        let assets: Assets = native_asset(100 * UNITS);
        let dest = self.dest();
        let context = Junctions::from([
            Junction::GlobalConsensus(NetworkId::Polkadot),
            Junction::Parachain(self.current_hop()),
        ]);
        let fees = assets
            .get(0)
            .expect("should have at least 1 asset")
            .clone()
            .reanchored(&dest, &context)
            .expect("should reanchor");
        let give: AssetFilter = Definite(give.into());
        let want: Assets = want.into();

        Xcm::<()>::builder_unsafe()
            .set_fees_mode(true)
            .transfer_reserve_asset(
                assets,
                dest,
                Xcm::builder_unsafe()
                    .buy_execution(fees, self.weight_limit.clone())
                    .exchange_asset(give, want, is_sell)
                    .deposit_asset(Wild(AllCounted(1)), local_account(beneficiary))
                    .build(),
            )
            .build()
    }

    fn dest(&self) -> Location {
        self.dest_chain.map(para).unwrap_or(Location::parent())
    }

    fn current_hop(&self) -> u32 {
        self.current_hop.unwrap()
    }
}

pub(crate) fn hashed_account(para_id: u32, account_id: H160) -> H160 {
    let location = (
        b"SiblingChain",
        Compact::<u32>::from(para_id),
        (b"AccountId32", account_id.0).encode(),
    )
        .encode();
    let mut output = [0u8; 20];
    Blake2x256::hash(&location, &mut output);
    H160::from(output)
}
