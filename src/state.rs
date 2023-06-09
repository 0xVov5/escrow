use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Uint128};
use cw_storage_plus::Map;
use cw20::{Cw20CoinVerified};

#[cw_serde]
pub struct Escrow {
    pub id: u32,
    pub owner: Addr,
    pub coin_amount: Uint128,
    pub token_amount: Uint128,
    pub is_coin_escrow: bool,
    pub is_complete: bool,
    pub is_cancelled: bool,
    pub balance: GenericBalance,
}

pub const ESCROWS: Map<&str, Escrow> = Map::new("escrow");

#[cw_serde]
#[derive(Default)]
pub struct GenericBalance {
    pub native: Vec<Coin>,
    pub cw20: Vec<Cw20CoinVerified>,
}