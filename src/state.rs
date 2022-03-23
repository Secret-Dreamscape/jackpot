use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{HumanAddr, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};

use crate::msg::Jackpot;

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
  pub owner: HumanAddr,
  pub jackpots: Vec<Jackpot>,
  pub funds_liberated: Option<HumanAddr>,
  pub nft_contract: HumanAddr,
  pub nft_hash: String,
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
  singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
  singleton_read(storage, CONFIG_KEY)
}
