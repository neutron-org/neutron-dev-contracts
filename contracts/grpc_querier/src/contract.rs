use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, ContractResult, Deps, DepsMut, Empty, Env, MessageInfo,
    QueryRequest, Response, StdError, StdResult, SystemResult,
};
use std::str::from_utf8;

use crate::grpc;
use cw2::set_contract_version;
use neutron_std::types::{
    cosmos::{auth, bank},
    ibc::{
        applications::transfer,
        core::{client, connection},
    },
    neutron::{feeburner, interchainqueries, interchaintxs},
    osmosis::tokenfactory,
};
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
pub fn execute(_: DepsMut, _: Env, _: MessageInfo, _msg: ExecuteMsg) -> StdResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _: Env, msg: QueryMsg) -> StdResult<Binary> {
    deps.api
        .debug(format!("WASMDEBUG: query: received msg: {:?}", msg).as_str());

    let bank_querier = bank::v1beta1::BankQuerier::new(&deps.querier);
    let auth_querier = auth::v1beta1::AuthQuerier::new(&deps.querier);
    let transfer_querier = transfer::v1::TransferQuerier::new(&deps.querier);
    let client_querier = client::v1::ClientQuerier::new(&deps.querier);
    let connection_querier = connection::v1::ConnectionQuerier::new(&deps.querier);
    let tokenfactory_querier = tokenfactory::v1beta1::TokenfactoryQuerier::new(&deps.querier);
    let interchaintxs_querier = interchaintxs::v1::InterchaintxsQuerier::new(&deps.querier);
    let interchainqueries_querier = interchainqueries::InterchainqueriesQuerier::new(&deps.querier);
    let feeburner_querier = feeburner::FeeburnerQuerier::new(&deps.querier);

    match msg {
        QueryMsg::BankBalance { address, denom } => {
            Ok(to_json_binary(&bank_querier.balance(address, denom)?)?)
        }

        QueryMsg::BankDenomMetadata { denom } => {
            Ok(to_json_binary(&bank_querier.denom_metadata(denom)?)?)
        }

        QueryMsg::BankParams {} => Ok(to_json_binary(&bank_querier.params()?)?),

        QueryMsg::BankSupplyOf { denom } => Ok(to_json_binary(&bank_querier.supply_of(denom)?)?),

        QueryMsg::AuthAccount { address } => Ok(to_json_binary(&auth_querier.account(address)?)?),

        QueryMsg::TransferDenomTrace { hash } => {
            Ok(to_json_binary(&transfer_querier.denom_trace(hash)?)?)
        }

        QueryMsg::IbcClientState { client_id } => {
            Ok(to_json_binary(&client_querier.client_state(client_id)?)?)
        }

        QueryMsg::IbcConsensusState {
            client_id,
            revision_number,
            revision_height,
            latest_height,
        } => Ok(to_json_binary(&client_querier.consensus_state(
            client_id,
            revision_number,
            revision_height,
            latest_height,
        )?)?),

        QueryMsg::IbcConnection { connection_id } => Ok(to_json_binary(
            &connection_querier.connection(connection_id)?,
        )?),

        QueryMsg::TokenfactoryParams {} => Ok(to_json_binary(&tokenfactory_querier.params()?)?),

        QueryMsg::TokenfactoryDenomAuthorityMetadata { creator, subdenom } => Ok(to_json_binary(
            &tokenfactory_querier.denom_authority_metadata(creator, subdenom)?,
        )?),

        QueryMsg::TokenfactoryDenomsFromCreator { creator } => Ok(to_json_binary(
            &tokenfactory_querier.denoms_from_creator(creator)?,
        )?),

        QueryMsg::ContractmanagerAddressFailures { address } => {
            query_contractmanager_query_address_failures(deps, address)
        }

        QueryMsg::ContractmanagerFailures { address } => {
            query_contractmanager_query_failures(deps, address)
        }

        QueryMsg::InterchaintxsParams {} => Ok(to_json_binary(&interchaintxs_querier.params()?)?),

        QueryMsg::InterchainqueriesParams {} => {
            Ok(to_json_binary(&interchainqueries_querier.params()?)?)
        }

        QueryMsg::FeeburnerParams {} => Ok(to_json_binary(&feeburner_querier.params()?)?),

        QueryMsg::FeeburnerTotalBurnedNeutronsAmount {} => {
            // WARN: should not work since we did not allowlist it
            // exists here only for testing purposes
            Ok(to_json_binary(
                &feeburner_querier.total_burned_neutrons_amount()?,
            )?)
        }
    }
}

// Can be refactored after https://hadronlabs.atlassian.net/browse/NTRN-359 is done
fn query_contractmanager_query_address_failures(deps: Deps, address: String) -> StdResult<Binary> {
    let msg = grpc::contractmanager::QueryAddressFailuresRequest { address };
    let mut bytes = Vec::new();
    Message::encode(&msg, &mut bytes).map_err(|_| StdError::generic_err("cannot encode proto"))?;

    let resp = make_stargate_query(
        deps,
        "/neutron.contractmanager.Query/AddressFailures".to_string(),
        bytes,
    )?;

    to_json_binary(&resp)
}

// Can be refactored after https://hadronlabs.atlassian.net/browse/NTRN-359 is done
fn query_contractmanager_query_failures(deps: Deps, address: String) -> StdResult<Binary> {
    let msg = grpc::contractmanager::QueryFailuresRequest {
        address,
        pagination: None,
    };
    let mut bytes = Vec::new();
    Message::encode(&msg, &mut bytes).map_err(|_| StdError::generic_err("cannot encode proto"))?;

    let resp = make_stargate_query(
        deps,
        "/neutron.contractmanager.Query/Failures".to_string(),
        bytes,
    )?;

    to_json_binary(&resp)
}

// Can be removed after https://hadronlabs.atlassian.net/browse/NTRN-359 is done
pub fn make_stargate_query(
    deps: Deps,
    path: String,
    encoded_query_data: Vec<u8>,
) -> StdResult<String> {
    #[allow(deprecated)]
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
