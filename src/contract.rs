#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, SubMsg, WasmMsg, Addr, BankMsg, Storage, Order};
use cw2::set_contract_version;
use cw20::{Balance, Cw20ExecuteMsg, Cw20CoinVerified};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, CreateMsg, ApproveMsg, ListResponse};
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
        ExecuteMsg::Create(msg) => execute_create(deps, env, info, msg),
        ExecuteMsg::Cancel {id} => execute_cancel(deps, info, id),
        ExecuteMsg::Approve(msg) => execute_approve(deps, info, msg),
    }
}

pub fn execute_create(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CreateMsg
) -> Result<Response, ContractError> {
    let mut balance = Balance::from(info.funds.clone());
    let amount = msg.token.map(|token| token.amount).unwrap_or(Uint128::new(0));
    if balance.is_empty() && amount == Uint128::new(0) {
        return Err(ContractError::EmptyBalance {});
    }
    if balance.is_empty() {
        balance = Balance::Cw20(Cw20CoinVerified {
            address: info.sender.clone(),
            amount,
        });
    }
    
    let mut coin_amount = Uint128::new(0);
    let mut token_amount = Uint128::new(0);
    let mut is_coin_escrow;
    
    let escrow_balance = match balance {
        Balance::Native(balance) => {
            if let Some(coin) = balance.0.get(0) {
                coin_amount = coin.amount;
                token_amount = msg.amount;
                is_coin_escrow = true;
            } else {
                return Err(ContractError::EmptyBalance {});
            }
            GenericBalance {
                native: balance.0,
                cw20: vec![],
            }
        },
        Balance::Cw20(token) => {
            coin_amount = amount;
            token_amount = token.amount.clone();
            is_coin_escrow = false;
            GenericBalance {
                native: vec![],
                cw20: vec![token],
            }
        },
    };
    
    let escrow = Escrow {
        id: msg.id,
        owner: info.sender.clone(),
        coin_amount,
        token_amount,
        is_coin_escrow,
        is_complete: false,
        is_cancelled: false,
        balance: escrow_balance.clone()
    };

    ESCROWS.update(deps.storage, &msg.id.to_string(), |existing| match existing {
        None => Ok(escrow),
        Some(_) => Err(ContractError::AlreadyInUse {}),
    })?;

    let messages: Vec<SubMsg> = send_tokens(&env.contract.address, &escrow_balance.clone())?;

    let res = Response::new()
    .add_attributes(vec![("action", "create"), ("id", &msg.id.to_string())])
    .add_submessages(messages);

    Ok(res)
}

pub fn execute_cancel(
    deps: DepsMut,
    info: MessageInfo,
    id: u32,
) -> Result<Response, ContractError> {
    let mut escrow = ESCROWS.load(deps.storage, &id.to_string())?;
    if escrow.owner != info.sender {
        return Err(ContractError::Unauthorized {})
    }
    if escrow.is_complete {
        return Err(ContractError::AlreadyComplete {});
    }
    if escrow.is_cancelled {
        return Err(ContractError::AlreadyCancel {})
    }
    escrow.is_cancelled = true;
    ESCROWS.save(deps.storage, &id.to_string(), &escrow)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "cancel"),
        ("id", id.to_string().as_str()),
    ]))
}

pub fn execute_approve(
    deps: DepsMut,
    info: MessageInfo,
    msg: ApproveMsg
) -> Result<Response, ContractError> {
    let mut escrow = ESCROWS.load(deps.storage, &msg.id.to_string())?;
    if escrow.is_complete {
        return Err(ContractError::AlreadyComplete {});
    }
    if escrow.is_cancelled {
        return Err(ContractError::AlreadyCancel {});
    }

    let balance = Balance::from(info.funds.clone());
    let mut coin_amount = Uint128::new(0);
    let escrow_balance = match balance {
        Balance::Native(balance) => {
            if let Some(coin) = balance.0.get(0) {
                coin_amount = coin.amount;
            }
            GenericBalance {
                native: balance.0,
                cw20: vec![],
            }
        },
        Balance::Cw20(token) => {
            GenericBalance {
                native: vec![],
                cw20: vec![token],
            }
        },
    };

    let amount: Uint128 = msg.token.map(|token| token.amount).unwrap_or(Uint128::new(0));

    if escrow.is_coin_escrow {
        if amount != escrow.token_amount {
            return Err(ContractError::InvalidAmount {});
        }
    } else {
        if coin_amount != escrow.coin_amount {
            return Err(ContractError::InvalidAmount {});
        }
    }

    let receive_messages: Vec<SubMsg> = send_tokens(&info.sender, &escrow.balance)?;
    let send_messages: Vec<SubMsg> = send_tokens(&escrow.owner, &escrow_balance)?;

    escrow.is_complete = true;
    ESCROWS.save(deps.storage, &msg.id.to_string(), &escrow)?;

    let res = Response::new()
    .add_attributes(vec![("action", "approve"), ("id", &msg.id.to_string())])
    .add_submessages(receive_messages)
    .add_submessages(send_messages);

    Ok(res)
}

