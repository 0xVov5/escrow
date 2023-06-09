use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128};
use cw20::Cw20ReceiveMsg;

use crate::state::{Escrow};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Create(CreateMsg),
    Cancel {
        id: u32
    },
    Approve(ApproveMsg)
}

#[cw_serde]
pub struct CreateMsg {
    pub id: u32,
    pub amount: Uint128,
    pub token: Option<Cw20ReceiveMsg>
}

#[cw_serde]
pub struct ApproveMsg {
    pub id: u32,
    pub token: Option<Cw20ReceiveMsg>
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Show all open escrows. Return type is ListResponse.
    #[returns(ListResponse)]
    List {},
    /// Returns the details of the named escrow, error if not created
    /// Return type: DetailsResponse.
    #[returns(Escrow)]
    Details { id: u32 },
}

#[cw_serde]
pub struct ListResponse {
    /// list all registered ids
    pub escrows: Vec<String>,
}
