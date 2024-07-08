// Copyright 2022 Neutron
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;
use std::marker::PhantomData;

use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    from_json, Binary, Coin, ContractResult, CustomQuery, FullDelegation, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SystemError, SystemResult, Uint128, Validator,
};
use neutron_sdk::bindings::query::NeutronQuery;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MOCK_CONTRACT_ADDR: &str = "cosmos2contract";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CustomQueryWrapper {}

// implement custom query
impl CustomQuery for CustomQueryWrapper {}

pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier, NeutronQuery> {
    let contract_addr = MOCK_CONTRACT_ADDR;
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(contract_addr, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<NeutronQuery>,
    query_responses: HashMap<u64, Binary>,
    registered_queries: HashMap<u64, Binary>,
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<NeutronQuery> = match from_json(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return QuerierResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                });
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<NeutronQuery>) -> QuerierResult {
        match &request {
            QueryRequest::Custom(NeutronQuery::InterchainQueryResult { query_id }) => {
                SystemResult::Ok(ContractResult::Ok(
                    (*self.query_responses.get(query_id).unwrap()).clone(),
                ))
            }
            QueryRequest::Custom(NeutronQuery::RegisteredInterchainQuery { query_id }) => {
                SystemResult::Ok(ContractResult::Ok(
                    (*self.registered_queries.get(query_id).unwrap()).clone(),
                ))
            }
            QueryRequest::Custom(NeutronQuery::RegisteredInterchainQueries {
                owners: _owners,
                connection_id: _connection_id,
                pagination: _pagination,
            }) => {
                todo!()
            }
            QueryRequest::Custom(NeutronQuery::InterchainAccountAddress { .. }) => {
                todo!()
            }
            _ => self.base.handle_query(request),
        }
    }

    pub fn _update_staking(
        &mut self,
        denom: &str,
        validators: &[Validator],
        delegations: &[FullDelegation],
    ) {
        self.base.staking.update(denom, validators, delegations);
    }

    pub fn add_query_response(&mut self, query_id: u64, response: Binary) {
        self.query_responses.insert(query_id, response);
    }
    pub fn add_registered_queries(&mut self, query_id: u64, response: Binary) {
        self.registered_queries.insert(query_id, response);
    }
}

#[derive(Clone, Default)]
pub struct BalanceQuerier {
    _balances: HashMap<String, Coin>,
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    _balances: HashMap<String, HashMap<String, Uint128>>,
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<NeutronQuery>) -> Self {
        WasmMockQuerier {
            base,
            query_responses: HashMap::new(),
            registered_queries: HashMap::new(),
        }
    }
}
