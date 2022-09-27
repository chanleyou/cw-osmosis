#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Querier, Reply, Response, StdResult, SubMsg, Uint128,
};
use cw2::set_contract_version;
use osmosis_std::shim::Duration;
use osmosis_std::types::cosmos::base::v1beta1::Coin as OsmosisCoin;
use osmosis_std::types::osmosis::gamm::v1beta1::{
    GammQuerier, MsgJoinSwapExternAmountIn, MsgJoinSwapExternAmountInResponse, Pool,
    QueryNumPoolsResponse, QueryPoolResponse,
};
use osmosis_std::types::osmosis::lockup::{MsgLockTokens, MsgLockTokensResponse};
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
const LOCK_SUPERFLUID_DELEGATE_ID: u64 = 2;
const LOCK_ID: u64 = 3;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let params = Parameters {
        pool_id: msg.pool_id,
        fee: 0,
        lock_duration: msg.lock_duration,
        denom: format!("gamm/pool/{}", msg.pool_id),
    };

    // let res = GammQuerier::new(&deps.querier).pool(msg.pool_id)?;
    // let pool = res.pool.unwrap();

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
    let params = PARAMETERS.load(deps.storage)?;

    let submessages: Vec<SubMsg> = info
        .funds
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
                    token_in: Some(OsmosisCoin::from(c.clone())),
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
        token_in: Some(OsmosisCoin::from(balance)),
    });

    Ok(Response::new()
        .add_submessage(SubMsg::reply_on_success(msg, COMPOUND_REPLY_ID))
        .add_attribute("method", "try_compound"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        MINT_SHARES_REPLY_ID => handle_mint_shares(deps, env, msg),
        COMPOUND_REPLY_ID => handle_compound_reply(deps, env, msg),
        LOCK_SUPERFLUID_DELEGATE_ID => handle_superfluid_lock_delegate_reply(deps, msg),
        LOCK_ID => handle_msg_lock_reply(deps, msg),
        id => Result::Err(ContractError::UnknownReplyId { id }),
    }
}

// TODO: refactor shared code between combine mint shares and compound
fn handle_mint_shares(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    // mint shares for user
    // lock LP for rewards
    let params = PARAMETERS.load(deps.storage)?;

    // TODO: handle errors instead of unwrap
    let data = msg.result.unwrap().data.unwrap();
    let res = MsgJoinSwapExternAmountInResponse::try_from(data)?;

    let msg = CosmosMsg::from(MsgLockAndSuperfluidDelegate {
        sender: env.contract.address.to_string(),
        coins: vec![OsmosisCoin::from(Coin {
            amount: Uint128::from(res.share_out_amount.parse::<u128>().unwrap()),
            denom: params.denom,
        })],
        val_addr: "osmovaloper1c584m4lq25h83yp6ag8hh4htjr92d954kphp96".to_string(), // TODO: unhardcode for mainnet
    });

    Ok(Response::new().add_submessage(SubMsg::reply_on_success(msg, LOCK_SUPERFLUID_DELEGATE_ID)))
}

fn handle_compound_reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    // 3. lock LP for rewards

    // TODO: handle errors instead of unwrap
    let data = msg.result.unwrap().data.unwrap();
    let res = MsgJoinSwapExternAmountInResponse::try_from(data)?;

    let params = PARAMETERS.load(deps.storage)?;

    let msg = CosmosMsg::from(MsgLockTokens {
        owner: env.contract.address.to_string(),
        duration: Some(Duration {
            seconds: 1209600, // 14 day superfluid lockup
            nanos: 0,
        }),
        coins: vec![OsmosisCoin::from(Coin {
            amount: Uint128::from(res.share_out_amount.parse::<u128>().unwrap()),
            denom: params.denom,
        })],
    });

    Ok(Response::new().add_submessage(SubMsg::reply_on_success(msg, LOCK_ID)))
}

fn handle_superfluid_lock_delegate_reply(
    _deps: DepsMut,
    msg: Reply,
) -> Result<Response, ContractError> {
    let data = msg.result.unwrap().data.unwrap();
    let res = MsgLockAndSuperfluidDelegateResponse::try_from(data)?;

    Ok(Response::new().add_attribute("lock_id", res.id.to_string()))
}

fn handle_msg_lock_reply(_deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    let data = msg.result.unwrap().data.unwrap();
    let res = MsgLockTokensResponse::try_from(data)?;

    Ok(Response::new().add_attribute("lock_id", res.id.to_string()))
}

// https://github.com/osmosis-labs/osmosis/blob/main/wasmbinding/stargate_whitelist.go
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryPoolRequest { pool_id } => to_binary(&query_pool(deps, pool_id)?),
        QueryMsg::QueryNumPoolsRequest {} => to_binary(&query_num_pools(deps)?),
    }
}

fn query_pool(deps: Deps, pool_id: u64) -> StdResult<QueryPoolResponse> {
    let res = GammQuerier::new(&deps.querier).pool(pool_id)?;
    Ok(QueryPoolResponse { pool: res.pool })
}

fn query_num_pools(deps: Deps) -> StdResult<QueryNumPoolsResponse> {
    let res = GammQuerier::new(&deps.querier).num_pools()?;
    Ok(res)
}

#[cfg(test)]
mod test {
    use osmosis_std::types::osmosis::gamm::v1beta1::QueryPoolResponse;
    use osmosis_testing::{Gamm, Module, OsmosisTestApp, Wasm};

    use super::*;

    #[test]
    fn assert_stuff() {
        let app = OsmosisTestApp::new();

        let account = app
            .init_account(&[
                Coin::new(1_000_000_000_000, "uatom"),
                Coin::new(1_000_000_000_000, "uosmo"),
            ])
            .unwrap();

        // create pool
        let gamm = Gamm::new(&app);
        let pool_liquidity = vec![Coin::new(1_000, "uatom"), Coin::new(1_000, "uosmo")];
        let pool_id = gamm
            .create_basic_pool(&pool_liquidity, &account)
            .unwrap()
            .data
            .pool_id;

        let wasm = Wasm::new(&app);

        let wasm_byte_code = std::fs::read("../../artifacts/vault.wasm").unwrap();
        let code_id = wasm
            .store_code(&wasm_byte_code, None, &account)
            .unwrap()
            .data
            .code_id;

        let contract_addr = wasm
            .instantiate(
                code_id,
                &InstantiateMsg {
                    pool_id,
                    lock_duration: 0,
                },
                None,
                None,
                &[],
                &account,
            )
            .unwrap()
            .data
            .address;

        let res = wasm
            .query::<QueryMsg, QueryNumPoolsResponse>(
                &contract_addr,
                &QueryMsg::QueryNumPoolsRequest {},
            )
            .unwrap();

        assert_eq!(res.num_pools, 1);
    }
}
