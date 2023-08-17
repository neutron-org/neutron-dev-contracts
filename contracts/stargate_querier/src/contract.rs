use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmos_sdk_proto::cosmos::{auth, bank};
use cosmos_sdk_proto::ibc;
use cosmwasm_std::{
    entry_point, to_binary, Binary, ContractResult, Deps, DepsMut, Empty, Env, MessageInfo,
    QueryRequest, Response, StdError, StdResult, SystemResult,
};
use std::str::from_utf8;

use crate::stargate;
use cw2::set_contract_version;
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::NeutronResult;
use prost::Message;
use serde_json_wasm::to_vec;

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
    _: DepsMut,
    _: Env,
    _: MessageInfo,
    _msg: ExecuteMsg,
) -> StdResult<Response<NeutronMsg>> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _: Env, msg: QueryMsg) -> NeutronResult<Binary> {
    deps.api
        .debug(format!("WASMDEBUG: execute: received msg: {:?}", msg).as_str());
    match msg {
        QueryMsg::BankBalance { address, denom } => query_bank_balance(deps, address, denom),
        QueryMsg::BankDenomMetadata { denom } => query_bank_denom_metadata(deps, denom),
        QueryMsg::BankParams {} => query_bank_params(deps),
        QueryMsg::BankSupplyOf { denom } => query_bank_supply_of(deps, denom),
        QueryMsg::AuthAccount { address } => query_auth_account(deps, address),
        QueryMsg::TransferDenomTrace { hash } => query_transfer_denom_trace(deps, hash),
        QueryMsg::IbcClientState { client_id } => query_ibc_client_state(deps, client_id),
        QueryMsg::IbcConsensusState {
            client_id,
            revision_number,
            revision_height,
            latest_height,
        } => query_ibc_consensus_state(
            deps,
            client_id,
            revision_number,
            revision_height,
            latest_height,
        ),
        QueryMsg::IbcConnection { connection_id } => query_ibc_connection(deps, connection_id),
        QueryMsg::TokenfactoryParams {} => query_tokenfactory_params(deps),
        QueryMsg::TokenfactoryDenomAuthorityMetadata { denom } => {
            query_tokenfactory_denom_authority_metadata(deps, denom)
        }
        QueryMsg::TokenfactoryDenomsFromCreator { creator } => {
            query_tokenfactory_denoms_from_creator(deps, creator)
        }
        QueryMsg::InterchaintxParams {} => query_interchaintx_params(deps),
        QueryMsg::InterchainqueriesParams {} => query_interchainqueries_params(deps),
        QueryMsg::FeeburnerParams {} => query_feeburner_params(deps),
        QueryMsg::FeeburnerTotalBurnedNeutronsAmount {} => {
            query_feeburner_total_burned_neutrons_amount(deps)
        }
    }
}

