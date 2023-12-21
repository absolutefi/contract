#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Reply};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::querier::{query_sale, query_sales, query_sales_owner, query_progress};
use crate::state::{CONFIG, Config, ADMIN, OWNER_CACHE, TOKEN_ADDRESS_BY_PRESALE_ID, PRESALE_ID};
use crate::handler::{execute_update_config, execute_create_presale, execute_participate, execute_claim, execute_refund};
use cw_utils::parse_reply_instantiate_data;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:absolute-fi";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        min_cap: msg.min_cap,
        min_token_sale_amt: msg.min_token_sale_amt,
        token_code_id: msg.token_code_id,
        fee_percentage: msg.fee_percentage
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;
    PRESALE_ID.save(deps.storage, &1)?;
    ADMIN.set(deps, Some(info.sender.clone()))?;
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfigMsg{ min_cap, min_token_sale_amt, token_code_id, fee_percentage} => execute_update_config(deps, info, min_cap, min_token_sale_amt, token_code_id, fee_percentage),
        ExecuteMsg::CreatePresaleMsg{ amount, param} => execute_create_presale(deps, env, info.clone(), info.sender.clone(), amount, param),
        ExecuteMsg::ParticipateMsg{ id, cur , allow_partial} => execute_participate(deps, env, info.sender, id, cur, allow_partial),
        ExecuteMsg::ClaimMsg{ id } => execute_claim(deps, env, info, id),
        ExecuteMsg::RefundMsg{ id } => execute_refund(deps, env, info, id)
    }
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Admin {} => to_binary(&ADMIN.query_admin(deps)?),
        QueryMsg::Config {} => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::Sale { id } => to_binary(&query_sale(deps, env, id)?),
        QueryMsg::Sales {
            start_after,
            limit,
            is_ascending,
        } => to_binary(&query_sales(deps, env, start_after, limit, is_ascending)?),
        QueryMsg::SalesOwner {
            address,
            start_after,
            limit,
            is_ascending,
        } => to_binary(&query_sales_owner(
            deps,
            env,
            address,
            start_after,
            limit,
            is_ascending,
        )?),
        QueryMsg::Progress { id, address } => to_binary(&query_progress(deps, id, address)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        1 => {
            let contract_addr = deps
                .api
                .addr_validate(&parse_reply_instantiate_data(msg).unwrap().contract_address)?;
            let presale_id = PRESALE_ID.load(deps.storage)?;
            TOKEN_ADDRESS_BY_PRESALE_ID.save(deps.storage,presale_id, &contract_addr)?;
            OWNER_CACHE.remove(deps.storage);


            Ok(Response::new().add_attribute("token_addr", contract_addr))
        }
        _ => Err(ContractError::InvalidReplyId)?,
    }
}