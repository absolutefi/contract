use cosmwasm_std::{
  to_binary, Addr, Coin, Decimal, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
  WasmMsg, CosmosMsg, to_json_binary, SubMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20Coin, Logo, MinterResponse, };
use cw20_base::msg::{InstantiateMarketingInfo, InstantiateMsg as Cw20InstantiateMsg};
use cw_asset::{Asset, AssetInfoBase, AssetInfoUnchecked, AssetUnchecked};

use crate::{
  state::{
      ADMIN, CONFIG, PRESALE_ID,
      PRESALE,
      PRESALE_PROGRESS,
      PRESALE_PARTICIPANT_BY_PRESALE_ID,
      PRESALE_WL,
      SALE_OWNER,
      SaleProgress,
      SellParam,
      Sale,
      SaleStatus,
      TOKEN_ADDRESS_BY_PRESALE_ID
  },
  ContractError,
};


#[allow(clippy::too_many_arguments)]
pub fn execute_update_config(
  deps: DepsMut,
  info: MessageInfo,
  min_cap: [Uint128; 2],
  min_token_sale_amt: Uint128,
  token_code_id: u64,
  fee_percentage: Uint128,
) -> Result<Response, ContractError> {
  ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

  let mut cfg = CONFIG.load(deps.storage)?;
  cfg.min_cap= min_cap;
  cfg.min_token_sale_amt= min_token_sale_amt;
  cfg.token_code_id= token_code_id;
  cfg.fee_percentage= fee_percentage;


  CONFIG.save(deps.storage, &cfg)?;

  Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn execute_create_presale(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  owner: Addr,
  amount: Uint128,
  param: SellParam,
) -> Result<Response, ContractError> {
  let config = CONFIG.load(deps.storage)?;
  let id = PRESALE_ID.load(deps.storage)?;


  param.assert_start_end(env.block.time.seconds())?;

  PRESALE.save(
      deps.storage,
      id,
      &Sale::from_param(
          deps.api,
          param.clone(),
          id,
          env.block.time.seconds(),
          owner.clone(),
          info.sender.clone(),
      )?,
  )?;
  let messages =SubMsg::reply_on_success(WasmMsg::Instantiate {
    admin: None ,
    code_id: config.token_code_id,
    msg: to_binary(&Cw20InstantiateMsg {
        name: param.token_name.clone(),
        symbol: param.token_symbol.clone(),
        decimals: 6,
        initial_balances: vec![Cw20Coin {
            address: info.sender.clone().to_string(),
            amount: param.clone().hard_cap,
        }],
        mint: None,
        marketing: None 
        })?,
        funds: vec![],
        label: "Absolute Fi".to_string()
    }, 1);
    PRESALE_ID.save(deps.storage, &(id + 1))?;
    PRESALE_PROGRESS.save(deps.storage, id, &SaleProgress::default())?;
    SALE_OWNER.save(deps.storage, (&owner, id), &())?;

  Ok(Response::new()
        .add_submessage(messages)
        .add_attribute("action", "sell")
        .add_attribute("id", id.to_string())
        .add_attribute("owner", owner))
}


pub fn execute_participate(
    deps: DepsMut,
    env: Env,
    buyer: Addr,
    id: u64,
    cur: Asset,
    allow_partial: bool,
) -> Result<Response, ContractError> {
    let sale = PRESALE.load(deps.storage, id)?;
    let mut sale_prog = PRESALE_PROGRESS.load(deps.storage, id)?;
    let mut sale_pers = PRESALE_PARTICIPANT_BY_PRESALE_ID
        .may_load(deps.storage, (&buyer, id))?
        .unwrap_or_default();

    let mut msgs = vec![];

    (cur.info == sale.cur_info)
        .then(|| ())
        .ok_or_else(|| ContractError::buy("Currency token mismatched"))?;
    (sale.owner != buyer)
        .then(|| ())
        .ok_or_else(|| ContractError::buy("Sale owner cannot participate"))?;

    match sale.status(&sale_prog, env.block.time.seconds()) {
        SaleStatus::NotStarted => {
            Err(ContractError::NotStarted)?;
        }
        SaleStatus::Failed | SaleStatus::Ended => {
            Err(ContractError::AlreadyEnded)?;
        }
        SaleStatus::Filled => {
            Err(ContractError::AlreadyFilled)?;
        }
        SaleStatus::Ongoing => {
            let token_bought_amt = cur
                .amount
                .multiply_ratio(sale.token_sale_amt, (sale.hard_cap - sale.owner_allocation));
            if let Some(cap) = sale.max_cur_alloc_per {
                if cur.amount + sale_prog.cur_raised > cap {
                    Err(ContractError::buy(
                        "Token bought exceed maximum allowed per account",
                    ))?;
                }
            }

            if let Some(wl_end) = sale.wl_end_time {
                if wl_end <= env.block.time.seconds() {
                    PRESALE_WL
                        .has(deps.storage, (id, &buyer))
                        .then(|| ())
                        .ok_or_else(|| {
                            ContractError::whitelist("Buyer address is not whitelisted")
                        })?;
                }
            }

            if token_bought_amt + sale_prog.token_sold > sale.token_sale_amt
                || cur.amount + sale_prog.cur_raised > sale.hard_cap
            {
                match allow_partial {
                    true => {
                        let pt_token_bought_amt = sale.token_sale_amt - sale_prog.token_sold;
                        let pt_cur_spent = (sale.hard_cap - sale.owner_allocation) - sale_prog.cur_raised;

                        sale_prog.token_sold += pt_token_bought_amt;
                        sale_prog.cur_raised += pt_cur_spent;

                        sale_pers.token_got += pt_token_bought_amt;
                        sale_pers.cur_spent += pt_cur_spent;

                        PRESALE_PROGRESS.save(deps.storage, id, &sale_prog)?;
                        PRESALE_PARTICIPANT_BY_PRESALE_ID.save(deps.storage, (&buyer, id), &sale_pers)?;
                        msgs.push(
                            Asset {
                                info: cur.info,
                                amount: cur.amount - pt_cur_spent,
                            }
                            .transfer_msg(&buyer)?,
                        );
                    }
                    false => Err(ContractError::buy("Token bought exceed sale amount"))?,
                };
            } else {
                sale_prog.token_sold += token_bought_amt;
                sale_prog.cur_raised += cur.amount;

                sale_pers.token_got += token_bought_amt;
                sale_pers.cur_spent += cur.amount;

                PRESALE_PROGRESS.save(deps.storage, id, &sale_prog)?;
                PRESALE_PARTICIPANT_BY_PRESALE_ID.save(deps.storage, (&buyer, id), &sale_pers)?;
            }
        }
    };

    Ok(Response::new()
        .add_messages(msgs))
}

pub fn execute_claim(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let sale = PRESALE.load(deps.storage, id)?;
    let mut sale_prog = PRESALE_PROGRESS.load(deps.storage, id)?;

    let mut msgs = vec![];

    match sale.status(&sale_prog, env.block.time.seconds()) {
        SaleStatus::NotStarted => {
            Err(ContractError::NotStarted)?;
        }
        SaleStatus::Ongoing => {
            Err(ContractError::Ongoing)?;
        }
        SaleStatus::Failed => {
            Err(ContractError::Failed)?;
        }
        SaleStatus::Ended | SaleStatus::Filled => {
            let token_address = TOKEN_ADDRESS_BY_PRESALE_ID.load(deps.storage, id)?;
            let presale = PRESALE.load(deps.storage, id)?;
            match info.sender == sale.owner {
                true => {
                    // check if excess is sent
                    (!sale_prog.is_excess_sent)
                        .then(|| ())
                        .ok_or_else(|| ContractError::claim("Already claim currency"))?;

                    // update current state
                    sale_prog.is_excess_sent = true;
                    // Change to owner click mint first
                    PRESALE_PROGRESS.save(deps.storage, id, &sale_prog)?;

                    // transfer excess amount of currency
                    msgs.push(
                        Asset {
                            info: AssetInfoBase::Cw20((token_address)),
                            amount: presale.owner_allocation,
                        }
                        .transfer_msg(info.sender.clone())?,
                    );
                    msgs.push(
                        Asset {
                            info: presale.cur_info,
                            amount: sale_prog.cur_raised,
                        }
                        .transfer_msg(info.sender.clone())?,
                    );
                }
                false => {
                    // participant
                    let mut sale_pers = PRESALE_PARTICIPANT_BY_PRESALE_ID
                        .load(deps.storage, (&info.sender, id))
                        .map_err(|_| ContractError::ParticipationNotFound)?;

                    // check if already claimed
                    (!sale_pers.is_claimed)
                        .then(|| ())
                        .ok_or_else(|| ContractError::claim("Already claim token"))?;

                    // update current state
                    sale_pers.is_claimed = true;
                    PRESALE_PARTICIPANT_BY_PRESALE_ID.save(deps.storage, (&info.sender, id), &sale_pers)?;

                    // update total token claimed state
                    sale_prog.token_claimed += sale_pers.token_got;
                    PRESALE_PROGRESS.save(deps.storage, id, &sale_prog)?;

                    // transfer token bought
                    //change to mint and transfer
                    msgs.push(
                        Asset {
                            info: AssetInfoBase::Cw20((token_address)),
                            amount: sale_pers.token_got,
                        }
                        .transfer_msg(info.sender)?,
                    )
                }
            };
        }
    };

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "claim"))
}

