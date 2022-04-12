use cosmwasm_std::Addr;
use cw_storage_plus::Map;

pub const GAME: Map<&Addr, String> = Map::new("game");
