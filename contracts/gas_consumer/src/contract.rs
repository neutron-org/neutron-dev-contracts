use std::str;

use crate::{
    msg::{ExecuteMsg, InstantiateMsg},
    state::{Config, CONFIG},
};
use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response, StdError};

use cw2::set_contract_version;
use neutron_sdk::{NeutronError, NeutronResult};
use sha2::{Digest, Sha256};

const CONTRACT_NAME: &str = concat!("crates.io:neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> NeutronResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(
        deps.storage,
        &Config {
            owner: msg.owner,
            hash_iterations: msg.hash_iterations,
        },
    )?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> NeutronResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            hash_iterations,
        } => execute_update_config(deps, info, owner, hash_iterations),
        ExecuteMsg::Hashes {} => execute_hashes(deps),
    }
}

fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: String,
    hash_iterations: u64,
) -> NeutronResult<Response> {
    if info.sender.to_string() != CONFIG.load(deps.storage)?.owner {
        return Err(NeutronError::Std(StdError::generic_err("Unauthorized")));
    }

    CONFIG.save(
        deps.storage,
        &Config {
            owner,
            hash_iterations,
        },
    )?;
    Ok(Response::default())
}

fn execute_hashes(deps: DepsMut) -> NeutronResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    for i in 0..config.hash_iterations {
        let mut hasher = Sha256::new();

        hasher.update(i.to_le_bytes());

        // read hash digest and consume hasher
        hasher.finalize();
    }
    Ok(Response::default().add_attribute("hash_iterations", format!("{}", config.hash_iterations)))
}