fn query_bank_balance(deps: Deps, address: String, denom: String) -> NeutronResult<Binary> {
    let msg = bank::v1beta1::QueryBalanceRequest { address, denom };
    let resp = make_stargate_query(
        deps,
        "/cosmos.bank.v1beta1.Query/Balance".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(to_binary(&resp)?)
}

fn query_bank_denom_metadata(deps: Deps, denom: String) -> NeutronResult<Binary> {
    let msg = bank::v1beta1::QueryDenomMetadataRequest { denom };
    let mut bytes = Vec::new();
    ::prost::Message::encode(&msg, &mut bytes)
        .map_err(|_| StdError::generic_err("cannot encode proto"))?;

    let resp = make_stargate_query(
        deps,
        "/cosmos.bank.v1beta1.Query/DenomMetadata".to_string(),
        bytes,
    )?;

    Ok(to_binary(&resp)?)
}

fn query_bank_params(deps: Deps) -> NeutronResult<Binary> {
    let msg = bank::v1beta1::QueryParamsRequest {};
    let mut bytes = Vec::new();
    ::prost::Message::encode(&msg, &mut bytes)
        .map_err(|_| StdError::generic_err("cannot encode proto"))?;

    let resp = make_stargate_query(deps, "/cosmos.bank.v1beta1.Query/Params".to_string(), bytes)?;
    Ok(to_binary(&resp)?)
}

fn query_bank_supply_of(deps: Deps, denom: String) -> NeutronResult<Binary> {
    let msg = bank::v1beta1::QuerySupplyOfRequest { denom };
    let resp = make_stargate_query(
        deps,
        "/cosmos.bank.v1beta1.Query/SupplyOf".to_string(),
        ::prost::Message::encode_to_vec(&msg),
    )?;

    Ok(to_binary(&resp)?)
}

fn query_auth_account(deps: Deps, address: String) -> NeutronResult<Binary> {
    let msg = auth::v1beta1::QueryAccountRequest { address };
    let resp = make_stargate_query(
        deps,
        "/cosmos.auth.v1beta1.Query/Account".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(to_binary(&resp)?)
}

fn query_transfer_denom_trace(deps: Deps, hash: String) -> NeutronResult<Binary> {
    let msg = ibc::applications::transfer::v1::QueryDenomTraceRequest { hash };
    let resp = make_stargate_query(
        deps,
        "/ibc.applications.transfer.v1.Query/DenomTrace".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(to_binary(&resp)?)
}

fn query_ibc_client_state(deps: Deps, client_id: String) -> NeutronResult<Binary> {
    let msg = ibc::core::client::v1::QueryClientStateRequest { client_id };
    let resp = make_stargate_query(
        deps,
        "/ibc.core.client.v1.Query/ClientState".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(to_binary(&resp)?)
}

fn query_ibc_consensus_state(
    deps: Deps,
    client_id: String,
    revision_number: u64,
    revision_height: u64,
    latest_height: bool,
) -> NeutronResult<Binary> {
    let msg = ibc::core::client::v1::QueryConsensusStateRequest {
        client_id,
        revision_number,
        revision_height,
        latest_height,
    };
    let resp = make_stargate_query(
        deps,
        "/ibc.core.client.v1.Query/ConsensusState".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(to_binary(&resp)?)
}

fn query_ibc_connection(deps: Deps, connection_id: String) -> NeutronResult<Binary> {
    let msg = ibc::core::connection::v1::QueryConnectionRequest { connection_id };
    let resp = make_stargate_query(
        deps,
        "/ibc.core.connection.v1.Query/Connection".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(to_binary(&resp)?)
}

fn query_tokenfactory_params(deps: Deps) -> NeutronResult<Binary> {
    let msg = osmosis_std::types::osmosis::tokenfactory::v1beta1::QueryParamsRequest {};
    let resp = make_stargate_query(
        deps,
        "/osmosis.tokenfactory.v1beta1.Query/Params".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(to_binary(&resp)?)
}

fn query_tokenfactory_denom_authority_metadata(deps: Deps, denom: String) -> NeutronResult<Binary> {
    let msg =
        osmosis_std::types::osmosis::tokenfactory::v1beta1::QueryDenomAuthorityMetadataRequest {
            denom,
        };
    let resp = make_stargate_query(
        deps,
        "/osmosis.tokenfactory.v1beta1.Query/DenomAuthorityMetadata".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(to_binary(&resp)?)
}

fn query_tokenfactory_denoms_from_creator(deps: Deps, creator: String) -> NeutronResult<Binary> {
    let msg = osmosis_std::types::osmosis::tokenfactory::v1beta1::QueryDenomsFromCreatorRequest {
        creator,
    };
    let resp = make_stargate_query(
        deps,
        "/osmosis.tokenfactory.v1beta1.Query/DenomsFromCreator".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(to_binary(&resp)?)
}

fn query_interchaintx_params(deps: Deps) -> NeutronResult<Binary> {
    let msg = stargate::interchaintx::QueryParams {};
    let resp = make_stargate_query(
        deps,
        "/neutron.interchaintxs.Query/Params".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(to_binary(&resp)?)
}

fn query_interchainqueries_params(deps: Deps) -> NeutronResult<Binary> {
    let msg = stargate::interchainqueries::QueryParams {};
    let resp = make_stargate_query(
        deps,
        "/neutron.interchainqueries.Query/Params".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(to_binary(&resp)?)
}

fn query_feeburner_params(deps: Deps) -> NeutronResult<Binary> {
    let msg = stargate::feeburner::QueryParams {};
    let resp = make_stargate_query(
        deps,
        "/neutron.feeburner.Query/Params".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(to_binary(&resp)?)
}

// WARN: should not work since we did not allowlist it
// exists here only for testing purposes
fn query_feeburner_total_burned_neutrons_amount(deps: Deps) -> NeutronResult<Binary> {
    let msg = stargate::feeburner::QueryTotalBurnedNeutronsAmountRequest {};
    let resp = make_stargate_query(
        deps,
        "/neutron.feeburner.Query/TotalBurnedNeutronsAmount".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(to_binary(&resp)?)
}

pub fn make_stargate_query(
    deps: Deps,
    path: String,
    encoded_query_data: Vec<u8>,
) -> StdResult<String> {
    let raw = to_vec::<QueryRequest<Empty>>(&QueryRequest::Stargate {
        path,
        data: encoded_query_data.into(),
    })
    .map_err(|serialize_err| {
        StdError::generic_err(format!("Serializing QueryRequest: {}", serialize_err))
    })?;
    match deps.querier.raw_query(&raw) {
        SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
            "Querier system error: {}",
            system_err
        ))),
        SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(format!(
            "Querier contract error: {}",
            contract_err
        ))),
        // response(value) is base64 encoded bytes
        SystemResult::Ok(ContractResult::Ok(value)) => {
            let str = value.to_base64();
            deps.api
                .debug(format!("WASMDEBUG: make_stargate_query: {:?}", str).as_str());
            from_utf8(value.as_slice())
                .map(|s| s.to_string())
                .map_err(|_e| StdError::generic_err("Unable to encode from utf8"))
        }
    }
}
