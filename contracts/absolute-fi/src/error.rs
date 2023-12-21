use cosmwasm_std::{StdError, Uint128};
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Admin: {0}")]
    Admin(#[from] AdminError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Fee amount mismatched, expected {0} found {1}")]
    FeeMismatch(Uint128, Uint128),

    #[error("Only sale owner is authorized")]
    OnlySaleOwner,

    #[error("Account's sale participation not found")]
    ParticipationNotFound,

    #[error("Whitelist: {0}")]
    Whitelist(String),

    #[error("Sell: {0}")]
    Sell(String),

    #[error("Buy: {0}")]
    Buy(String),

    #[error("Claim: {0}")]
    Claim(String),

    #[error("Refund: {0}")]
    Refund(String),

    #[error("This sale is not started yet")]
    NotStarted,

    #[error("This sale is currently ongoing")]
    Ongoing,

    #[error("This sale is already ended")]
    AlreadyEnded,

    #[error("This sale is already filled")]
    AlreadyFilled,

    #[error("This sale is already filled")]
    InvalidReplyId,

    #[error("This sale amount soft cap has not been reached, please `refund` instead")]
    Failed,

    #[error("This sale amount soft cap has been reached, please `claim` instead")]
    Ended,

    #[error("{0}")]
    Custom(String),
}

impl ContractError {
    pub fn whitelist(description: impl Into<String>) -> Self {
        Self::Whitelist(description.into())
    }

    pub fn sell(description: impl Into<String>) -> Self {
        Self::Sell(description.into())
    }

    pub fn buy(description: impl Into<String>) -> Self {
        Self::Buy(description.into())
    }

    pub fn claim(description: impl Into<String>) -> Self {
        Self::Claim(description.into())
    }

    pub fn refund(description: impl Into<String>) -> Self {
        Self::Refund(description.into())
    }

    pub fn custom(description: impl Into<String>) -> Self {
        Self::Custom(description.into())
    }
}

#[derive(Error, Debug)]
pub enum LockError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Sent amount and lock amount mismatched")]
    AmountMismatched,

    #[error("Lock amount must be gteq than {0}")]
    LockMinimum(Uint128),

    #[error("Token address mismatched, must be original token")]
    TokenMismatched,

    #[error("Vault owner address mismatched")]
    OwnerMismatched,

    #[error("{0}")]
    Custom(String),
}

impl LockError {
    pub fn custom(description: impl Into<String>) -> Self {
        Self::Custom(description.into())
    }
}

#[derive(Error, Debug)]
pub enum ClaimError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Vault id mismatched")]
    VaultIdMismatched,

    #[error("{0}")]
    Custom(String),
}

impl ClaimError {
    pub fn custom(description: impl Into<String>) -> Self {
        Self::Custom(description.into())
    }
}
