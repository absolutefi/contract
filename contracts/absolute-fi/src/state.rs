use cosmwasm_std::{Addr, Api, Uint128, StdResult};
use crate::ContractError;
use cw_controllers::Admin;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_asset::{AssetInfo, AssetInfoBase, AssetInfoUnchecked};


//SALE STATE

// global config
pub const CONFIG: Item<Config> = Item::new("config");
pub const ADMIN: Admin = Admin::new("admin");

pub const OWNER_CACHE: Item<Addr> = Item::new("owner_cache");

// sale related state
pub const PRESALE_ID: Item<u64> = Item::new("presale_id");
pub const PRESALE: Map<u64, Sale> = Map::new("presale");
pub const PRESALE_PROGRESS: Map<u64, SaleProgress> = Map::new("presale_progress");
pub const PRESALE_PARTICIPANT_BY_PRESALE_ID: Map<(&Addr, u64), SaleProgressPersonal> = Map::new("sale_progress_personal");
pub const PRESALE_WL: Map<(u64, &Addr), ()> = Map::new("sale_wl");

pub const TOKEN_ADDRESS_BY_PRESALE_ID: Map<u64, Addr> = Map::new("ta_pi");

// indexing helper for sale
pub const SALE_OWNER: Map<(&Addr, u64), ()> = Map::new("sale_owner");


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub min_cap: [Uint128; 2],
    pub min_token_sale_amt: Uint128,
    pub token_code_id: u64,
    pub fee_percentage: Uint128,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SellParam {
    pub referrer: Option<String>,
    // --
    pub start: u64,
    pub end: u64,
    // --
    pub token_sale_amt: Uint128,
    // --
    pub cur_info: AssetInfoUnchecked,
    pub soft_cap: Uint128,
    pub hard_cap: Uint128,
    pub max_cur_alloc_per: Option<Uint128>,
    pub owner_allocation: Uint128,
    pub token_name: String,
    pub token_symbol: String,
    pub token_project: String,
    pub token_description: String,
    pub token_marketing: String,
    pub token_logo: String,
    // --
    pub wl_end_time: Option<u64>,
}

impl SellParam {
    // pub fn assert_valid_wl(&self) -> Result<(), ContractError> {
    //     if let Some(wl) = self.wl_end_time {
    //         (wl > self.start && wl < self.end)
    //             .then(|| ())
    //             .ok_or_else(|| {
    //                 ContractError::sell("Whitelist end time must be between start and end")
    //             })?;
    //     }

    //     Ok(())
    // }

    pub fn assert_start_end(&self, now: u64) -> Result<(), ContractError> {
        (self.end > self.start)
            .then(|| ())
            .ok_or_else(|| ContractError::sell("Invalid end date, must be after start"))?;

        (self.start >= now)
            .then(|| ())
            .ok_or_else(|| ContractError::sell("Invalid start date, cannot be in the past"))?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Sale {
    pub id: u64,
    pub created_at: u64,
    // --
    pub owner: Addr,
    pub referrer: Option<Addr>,
    // --
    pub start: u64,
    pub end: u64,
    // --
    pub token_addr: Addr,
    pub token_sale_amt: Uint128,
    // --
    pub cur_info: AssetInfo,
    pub soft_cap: Uint128,
    pub hard_cap: Uint128,
    pub max_cur_alloc_per: Option<Uint128>,
    // --
    pub wl_end_time: Option<u64>,
    pub owner_allocation: Uint128,
    pub token_name: String,
    pub token_symbol: String,
    pub token_project: String,
    pub token_description: String,
    pub token_marketing: String,
    pub token_logo: String,
}

impl Sale {
    pub fn from_param(
        api: &dyn Api,
        param: SellParam,
        id: u64,
        now: u64,
        owner: Addr,
        token_addr: Addr,
    ) -> StdResult<Self> {
        Ok(Self {
            id,
            created_at: now,
            owner,
            referrer: param.referrer.map(|r| api.addr_validate(&r)).transpose()?,
            start: param.start,
            end: param.end,
            token_addr,
            owner_allocation: param.owner_allocation,
            token_name: param.token_name,
            token_symbol: param.token_symbol,
            token_project: param.token_project,
            token_description: param.token_description,
            token_marketing: param.token_marketing,
            token_logo: param.token_logo,
            token_sale_amt: param.token_sale_amt,
            cur_info: param.cur_info.check(api, None)?,
            soft_cap: param.soft_cap,
            hard_cap: param.hard_cap,
            max_cur_alloc_per: param.max_cur_alloc_per,
            wl_end_time: param.wl_end_time,
        })
    }

    pub fn status(&self, progress: &SaleProgress, now: u64) -> SaleStatus {

        if self.start > now {
            return SaleStatus::NotStarted;
        }

        if progress.token_sold == self.token_sale_amt && progress.cur_raised == self.hard_cap {
            return SaleStatus::Filled;
        }

        if now > self.end && progress.cur_raised < self.soft_cap {
            return SaleStatus::Failed;
        }

        if now >= self.start && self.end >= now {
            return SaleStatus::Ongoing;
        }

        SaleStatus::Ended
    }

    pub fn token_info(&self) -> AssetInfo {
        AssetInfo::cw20(self.token_addr.clone())
    }

}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SaleStatus {
    NotStarted,
    Ongoing,
    Ended,
    Filled,
    Failed,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, JsonSchema, Default)]
pub struct SaleProgress {
    pub token_sold: Uint128,
    pub cur_raised: Uint128,
    // --
    pub token_claimed: Uint128,
    // --
    pub is_excess_sent: bool,
    pub cur_excess: Uint128,
    pub token_excess: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, JsonSchema, Default)]
pub struct SaleProgressPersonal {
    pub is_claimed: bool,
    pub is_refunded: bool,
    pub token_got: Uint128,
    pub cur_spent: Uint128,
}
