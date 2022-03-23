use cosmwasm_std::{
  debug_print, to_binary, Api, BankMsg, Binary, Coin, CosmosMsg, Env, Extern, HandleResponse,
  HumanAddr, InitResponse, Querier, QueryRequest, StdError, StdResult, Storage, WasmQuery,
};
use rand::{seq::SliceRandom, SeedableRng};
use rand_chacha::ChaChaRng;
use sha2::{Digest, Sha256};

use crate::msg::{
  HandleMsg, InitMsg, JackpotResponse, JackpotsResponse, NFTQueries, NFTQueryAnswers, QueryMsg,
  ViewerInfo,
};
use crate::state::{config, config_read, State};
use lazy_static::lazy_static;

lazy_static! {
  static ref ALLOWED_WORDS: Vec<&'static str> = {
    let word_txt: &str = include_str!("words.txt");
    word_txt.split('\n').collect()
  };
}

const CLAIM_INTERVAL: u64 = 60 * 60 * 24 * 2; // 2 days

pub fn init<S: Storage, A: Api, Q: Querier>(
  deps: &mut Extern<S, A, Q>,
  env: Env,
  msg: InitMsg,
) -> StdResult<InitResponse> {
  let state = State {
    owner: env.message.sender,
    jackpots: msg.jackpots,
    funds_liberated: None,
    nft_contract: msg.nft_contract,
    nft_hash: msg.nft_hash,
  };

  config(&mut deps.storage).save(&state)?;

  Ok(InitResponse::default())
}

pub fn get_rng(env: &Env) -> ChaChaRng {
  let block_time: Vec<u8> = env.block.time.to_be_bytes().to_vec();
  let mut hasher = Sha256::new();
  hasher.update(block_time.as_slice());
  let random_seed: [u8; 32] = Sha256::digest(&block_time).into();
  ChaChaRng::from_seed(random_seed)
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
  deps: &mut Extern<S, A, Q>,
  env: Env,
  msg: HandleMsg,
) -> StdResult<HandleResponse> {
  match msg {
    HandleMsg::Fund {} => try_fund(deps, env),
    HandleMsg::LiberateFunds { target } => try_liberate_funds(deps, env, target),
    HandleMsg::UpdateComplexity { min, max, index } => {
      try_update_complexity(deps, env, min, max, index)
    }
    HandleMsg::NextWord { index } => try_next_word(deps, env, index),
    HandleMsg::ShowMeTheMoney {
      jackpot_index,
      nft_id,
      viewing_key,
    } => try_showing_me_the_money(deps, env, jackpot_index, nft_id, viewing_key),
  }
}

fn try_showing_me_the_money<S: Storage, A: Api, Q: Querier>(
  deps: &mut Extern<S, A, Q>,
  env: Env,
  jackpot_index: u8,
  nft_id: String,
  viewing_key: String,
) -> Result<HandleResponse, StdError> {
  let mut state = config(&mut deps.storage).load()?;
  let word = state.jackpots[jackpot_index as usize].word.clone();
  let word_index = ALLOWED_WORDS
    .iter()
    .position(|&r| r.to_ascii_uppercase() == word.to_ascii_uppercase())
    .unwrap_or(0);

  let nft_query = NFTQueries::PrivateMetadata {
    token_id: nft_id.clone(),
    viewer: Some(ViewerInfo {
      address: env.message.sender.clone(),
      viewing_key,
    }),
  };

  let nft_response =
    deps
      .querier
      .query::<NFTQueryAnswers>(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: state.nft_contract.clone(),
        callback_code_hash: state.nft_hash.clone(),
        msg: to_binary(&nft_query)?,
      }))?;

  if state.jackpots[jackpot_index as usize].first_claim.is_none() {
    state.jackpots[jackpot_index as usize].first_claim = Some(env.block.time);
  }

  match nft_response {
    NFTQueryAnswers::PrivateMetadata {
      token_uri: _,
      extension,
    } => {
      if let Some(extension) = extension {
        if let Some(traits) = extension.attributes {
          for t in traits {
            match t.trait_type {
              None => continue,
              Some(tt) => {
                if tt == *"Stamped Words" {
                  // check if the word id is in the list of words
                  if t.value.contains(&word_index.to_string()) {
                    state.jackpots[jackpot_index as usize]
                      .shown
                      .push(env.message.sender.clone());
                    config(&mut deps.storage).save(&state)?;
                    return Ok(HandleResponse::default());
                  }
                }
              }
            }
          }
          return Err(StdError::generic_err(
            "You don't have this word in your stamps",
          ));
        }
      }
    }
  }
  Err(StdError::generic_err(
    "NFT contract returned unexpected response",
  ))
}

fn try_next_word<S: Storage, A: Api, Q: Querier>(
  deps: &mut Extern<S, A, Q>,
  env: Env,
  index: u8,
) -> Result<HandleResponse, StdError> {
  let mut state = config(&mut deps.storage).load()?;

  if state.jackpots.len() <= index as usize {
    return Err(StdError::generic_err("index out of bounds"));
  }

  let jackpot = &mut state.jackpots[index as usize];

  if jackpot.first_claim.is_none() {
    return Err(StdError::generic_err("Nobody claimed this jackpot yet"));
  }

  let diff = env.block.time - jackpot.first_claim.unwrap();

  if diff < CLAIM_INTERVAL {
    return Err(StdError::generic_err(
      "Please give other players a chance to claim this jackpot as well",
    ));
  }

  let claimants_count = jackpot.shown.len();
  let prize_amount = jackpot.amount / (claimants_count as u64);

  let mut messages: Vec<CosmosMsg> = vec![];

  for (_, claimer) in jackpot.shown.iter().enumerate() {
    messages.push(CosmosMsg::Bank(BankMsg::Send {
      from_address: env.contract.address.clone(),
      to_address: claimer.clone(),
      amount: vec![Coin::new(prize_amount as u128, "uscrt")],
    }));
  }

  jackpot.word = get_word(&env, jackpot.complexity_min, jackpot.complexity_max);
  jackpot.first_claim = None;

  Ok(HandleResponse {
    messages,
    log: vec![],
    data: None,
  })
}

