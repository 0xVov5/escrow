#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, Uint128};
use cw2::set_contract_version;
use cw20::{Balance, Cw20Coin, Cw20CoinVerified, Cw20ExecuteMsg, Cw20ReceiveMsg};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ESCROWS, Escrow, GenericBalance};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:escrow";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // no setup
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Create {id, receive_token_address} => execute_create(deps, info, id, receive_token_address),
    }
}

pub fn execute_create(
    deps: DepsMut,
    info: MessageInfo,
    id: u32,
    receive_token_address: String
) -> Result<Response, ContractError> {
    let balance = Balance::from(info.funds.clone());
    if balance.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }
    let token1_address;
    let token1_amount;
    match balance {
        Balance::Cw20(token) => {
            token1_address = token.address.clone();
            token1_amount = token.amount.clone();
        },
        _ => ()
    };
    let temp_address = deps.api.addr_validate(&receive_token_address).ok();
    let token2_address;
    match temp_address {
        None => {
            return Err(ContractError::EmptyBalance {});
        },
        Some(address) => {
            token2_address = address;
        }
    }
    
    let escrow = Escrow {
        owner: info.sender.clone(),
        token1_address,
        token1_amount,
        token2_address,
        token2_amount: token1_amount * Uint128::new(2),
        is_complete: false,
        is_cancelled: false
    };

    ESCROWS.update(deps.storage, id, |existing| match existing {
        None => Ok(escrow),
        Some(_) => Err(ContractError::InvalidReceiveTokenAddress {}),
    })?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}
