use crate::{msg::{ExecuteMsg, InstantiateMsg, MigrateMsg}, state::CHUNKS};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult,
};

use cw2::set_contract_version;
use neutron_sdk::bindings::msg::NeutronMsg;


const CONTRACT_NAME: &str = concat!("crates.io:neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: instantiate");
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response<NeutronMsg>> {
    deps.api
        .debug(format!("WASMDEBUG: execute: received msg: {:?}", msg).as_str());
    match msg {
        ExecuteMsg::ConsumeGas { chunks, chunk_size } => {
            execute_test_arg(deps, info, chunks, chunk_size)
        }
    }
}


fn execute_test_arg(
    deps: DepsMut,
    info: MessageInfo,
    chunks: u64,
    chunk_size: Option<u64>,
) -> StdResult<Response<NeutronMsg>> {


    let (last_chank_id,_) = CHUNKS.last(deps.storage)?.unwrap_or_default();
    for i in last_chank_id+1..last_chank_id+chunks {
        CHUNKS.save(deps.storage, i, &"s".repeat(chunk_size.unwrap_or(1) as usize))?
    }



    Ok(Response::default().
        add_attribute("chunks", format!("{}",chunks)).
        add_attribute("chunk_size", format!("{}",chunk_size.unwrap_or(1)))
    )
}