fn send_tokens(to: &Addr, balance: &GenericBalance) -> StdResult<Vec<SubMsg>> {
    let native_balance = &balance.native;
    let mut msgs: Vec<SubMsg> = if native_balance.is_empty() {
        vec![]
    } else {
        vec![SubMsg::new(BankMsg::Send {
            to_address: to.into(),
            amount: native_balance.to_vec(),
        })]
    };

    let cw20_balance = &balance.cw20;
    let cw20_msgs: StdResult<Vec<_>> = cw20_balance
        .iter()
        .map(|c| {
            let msg = Cw20ExecuteMsg::Transfer {
                recipient: to.into(),
                amount: c.amount,
            };
            let exec = SubMsg::new(WasmMsg::Execute {
                contract_addr: c.address.to_string(),
                msg: to_binary(&msg)?,
                funds: vec![],
            });
            Ok(exec)
        })
        .collect();
    msgs.append(&mut cw20_msgs?);
    Ok(msgs)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::List {} => to_binary(&query_list(deps)?),
        QueryMsg::Details { id } => to_binary(&query_details(deps, id)?),
    }
}

fn query_details(deps: Deps, id: u32) -> StdResult<Escrow> {
    let escrow = ESCROWS.load(deps.storage, &id.to_string())?;

    let details = escrow;
    Ok(details)
}

fn query_list(deps: Deps) -> StdResult<ListResponse> {
    Ok(ListResponse {
        escrows: all_escrow_ids(deps.storage)?,
    })
}

/// This returns the list of ids for all registered escrows
pub fn all_escrow_ids(storage: &dyn Storage) -> StdResult<Vec<String>> {
    ESCROWS
        .keys(storage, None, None, Order::Ascending)
        .collect()
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins};
    use cw20::Cw20ReceiveMsg;

    use super::*;

    #[test]
    fn create_test() {
        let mut deps = mock_dependencies();

        let instantiate_msg = InstantiateMsg {};
        let info = mock_info(&String::from("anyone"), &[]);
        let res: Response = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let sender = String::from("source");
        let balance = coins(100, "tokens");
        let info = mock_info(&sender, &balance);
        let create = CreateMsg {
            id: 1,
            amount: Uint128::new(100000000000000000000),
            token: None
        };
        let msg = ExecuteMsg::Create(create.clone());
        let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();
        assert_eq!(1, res.messages.len());
        assert_eq!(("action", "create"), res.attributes[0]);

        let info = mock_info(&sender, &[]);
        let create = CreateMsg {
            id: 2,
            amount: Uint128::new(100000000000000000000),
            token: Some(Cw20ReceiveMsg{
                sender: sender,
                amount: Uint128::new(100),
                msg: to_binary(&msg).unwrap()
            }),
        };
        let msg = ExecuteMsg::Create(create.clone());
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }

    #[test]
    fn cancel_escrow() {
        let mut deps = mock_dependencies();

        let instantiate_msg = InstantiateMsg {};
        let info = mock_info(&String::from("anyone"), &[]);
        instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

        let sender = String::from("source");
        let balance = coins(100, "tokens");
        let info = mock_info(&sender, &balance);
        let create = CreateMsg {
            id: 1,
            amount: Uint128::new(100000000000000000000),
            token: None
        };
        let msg = ExecuteMsg::Create(create.clone());
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::Cancel{
            id: 1,
        };
        let info = mock_info(&sender, &[]);
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }

    #[test]
    fn approve_escrow() {
        let mut deps = mock_dependencies();

        let instantiate_msg = InstantiateMsg {};
        let info = mock_info(&String::from("anyone"), &[]);
        instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

        let sender = String::from("source");
        let balance = coins(100, "tokens");
        let info = mock_info(&sender, &balance);
        let create = CreateMsg {
            id: 1,
            amount: Uint128::new(100000000000000000000),
            token: None
        };
        let msg = ExecuteMsg::Create(create.clone());
        execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

        let receiver = String::from("receiver");
        let receiver_info = mock_info(&receiver, &[]);
        
        let approve = ApproveMsg {
            id: 1,
            token: Some(Cw20ReceiveMsg{
                sender: receiver,
                amount: Uint128::new(100000000000000000000),
                msg: to_binary(&msg).unwrap()
            }),
        };
        let receiver_msg = ExecuteMsg::Approve(approve.clone());
        execute(deps.as_mut(), mock_env(), receiver_info, receiver_msg).unwrap();
    }
}
