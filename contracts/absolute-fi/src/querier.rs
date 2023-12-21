use cosmwasm_std::{Addr, Deps, Env, Order, StdResult};
use cw_storage_plus::{Bound, PrimaryKey};

use crate::{
  state::{ PRESALE, PRESALE_PROGRESS,PRESALE_PARTICIPANT_BY_PRESALE_ID, SaleProgressPersonal, SALE_OWNER }, msg::{SaleResponse, SalesResponse}
};

const DEF_ITER_LIMIT: u64 = 30;
const DEF_WL_LIMIT: u64 = 100;


pub fn query_sale(deps: Deps, env: Env, id: u64) -> StdResult<SaleResponse> {
    let sale = PRESALE.load(deps.storage, id)?;
    let progress = PRESALE_PROGRESS.load(deps.storage, id)?;
    let status = sale.status(&progress, env.block.time.seconds());

    Ok(SaleResponse {
        sale,
        progress,
        status,
    })
}

pub fn query_sales(
    deps: Deps,
    env: Env,
    start_after: Option<u64>,
    limit: Option<u64>,
    is_ascending: Option<bool>,
) -> StdResult<SalesResponse> {
    let bound = match is_ascending.unwrap_or(true) {
        true => (start_after.map(Bound::exclusive), None, Order::Ascending),
        false => (None, start_after.map(Bound::exclusive), Order::Descending),
    };

    let bound_p = bound.clone();

    let ss = PRESALE
        .range(deps.storage, bound.0, bound.1, bound.2)
        .map(|e| e.unwrap().1)
        .take(limit.unwrap_or(DEF_ITER_LIMIT) as usize);

    let progs = PRESALE_PROGRESS
        .range(deps.storage, bound_p.0, bound_p.1, bound_p.2)
        .map(|e| e.unwrap().1)
        .take(limit.unwrap_or(DEF_ITER_LIMIT) as usize);

    let now = env.block.time.seconds();

    let sales = ss
        .into_iter()
        .zip(progs.into_iter())
        .map(|(sale, progress)| {
            let status = sale.status(&progress, now);

            SaleResponse {
                sale,
                progress,
                status,
            }
        })
        .collect::<Vec<_>>();

    Ok(SalesResponse { sales })
}

pub fn query_sales_owner(
    deps: Deps,
    env: Env,
    address: Addr,
    start_after: Option<u64>,
    limit: Option<u64>,
    is_ascending: Option<bool>,
) -> StdResult<SalesResponse> {
    let bound = match is_ascending.unwrap_or(true) {
        true => (start_after.map(Bound::exclusive), None, Order::Ascending),
        false => (None, start_after.map(Bound::exclusive), Order::Descending),
    };

    let now = env.block.time.seconds();

    let sales = SALE_OWNER
        .prefix(&address)
        .keys(deps.storage, bound.0, bound.1, bound.2)
        .map(|e| {
            let id = e.unwrap();
            let sale = PRESALE.load(deps.storage, id).unwrap();
            let progress = PRESALE_PROGRESS.load(deps.storage, id).unwrap();
            let status = sale.status(&progress, now);

            SaleResponse {
                sale,
                progress,
                status,
            }
        })
        .take(limit.unwrap_or(DEF_ITER_LIMIT) as usize)
        .collect::<Vec<_>>();

    Ok(SalesResponse { sales })
}

pub fn query_progress(deps: Deps, id: u64, address: Addr) -> StdResult<SaleProgressPersonal> {
    Ok(PRESALE_PARTICIPANT_BY_PRESALE_ID
        .may_load(deps.storage, (&address, id))?
        .unwrap_or_default())
}