fn get_word(env: &Env, complexity_min: u32, complexity_max: u32) -> String {
  let mut rng = get_rng(env);

  let words: Vec<(String, u32)> = vec![
    ("outwars".to_string(), 3097524),
    ("busyness".to_string(), 990104),
    ("bareback".to_string(), 924369),
    ("dev".to_string(), 444781),
    ("trackway".to_string(), 439563),
    ("kyboshed".to_string(), 393287),
    ("jugglers".to_string(), 346648),
    ("accessed".to_string(), 339272),
    ("colorful".to_string(), 276767),
    ("scalawag".to_string(), 271136),
    ("postriot".to_string(), 199518),
    ("violably".to_string(), 158839),
    ("chemics".to_string(), 101476),
    ("shaley".to_string(), 92513),
    ("fungo".to_string(), 91897),
    ("cuddlers".to_string(), 89355),
    ("savagism".to_string(), 78129),
    ("winner".to_string(), 77496),
    ("hunnish".to_string(), 74683),
    ("flattops".to_string(), 71206),
    ("zastruga".to_string(), 66113),
    ("pyelitic".to_string(), 61718),
    ("coplot".to_string(), 51210),
  ];

  let words_in_range = words
    .iter()
    .filter(|(_, complexity)| complexity >= &complexity_min && complexity <= &complexity_max)
    .collect::<Vec<_>>();

  return words_in_range.choose(&mut rng).unwrap().0.clone();
}

fn try_update_complexity<S: Storage, A: Api, Q: Querier>(
  deps: &mut Extern<S, A, Q>,
  env: Env,
  min: u32,
  max: u32,
  index: u8,
) -> Result<HandleResponse, StdError> {
  let mut state = config(&mut deps.storage).load()?;

  if index >= state.jackpots.len() as u8 {
    return Err(StdError::generic_err("index out of bounds"));
  }

  if min > max {
    return Err(StdError::generic_err(
      "min must be less than or equal to max",
    ));
  }

  if state.owner != env.message.sender {
    return Err(StdError::generic_err(
      "only owner can update jackpot complexity",
    ));
  }

  state.jackpots[index as usize].complexity_min = min;
  state.jackpots[index as usize].complexity_max = max;

  config(&mut deps.storage).save(&state)?;

  Ok(HandleResponse::default())
}

fn try_liberate_funds<S: Storage, A: Api, Q: Querier>(
  deps: &mut Extern<S, A, Q>,
  env: Env,
  target: HumanAddr,
) -> Result<HandleResponse, StdError> {
  let mut state = config(&mut deps.storage).load()?;

  if env.message.sender != state.owner {
    return Err(StdError::generic_err("only the owner can liberate funds"));
  }

  if state.funds_liberated.is_some() {
    return Err(StdError::generic_err("funds already liberated"));
  }

  let total = state.jackpots.iter().map(|j| j.amount).sum::<u64>();

  let transfers = vec![CosmosMsg::Bank(BankMsg::Send {
    from_address: env.contract.address,
    to_address: target,
    amount: vec![Coin::new(total as u128, "uscrt")],
  })];

  state
    .jackpots
    .iter_mut()
    .enumerate()
    .for_each(|(_, jackpot)| {
      jackpot.amount = 0;
    });

  config(&mut deps.storage).save(&state)?;

  Ok(HandleResponse {
    messages: transfers,
    log: vec![],
    data: None,
  })
}

fn try_fund<S: Storage, A: Api, Q: Querier>(
  deps: &mut Extern<S, A, Q>,
  env: Env,
) -> Result<HandleResponse, StdError> {
  let mut state = config(&mut deps.storage).load()?;

  if state.funds_liberated.is_some() {
    return Err(StdError::generic_err(
      "funds liberated, contract is now closed",
    ));
  }

  if env.message.sent_funds.is_empty() {
    return Err(StdError::generic_err("Funds must be sent to this contract"));
  }

  if env.message.sent_funds[0].denom != "uscrt" {
    return Err(StdError::generic_err("Funds must be sent in uscrt"));
  }

  let money = env.message.sent_funds[0].amount.u128() as u64;
  let jackpots_count = state.jackpots.len();

  let piece = money / jackpots_count as u64;

  for i in 0..jackpots_count {
    state.jackpots[i].amount += piece;
  }

  config(&mut deps.storage).save(&state)?;

  Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
  deps: &Extern<S, A, Q>,
  msg: QueryMsg,
) -> StdResult<Binary> {
  match msg {
    QueryMsg::GetJackpots {} => to_binary(&query_jackpots(deps)?),
  }
}

fn query_jackpots<S: Storage, A: Api, Q: Querier>(
  deps: &Extern<S, A, Q>,
) -> StdResult<JackpotsResponse> {
  let state = config_read(&deps.storage).load()?;

  let mut jackpots_response: Vec<JackpotResponse> = vec![];
  for jackpot in state.jackpots {
    let claimable_time = if let Some(t) = jackpot.first_claim {
      Some(t + CLAIM_INTERVAL)
    } else {
      None
    };
    jackpots_response.push(JackpotResponse {
      word: jackpot.word.clone(),
      amount: jackpot.amount,
      claimants: jackpot.shown,
      claimable_time,
    });
  }
 
  Ok(JackpotsResponse {
    jackpots: jackpots_response,
  })
}
