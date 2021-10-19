use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::CanonicalAddr;
use cw_controllers::Admin;
use cw_storage_plus::Item;

use white_whale::deposit_info::DepositInfo;
use white_whale::fee::VaultFee;

use crate::pool_info::PoolInfoRaw;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// The Stablecoin-vault State contains configuration options for the vault including
// the address of the pool to trade in as well as some other addresses
pub struct State {
    pub trader: CanonicalAddr,
    pub vault_address: CanonicalAddr,
    pub seignorage_address: CanonicalAddr,
    pub pool_address: CanonicalAddr,
}

pub const ADMIN: Admin = Admin::new("admin");
pub const STATE: Item<State> = Item::new("\u{0}{5}state");
pub const POOL_INFO: Item<PoolInfoRaw> = Item::new("\u{0}{4}pool");
pub const DEPOSIT_INFO: Item<DepositInfo> = Item::new("\u{0}{7}deposit");
pub const FEE: Item<VaultFee> = Item::new("\u{0}{12}fee");