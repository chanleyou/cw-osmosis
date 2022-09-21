use std::error::Error;

#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, SubMsg,
};
use cosmwasm_std::{CosmosMsg, Reply};
use cw2::set_contract_version;
use osmosis_std::types::cosmos::base::v1beta1::Coin as CoinProtobuf;
use osmosis_std::types::osmosis::gamm::v1beta1::{
    GammQuerier, MsgJoinSwapExternAmountIn, MsgJoinSwapExternAmountInResponse, QueryPoolRequest,
    QueryPoolResponse,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Params, PARAMS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const COMPOUND_REPLY_ERROR_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let params = Params {
        pool_id: msg.pool_id,
        fee: 0,
        lock_period: 0,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    PARAMS.save(deps.storage, &params)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("pool_id", msg.pool_id.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Join {} => try_join(deps, info),
        ExecuteMsg::Compound { min_shares } => try_compound(deps, env.contract.address, min_shares),
    }
}

pub fn try_join(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let funds = info.funds;

    // mint LP based on funds deposited

    Ok(Response::new().add_attribute("method", "try_join"))
}

// TODO: make this permissioned
pub fn try_compound(
    deps: DepsMut,
    address: Addr,
    min_shares: u64,
) -> Result<Response, ContractError> {
    let params = PARAMS.load(deps.storage)?;

    // 1. get current reward token balances
    let balance = deps.querier.query_balance(&address, "uosmo")?;

    if balance.amount.is_zero() {
        return Result::Err(ContractError::ZeroBalance {});
    }

    // 2. swap rewards for LP
    let msg = CosmosMsg::from(MsgJoinSwapExternAmountIn {
        sender: address.to_string(),
        pool_id: params.pool_id,
        share_out_min_amount: min_shares.to_string(), // TODO: format this to string using pool denom
        token_in: Some(CoinProtobuf::from(balance)),
    });

    // 3. lock LP for rewards
    // TODO

    Ok(Response::new()
        .add_submessage(SubMsg::reply_on_error(msg, COMPOUND_REPLY_ERROR_ID))
        .add_attribute("method", "try_compound"))
}

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
//     match msg.id {
//         COMPOUND_REPLY_ERROR_ID => Result::Err(ContractError::CompoundFailed {}),
//         id => Result::Err(ContractError::UnknownReplyId { id }),
//     }
// }

// fn handle_compound_success_reply(
//     deps: DepsMut,
//     reply: MsgJoinSwapExternAmountInResponse,
// ) -> StdResult<Response> {
// }

// queries will not work until osmosis v12, see: https://lib.rs/crates/osmosis-std
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Query(QueryPoolRequest { pool_id }) => to_binary(&query_pool(deps, pool_id)?),
    }
}

fn query_pool(deps: Deps, pool_id: u64) -> StdResult<QueryPoolResponse> {
    let res = GammQuerier::new(&deps.querier).pool(pool_id)?;
    Ok(res)
}
