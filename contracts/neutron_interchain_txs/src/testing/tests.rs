// Copyright 2022 Neutron Licensed under the Apache License, Version 2.0 (the "License");
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

use crate::contract::sudo;
use crate::msg::ExecuteMsg;
use crate::storage::IntegrationTestsSudoFailureMock;
use crate::{
    contract::{execute, query_errors_queue},
    storage::{add_error_to_queue, read_errors_from_queue, ERRORS_QUEUE},
};
use cosmwasm_std::testing::{
    message_info, mock_dependencies as cw_mock_dependencies, mock_env, MockApi, MockQuerier,
    MockStorage,
};
use cosmwasm_std::{from_json, Addr, OwnedDeps, StdError};
use neutron_sdk::bindings::query::NeutronQuery;
use neutron_sdk::sudo::msg::{RequestPacket, SudoMsg};
use std::marker::PhantomData;

pub fn mock_dependencies() -> OwnedDeps<MockStorage, MockApi, MockQuerier, NeutronQuery> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: MockQuerier::default(),
        custom_query_type: PhantomData,
    }
}

#[test]
fn test_query_errors_queue() {
    let mut deps = mock_dependencies();

    let result = query_errors_queue(deps.as_ref()).unwrap();
    let result: Vec<(Vec<u8>, String)> = from_json(result).unwrap();

    assert_eq!(0, result.len());

    let error_msg = "Error message".to_string();

    ERRORS_QUEUE
        .save(&mut deps.storage, 0u32, &error_msg)
        .unwrap();

    let result = query_errors_queue(deps.as_ref()).unwrap();
    let result: Vec<(Vec<u8>, String)> = from_json(result).unwrap();

    assert_eq!(1, result.len());
}

#[test]
fn test_errors_queue() {
    let mut store = MockStorage::new();

    let errors = read_errors_from_queue(&store);
    let errors = errors.unwrap();

    assert_eq!(0, errors.len());

    let error = "some error message".to_string();

    add_error_to_queue(&mut store, error.clone()).unwrap();

    let errors = read_errors_from_queue(&store);
    let errors = errors.unwrap();

    assert_eq!(1, errors.len());
    assert_eq!(errors, vec![(0u32.to_be_bytes().to_vec(), error.clone())]);

    add_error_to_queue(&mut store, error.clone()).unwrap();
    add_error_to_queue(&mut store, error.clone()).unwrap();

    let errors = read_errors_from_queue(&store);
    let errors = errors.unwrap();

    assert_eq!(3, errors.len());
    assert_eq!(
        errors,
        vec![
            (0u32.to_be_bytes().to_vec(), error.clone()),
            (1u32.to_be_bytes().to_vec(), error.clone()),
            (2u32.to_be_bytes().to_vec(), error)
        ]
    );
}

#[test]
fn test_failure_mocks() {
    let mut deps = cw_mock_dependencies();
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked(""), &[]),
        ExecuteMsg::IntegrationTestsSetSudoFailureMock {
            state: IntegrationTestsSudoFailureMock::Enabled,
        },
    )
    .unwrap();

    let src_port = String::from("src_port");
    let src_channel = String::from("src_channel");
    let dst_port = String::from("dst_port");
    let dst_channel = String::from("dst_channel");
    let sudo_resp = SudoMsg::Timeout {
        request: RequestPacket {
            sequence: Some(1u64),
            source_port: Some(src_port),
            source_channel: Some(src_channel),
            destination_port: Some(dst_port),
            destination_channel: Some(dst_channel),
            data: None,
            timeout_height: None,
            timeout_timestamp: None,
        },
    };

    let err = sudo(deps.as_mut(), mock_env(), sudo_resp).unwrap_err();
    assert_eq!(err, StdError::generic_err("Integrations test mock error"));
}
