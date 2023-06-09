use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    
    #[error("Send some coins to create an escrow")]
    EmptyBalance {},

    #[error("Escrow id already in use")]
    AlreadyInUse {},

    #[error("Invalid CW20 token address")]
    InvalidTokenAddress {},

    #[error("This escrow was already cancelled")]
    AlreadyCancel {},
}
