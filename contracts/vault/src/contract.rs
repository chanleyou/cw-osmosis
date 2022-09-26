#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdResult, SubMsg, Uint128,
};
use cw2::set_contract_version;
use osmosis_std::types::cosmos::base::v1beta1::Coin as CoinProtobuf;
use osmosis_std::types::osmosis::gamm::v1beta1::{
    GammQuerier, MsgJoinSwapExternAmountIn, MsgJoinSwapExternAmountInResponse,
};
use osmosis_std::types::osmosis::superfluid::{
    MsgLockAndSuperfluidDelegate, MsgLockAndSuperfluidDelegateResponse, MsgSuperfluidUnbondLock,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Parameters, PARAMETERS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MINT_SHARES_REPLY_ID: u64 = 0;
const COMPOUND_REPLY_ID: u64 = 1;
const LOCK_DELEGATE_ID: u64 = 2;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let params = Parameters {
        pool_id: msg.pool_id,
        fee: 0,
        lock_duration: msg.lock_duration,
        denom: format!("gamm/pool/{}", msg.pool_id),
    };

    // this will not work until Osmosis v12
    // let res = GammQuerier::new(&deps.querier).pool(msg.pool_id)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    PARAMETERS.save(deps.storage, &params)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("pool_id", msg.pool_id.to_string())
        .add_attribute("lock_duration", msg.lock_duration.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit {} => try_deposit(deps, info, env.contract.address),
        ExecuteMsg::Compound { min_shares } => try_compound(deps, env.contract.address, min_shares),
    }
}

pub fn try_deposit(
    deps: DepsMut,
    info: MessageInfo,
    address: Addr,
) -> Result<Response, ContractError> {
    let funds = info.funds;
    let params = PARAMETERS.load(deps.storage)?;

    let submessages: Vec<SubMsg> = funds
        .iter()
        .filter_map(|c| match c.denom.as_str() {
            denom if denom == format!("gamm/pool/{}", params.pool_id) => {
                // mint shares for user
                None
            }
            _ => {
                // swap and deposit, this will throw an error if denoms do not belong in the pool
                let msg = CosmosMsg::from(MsgJoinSwapExternAmountIn {
                    sender: address.to_string(),
                    pool_id: params.pool_id,
                    share_out_min_amount: "1".to_string(), // TODO
                    token_in: Some(CoinProtobuf::from(c.clone())),
                });

                Some(SubMsg::reply_on_success(msg, MINT_SHARES_REPLY_ID))
            }
        })
        .collect();

    // TODO: throw an error if user didn't send any funds that would result in LP being generated
    // TODO: mint LP based on funds deposited

    Ok(Response::new()
        .add_attribute("method", "try_join")
        .add_submessages(submessages))
}

// TODO: make this permissioned
pub fn try_compound(
    deps: DepsMut,
    address: Addr,
    min_shares: u64,
) -> Result<Response, ContractError> {
    let params = PARAMETERS.load(deps.storage)?;

    // 1. get current reward token balances
    // TODO: expand for multiple token rewards (Array<denoms>?)
    let balance = deps.querier.query_balance(&address, "uosmo")?;

    if balance.amount.is_zero() {
        return Result::Err(ContractError::ZeroBalance {});
    }

    // TODO: send reward to treasury

    // 2. compound rewards into pool
    let msg = CosmosMsg::from(MsgJoinSwapExternAmountIn {
        sender: address.to_string(),
        pool_id: params.pool_id,
        share_out_min_amount: min_shares.to_string(),
        token_in: Some(CoinProtobuf::from(balance)),
    });

    Ok(Response::new()
        .add_submessage(SubMsg::reply_on_success(msg, COMPOUND_REPLY_ID))
        .add_attribute("method", "try_compound"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        MINT_SHARES_REPLY_ID => handle_mint_shares(deps, msg),
        COMPOUND_REPLY_ID => handle_compound_reply(deps, msg),
        LOCK_DELEGATE_ID => handle_lock_delegate_reply(deps, msg),
        id => Result::Err(ContractError::UnknownReplyId { id }),
    }
}

// TODO: refactor shared code between combine mint shares and compound
fn handle_mint_shares(_deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    // mint shares for user
    // lock LP for rewards

    // TODO: handle errors instead of unwrap
    let data = msg.result.unwrap().data.unwrap();
    let res = MsgJoinSwapExternAmountInResponse::try_from(data)?;

    Ok(Response::new().add_attribute("minted", res.share_out_amount))
}

fn handle_compound_reply(deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    // 3. lock LP for rewards

    // TODO: handle errors instead of unwrap
    let data = msg.result.unwrap().data.unwrap();
    let res = MsgJoinSwapExternAmountInResponse::try_from(data)?;

    let params = PARAMETERS.load(deps.storage)?;

    let msg = CosmosMsg::from(MsgLockAndSuperfluidDelegate {
        sender: "todo".to_string(),
        coins: vec![CoinProtobuf::from(Coin {
            amount: Uint128::from(res.share_out_amount.parse::<u64>().unwrap()),
            denom: params.denom,
        })],
        val_addr: "todo".to_string(),
    });

    Ok(Response::new().add_submessage(SubMsg::reply_on_success(msg, LOCK_DELEGATE_ID)))
}

fn handle_lock_delegate_reply(_deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    let data = msg.result.unwrap().data.unwrap();
    let res = MsgLockAndSuperfluidDelegateResponse::try_from(data)?;

    Ok(Response::new().add_attribute("lock_id", res.id.to_string()))
}

// https://github.com/osmosis-labs/osmosis/blob/main/wasmbinding/stargate_whitelist.go
// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
//     match msg {
//         QueryMsg::QueryPoolRequest { pool_id } => to_binary(&query_pool(deps, pool_id)?),
//         QueryMsg::QueryNumPoolsRequest {} => to_binary(&query_num_pools(deps)?),
//     }
// }

// fn query_pool(deps: Deps, pool_id: u64) -> StdResult<QueryPoolResponse> {
//     let res = GammQuerier::new(&deps.querier).pool(pool_id)?;
//     Ok(QueryPoolResponse { pool: res.pool })
// }

// fn query_num_pools(deps: Deps) -> StdResult<QueryNumPoolsResponse> {
//     let res = GammQuerier::new(&deps.querier).num_pools()?;
//     Ok(QueryNumPoolsResponse {
//         num_pools: res.num_pools,
//     })
// }
