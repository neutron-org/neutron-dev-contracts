use crate::msg::{ExecuteMsg, InstantiateMsg};
use cosmos_sdk_proto::cosmos::{auth, bank};
use cosmos_sdk_proto::ibc;
use cosmwasm_std::{
    entry_point, ContractResult, Deps, DepsMut, Empty, Env, MessageInfo, QueryRequest, Response,
    StdError, StdResult, SystemResult,
};
use std::str::from_utf8;

use crate::stargate;
use cw2::set_contract_version;
use neutron_sdk::bindings::msg::NeutronMsg;
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
    deps: DepsMut,
    _: Env,
    _: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response<NeutronMsg>> {
    deps.api
        .debug(format!("WASMDEBUG: execute: received msg: {:?}", msg).as_str());
    match msg {
        ExecuteMsg::QueryBankBalance { address, denom } => {
            execute_query_balance(deps, address, denom)
        }
        ExecuteMsg::QueryBankDenomMetadata { denom } => execute_query_denom_metadata(deps, denom),
        ExecuteMsg::QueryBankParams {} => execute_query_bank_params(deps),
        ExecuteMsg::QueryBankSupplyOf { denom } => execute_query_supply_of(deps, denom),
        ExecuteMsg::QueryAuthAccount { address } => execute_query_account(deps, address),
        ExecuteMsg::QueryTransferDenomTrace { hash } => execute_query_denom_trace(deps, hash),
        ExecuteMsg::QueryIbcClientState { client_id } => {
            execute_query_client_state(deps, client_id)
        }
        ExecuteMsg::QueryIbcConsensusState {
            client_id,
            revision_number,
            revision_height,
            latest_height,
        } => execute_query_consensus_state(
            deps,
            client_id,
            revision_number,
            revision_height,
            latest_height,
        ),
        ExecuteMsg::QueryIbcConnection { connection_id } => {
            execute_query_connection(deps, connection_id)
        }
        ExecuteMsg::TokenfactoryParams {} => execute_query_tokenfactory_params(deps),
        ExecuteMsg::TokenfactoryDenomAuthorityMetadata { denom } => {
            execute_query_tokenfactory_denom_authority_metadata(deps, denom)
        }
        ExecuteMsg::TokenfactoryDenomsFromCreator { creator } => {
            execute_query_tokenfactory_denoms_from_creator(deps, creator)
        }
        ExecuteMsg::ContractmanagerAddressFailures { address } => {
            execute_query_contractmanager_query_address_failures(deps, address)
        }
        ExecuteMsg::ContractmanagerFailures { address } => {
            execute_query_contractmanager_query_failures(deps, address)
        }
        ExecuteMsg::QueryInterchaintxParams {} => execute_query_interchaintx_params(deps),
        ExecuteMsg::QueryInterchainqueriesParams {} => execute_query_interchainqueries_params(deps),
        ExecuteMsg::QueryFeeburnerParams {} => execute_query_feeburner_params(deps),
    }
}

