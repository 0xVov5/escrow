use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Uint128};
use cw_storage_plus::Map;
use cw20::{Cw20CoinVerified};

#[cw_serde]
pub struct Escrow {
    owner: Addr,
    token1_address: Addr,
    token1_amount: Uint128,
    token2_address: Addr,
    token2_amount: Uint128,
    is_complete: bool,
    is_cancelled: bool
}

pub const ESCROWS: Map<&str, Escrow> = Map::new("escrow");

#[cw_serde]
#[derive(Default)]
pub struct GenericBalance {
    pub native: Vec<Coin>,
    pub cw20: Vec<Cw20CoinVerified>,
}