use crate::storage::{
    IntegrationTestsSudoMock, IntegrationTestsSudoSubmsgMock, INTEGRATION_TESTS_SUDO_FAILURE_MOCK,
    INTEGRATION_TESTS_SUDO_SUBMSG_FAILURE_MOCK,
};
use cosmwasm_std::{DepsMut, Response, StdResult};
use neutron_sdk::bindings::msg::NeutronMsg;

pub fn set_sudo_failure_mock(deps: DepsMut) -> StdResult<Response<NeutronMsg>> {
    INTEGRATION_TESTS_SUDO_FAILURE_MOCK.save(deps.storage, &IntegrationTestsSudoMock::Enabled)?;
    Ok(Response::default())
}

pub fn set_sudo_submsg_failure_mock(deps: DepsMut) -> StdResult<Response<NeutronMsg>> {
    INTEGRATION_TESTS_SUDO_SUBMSG_FAILURE_MOCK
        .save(deps.storage, &IntegrationTestsSudoSubmsgMock::Enabled)?;
    Ok(Response::default())
}

pub fn set_sudo_submsg_failure_in_reply_mock(deps: DepsMut) -> StdResult<Response<NeutronMsg>> {
    INTEGRATION_TESTS_SUDO_SUBMSG_FAILURE_MOCK.save(
        deps.storage,
        &IntegrationTestsSudoSubmsgMock::EnabledInReply,
    )?;
    Ok(Response::default())
}

pub fn unset_sudo_failure_mock(deps: DepsMut) -> StdResult<Response<NeutronMsg>> {
    INTEGRATION_TESTS_SUDO_FAILURE_MOCK.save(deps.storage, &IntegrationTestsSudoMock::Disabled)?;
    INTEGRATION_TESTS_SUDO_SUBMSG_FAILURE_MOCK
        .save(deps.storage, &IntegrationTestsSudoSubmsgMock::Disabled)?;
    Ok(Response::default())
}