pub fn execute_refund(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let sale = PRESALE.load(deps.storage, id)?;
    let mut sale_prog = PRESALE_PROGRESS.load(deps.storage, id)?;

    let mut msgs = vec![];

    match sale.status(&sale_prog, env.block.time.seconds()) {
        SaleStatus::NotStarted => {
            Err(ContractError::NotStarted)?;
        }
        SaleStatus::Ongoing => {
            Err(ContractError::Ongoing)?;
        }
        SaleStatus::Ended | SaleStatus::Filled => {
            Err(ContractError::Ended)?;
        }
        SaleStatus::Failed => match info.sender == sale.owner {
            true => {
                // owner of the token sale -> claim token back

                // check if already refunded
                (!sale_prog.is_excess_sent)
                    .then(|| ())
                    .ok_or_else(|| ContractError::refund("Already refunded excess token"))?;

                let total_token = sale.token_sale_amt;

                msgs.push(
                    Asset {
                        info: sale.token_info(),
                        amount: total_token,
                    }
                    .transfer_msg(&info.sender)?,
                );

                // save current sale progress state
                sale_prog.is_excess_sent = true;
                sale_prog.token_excess = total_token;

                PRESALE_PROGRESS.save(deps.storage, id, &sale_prog)?;
            }
            false => {
                // participant account -> claim back currency spent
                let mut sale_pers = PRESALE_PARTICIPANT_BY_PRESALE_ID
                    .load(deps.storage, (&info.sender, id))
                    .map_err(|_| ContractError::ParticipationNotFound)?;

                // check if already refunded
                (!sale_pers.is_refunded && !sale_pers.is_claimed)
                    .then(|| ())
                    .ok_or_else(|| {
                        ContractError::refund("Already refunded or claimed to participant")
                    })?;

                msgs.push(
                    Asset {
                        info: sale.cur_info,
                        amount: sale_pers.cur_spent,
                    }
                    .transfer_msg(&info.sender)?,
                );

                // save current sale personal progress state
                sale_pers.is_refunded = true;
                PRESALE_PARTICIPANT_BY_PRESALE_ID.save(deps.storage, (&info.sender, id), &sale_pers)?;
            }
        },
    };

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "refund"))
}