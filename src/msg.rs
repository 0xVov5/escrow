use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Create{
        id: u32,
        receive_token_address: String
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
