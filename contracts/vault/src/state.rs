use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

/// Vault Parameters
#[cw_serde]
pub struct Parameters {
    pub pool_id: u64,
    pub lock_duration: u64, // TODO: "24h" | "168h" | "336h"
    pub fee: u64,
    pub denom: String, // accepted_denoms
}

// struct Lock {
//     id: u64,
//     amount: u64,
// }

// pub struct Locks {
//     pub locks: Vec<{}>,
// }

pub const WHITELIST: Map<&Addr, bool> = Map::new("whitelist");
pub const PARAMETERS: Item<Parameters> = Item::new("parameters");
