use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw20::Cw20ReceiveMsg;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Create(CreateMsg),
    Cancel {
        id: u32
    }
}

#[cw_serde]
pub struct CreateMsg {
    pub id: u32,
    pub amount: Uint128,
    pub token: Option<Cw20ReceiveMsg>
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
