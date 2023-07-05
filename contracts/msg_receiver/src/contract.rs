use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, TestArg};
use cosmwasm_std::{entry_point, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, SubMsg, Reply};
use cw2::set_contract_version;
use neutron_sdk::bindings::msg::NeutronMsg;

use crate::state::{STARGATE_QUERY_ID, STARGATE_REPLIES, TEST_ARGS};

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
    STARGATE_QUERY_ID.save(deps.storage, &0)?;
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
        ExecuteMsg::TestMsg { return_err, arg } => execute_test_arg(deps, info, return_err, arg),
        ExecuteMsg::CallStaking {} => execute_call_staking(deps),
        ExecuteMsg::StargateMsg { type_url, value } => {
            execute_stargate_msg(deps, info, type_url, value)
        },
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::TestMsg { arg } => to_binary(&query_test_msg(deps, env, arg)?),
    }
}

fn query_test_msg(deps: Deps, _env: Env, arg: String) -> StdResult<Option<TestArg>> {
    TEST_ARGS.may_load(deps.storage, &arg)
}

fn execute_test_arg(
    deps: DepsMut,
    info: MessageInfo,
    return_err: bool,
    arg: String,
) -> StdResult<Response<NeutronMsg>> {
    if return_err {
        return Err(StdError::generic_err("return error"));
    }

    TEST_ARGS.update::<_, StdError>(deps.storage, &arg, |maybe_t| match maybe_t {
        Some(t) => Ok(TestArg {
            sender: info.sender.to_string(),
            funds: info.funds,
            count: t.count + 1,
        }),
        None => Ok(TestArg {
            sender: info.sender.to_string(),
            funds: info.funds,
            count: 1,
        }),
    })?;

    Ok(Response::default().add_attribute("arg", arg))
}

fn execute_call_staking(deps: DepsMut) -> StdResult<Response<NeutronMsg>> {
    deps.querier.query_bonded_denom()?; // should fail since Neutron does not have staking module
    Ok(Response::default())
}

fn execute_stargate_msg(
    deps: DepsMut,
    _: MessageInfo,
    type_url: String,
    value: String,
) -> StdResult<Response<NeutronMsg>> {
    // let msg = bank::v1beta1::QueryBalanceRequest{
    //     address: "todo",
    //     denom: "abcd"
    // };
    //
    // let msg = CosmosMsg::Stargate{
    //     type_url: "/cosmos.bank.v1beta1.Query/Balance".to_string(),
    //     value: to_binary(&msg)?,
    // };
    let id = STARGATE_QUERY_ID.load(deps.storage)?;
    STARGATE_QUERY_ID.update(deps.storage, |c| -> StdResult<u64> { Ok(c + 1) })?;
    let msg = CosmosMsg::Stargate {
        type_url,
        value: Binary::from(value.as_bytes()),
    };
    let submsg = SubMsg::reply_always(msg, id);

    Ok(Response::new()
        .add_submessage(submsg)
        .add_attribute("stargate_query_id", id.to_string()))
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    let result_str = if msg.result.is_err() {
        msg.result.unwrap_err()
    } else {
        let result = msg.result.unwrap();
        result.data.map(|res| Binary::to_base64(&res)).unwrap_or_else(|| "kekw".to_string())
    };

    STARGATE_REPLIES.save(deps.storage, msg.id, &result_str)?;

    Ok(Response::default())
}
