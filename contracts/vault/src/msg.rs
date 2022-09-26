use cosmwasm_schema::{cw_serde, QueryResponses};
use osmosis_std::types::osmosis::gamm::v1beta1::{QueryNumPoolsResponse, QueryPoolResponse};

#[cw_serde]
pub struct InstantiateMsg {
    pub pool_id: u64,
    // pub fee: u64,
    pub lock_duration: u64, // TODO: check type for this
}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit {},
    Compound { min_shares: u64 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(QueryPoolResponse)]
    QueryPoolRequest { pool_id: u64 },
    #[returns(QueryNumPoolsResponse)]
    QueryNumPoolsRequest {},
}
