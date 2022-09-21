use cosmwasm_schema::{cw_serde, QueryResponses};
use osmosis_std::types::osmosis::gamm::v1beta1::{QueryPoolRequest, QueryPoolResponse};

#[cw_serde]
pub struct InstantiateMsg {
    pub pool_id: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Join {},
    Compound { min_shares: u64 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(QueryPoolResponse)]
    Query(QueryPoolRequest),
}