fn execute_query_balance(
    deps: DepsMut,
    address: String,
    denom: String,
) -> StdResult<Response<NeutronMsg>> {
    let msg = bank::v1beta1::QueryBalanceRequest { address, denom };
    let resp = make_stargate_query(
        deps.as_ref(),
        "/cosmos.bank.v1beta1.Query/Balance".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(Response::new().add_attribute("stargate_response", resp))
}

fn execute_query_denom_metadata(deps: DepsMut, denom: String) -> StdResult<Response<NeutronMsg>> {
    let msg = bank::v1beta1::QueryDenomMetadataRequest { denom };
    let mut bytes = Vec::new();
    ::prost::Message::encode(&msg, &mut bytes)
        .map_err(|_| StdError::generic_err("cannot encode proto"))?;

    let resp = make_stargate_query(
        deps.as_ref(),
        "/cosmos.bank.v1beta1.Query/DenomMetadata".to_string(),
        bytes,
    )?;

    Ok(Response::new().add_attribute("stargate_response", resp))
}

fn execute_query_bank_params(deps: DepsMut) -> StdResult<Response<NeutronMsg>> {
    let msg = bank::v1beta1::QueryParamsRequest {};
    let mut bytes = Vec::new();
    ::prost::Message::encode(&msg, &mut bytes)
        .map_err(|_| StdError::generic_err("cannot encode proto"))?;

    let resp = make_stargate_query(
        deps.as_ref(),
        "/cosmos.bank.v1beta1.Query/Params".to_string(),
        bytes,
    )?;
    Ok(Response::new().add_attribute("stargate_response", resp))
}

fn execute_query_supply_of(deps: DepsMut, denom: String) -> StdResult<Response<NeutronMsg>> {
    let msg = bank::v1beta1::QuerySupplyOfRequest { denom };
    let resp = make_stargate_query(
        deps.as_ref(),
        "/cosmos.bank.v1beta1.Query/SupplyOf".to_string(),
        ::prost::Message::encode_to_vec(&msg),
    )?;

    Ok(Response::new().add_attribute("stargate_response", resp))
}

fn execute_query_account(deps: DepsMut, address: String) -> StdResult<Response<NeutronMsg>> {
    let msg = auth::v1beta1::QueryAccountRequest { address };
    let resp = make_stargate_query(
        deps.as_ref(),
        "/cosmos.auth.v1beta1.Query/Account".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(Response::new().add_attribute("stargate_response", resp))
}

fn execute_query_denom_trace(deps: DepsMut, hash: String) -> StdResult<Response<NeutronMsg>> {
    let msg = ibc::applications::transfer::v1::QueryDenomTraceRequest { hash };
    let resp = make_stargate_query(
        deps.as_ref(),
        "/ibc.applications.transfer.v1.Query/DenomTrace".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(Response::new().add_attribute("stargate_response", resp))
}

fn execute_query_client_state(deps: DepsMut, client_id: String) -> StdResult<Response<NeutronMsg>> {
    let msg = ibc::core::client::v1::QueryClientStateRequest { client_id };
    let resp = make_stargate_query(
        deps.as_ref(),
        "/ibc.core.client.v1.Query/ClientState".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(Response::new().add_attribute("stargate_response", resp))
}

fn execute_query_consensus_state(
    deps: DepsMut,
    client_id: String,
    revision_number: u64,
    revision_height: u64,
    latest_height: bool,
) -> StdResult<Response<NeutronMsg>> {
    let msg = ibc::core::client::v1::QueryConsensusStateRequest {
        client_id,
        revision_number,
        revision_height,
        latest_height,
    };
    let resp = make_stargate_query(
        deps.as_ref(),
        "/ibc.core.client.v1.Query/ConsensusState".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(Response::new().add_attribute("stargate_response", resp))
}

fn execute_query_connection(
    deps: DepsMut,
    connection_id: String,
) -> StdResult<Response<NeutronMsg>> {
    let msg = ibc::core::connection::v1::QueryConnectionRequest { connection_id };
    let resp = make_stargate_query(
        deps.as_ref(),
        "/ibc.core.connection.v1.Query/Connection".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(Response::new().add_attribute("stargate_response", resp))
}

fn execute_query_tokenfactory_params(deps: DepsMut) -> StdResult<Response<NeutronMsg>> {
    let msg = osmosis_std::types::osmosis::tokenfactory::v1beta1::QueryParamsRequest {};
    let resp = make_stargate_query(
        deps.as_ref(),
        "/osmosis.tokenfactory.v1beta1.Query/Params".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(Response::new().add_attribute("stargate_response", resp))
}

fn execute_query_tokenfactory_denom_authority_metadata(
    deps: DepsMut,
    denom: String,
) -> StdResult<Response<NeutronMsg>> {
    let msg =
        osmosis_std::types::osmosis::tokenfactory::v1beta1::QueryDenomAuthorityMetadataRequest {
            denom,
        };
    let resp = make_stargate_query(
        deps.as_ref(),
        "/osmosis.tokenfactory.v1beta1.Query/DenomAuthorityMetadata".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(Response::new().add_attribute("stargate_response", resp))
}

fn execute_query_tokenfactory_denoms_from_creator(
    deps: DepsMut,
    creator: String,
) -> StdResult<Response<NeutronMsg>> {
    let msg = osmosis_std::types::osmosis::tokenfactory::v1beta1::QueryDenomsFromCreatorRequest {
        creator,
    };
    let resp = make_stargate_query(
        deps.as_ref(),
        "/osmosis.tokenfactory.v1beta1.Query/DenomsFromCreator".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(Response::new().add_attribute("stargate_response", resp))
}

fn execute_query_contractmanager_query_address_failures(
    deps: DepsMut,
    address: String,
) -> StdResult<Response<NeutronMsg>> {
    let msg = stargate::contractmanager::QueryAddressFailuresRequest { address };
    let resp = make_stargate_query(
        deps.as_ref(),
        "/neutron.contractmanager.Query/AddressFailures".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(Response::new().add_attribute("stargate_response", resp))
}

fn execute_query_contractmanager_query_failures(
    deps: DepsMut,
    address: String,
) -> StdResult<Response<NeutronMsg>> {
    let msg = stargate::contractmanager::QueryFailuresRequest {
        address,
        pagination: None,
    };
    let resp = make_stargate_query(
        deps.as_ref(),
        "/neutron.contractmanager.Query/Failures".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(Response::new().add_attribute("stargate_response", resp))
}

fn execute_query_interchaintx_params(deps: DepsMut) -> StdResult<Response<NeutronMsg>> {
    let msg = stargate::interchaintx::QueryParams {};
    let resp = make_stargate_query(
        deps.as_ref(),
        "/neutron.interchaintxs.Query/Params".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(Response::new().add_attribute("stargate_response", resp))
}

fn execute_query_interchainqueries_params(deps: DepsMut) -> StdResult<Response<NeutronMsg>> {
    let msg = stargate::interchainqueries::QueryParams {};
    let resp = make_stargate_query(
        deps.as_ref(),
        "/neutron.interchainqueries.Query/Params".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(Response::new().add_attribute("stargate_response", resp))
}

fn execute_query_feeburner_params(deps: DepsMut) -> StdResult<Response<NeutronMsg>> {
    let msg = stargate::feeburner::QueryParams {};
    let resp = make_stargate_query(
        deps.as_ref(),
        "/neutron.feeburner.Query/Params".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(Response::new().add_attribute("stargate_response", resp))
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
                .map_err(|_e| StdError::generic_err(format!("Unable to encode from utf8")))
        }
    }
}
