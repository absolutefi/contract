use cosmwasm_std::{Addr, Uint128};
use cw_asset::{Asset};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{SellParam, Sale, SaleProgress, SaleStatus,};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub min_cap: [Uint128; 2],
    pub min_token_sale_amt: Uint128,
    pub token_code_id: u64,
    pub fee_percentage: Uint128,
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfigMsg{
        min_cap: [Uint128; 2],
        min_token_sale_amt: Uint128,
        token_code_id: u64,
        fee_percentage: Uint128,
    },
    CreatePresaleMsg{
        amount: Uint128,
        param: SellParam,
    },
    ParticipateMsg{
        id: u64,
        cur: Asset,
        allow_partial: bool,
    },
    ClaimMsg{
        id: u64,
    },
    RefundMsg{
        id: u64,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Admin {},
    Config {},
    Sale {
        id: u64,
    },
    Sales {
        start_after: Option<u64>,
        limit: Option<u64>,
        is_ascending: Option<bool>,
    },
    SalesOwner {
        address: Addr,
        start_after: Option<u64>,
        limit: Option<u64>,
        is_ascending: Option<bool>,
    },
    Progress {
        id: u64,
        address: Addr,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurrencyWhitelistResponse {
    pub token: Vec<Addr>,
    pub native: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistResponse {
    pub addresses: Vec<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SaleResponse {
    pub sale: Sale,
    pub progress: SaleProgress,
    pub status: SaleStatus,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SalesResponse {
    pub sales: Vec<SaleResponse>,
}



