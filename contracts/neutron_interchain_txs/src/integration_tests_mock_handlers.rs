use crate::storage::{
    IntegrationTestsSudoFailureMock, IntegrationTestsSudoSubmsgFailureMock,
    INTEGRATION_TESTS_SUDO_FAILURE_MOCK, INTEGRATION_TESTS_SUDO_SUBMSG_FAILURE_MOCK,
};
use cosmwasm_std::{DepsMut, Response, StdResult};

pub fn set_sudo_failure_mock(
    deps: DepsMut,
    state: IntegrationTestsSudoFailureMock,
) -> StdResult<Response> {
    INTEGRATION_TESTS_SUDO_FAILURE_MOCK.save(deps.storage, &state)?;
    Ok(Response::default())
}

pub fn set_sudo_submsg_failure_mock(deps: DepsMut) -> StdResult<Response> {
    INTEGRATION_TESTS_SUDO_SUBMSG_FAILURE_MOCK.save(
        deps.storage,
        &IntegrationTestsSudoSubmsgFailureMock::Enabled,
    )?;
    Ok(Response::default())
}

pub fn set_sudo_submsg_failure_in_reply_mock(deps: DepsMut) -> StdResult<Response> {
    INTEGRATION_TESTS_SUDO_SUBMSG_FAILURE_MOCK.save(
        deps.storage,
        &IntegrationTestsSudoSubmsgFailureMock::EnabledInReply,
    )?;
    Ok(Response::default())
}

pub fn unset_sudo_failure_mock(deps: DepsMut) -> StdResult<Response> {
    INTEGRATION_TESTS_SUDO_FAILURE_MOCK
        .save(deps.storage, &IntegrationTestsSudoFailureMock::Disabled)?;
    INTEGRATION_TESTS_SUDO_SUBMSG_FAILURE_MOCK.save(
        deps.storage,
        &IntegrationTestsSudoSubmsgFailureMock::Disabled,
    )?;
    Ok(Response::default())
}
