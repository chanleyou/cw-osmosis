use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const WHITELIST: Map<&Addr, bool> = Map::new("whitelist");

#[cw_serde]
pub struct Params {
    pub pool_id: u64,
    pub lock_period: u64, // TODO all this stuff
    pub fee: u64,
    // accepted_denoms
}

pub const PARAMS: Item<Params> = Item::new("params");
