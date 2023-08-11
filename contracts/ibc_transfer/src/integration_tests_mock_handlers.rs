use crate::state::{IntegrationTestsSudoFailureMock, INTEGRATION_TESTS_SUDO_FAILURE_MOCK};
use cosmwasm_std::{DepsMut, Response, StdResult};
use neutron_sdk::bindings::msg::NeutronMsg;

pub fn set_sudo_failure_mock(
    deps: DepsMut,
    state: IntegrationTestsSudoFailureMock,
) -> StdResult<Response<NeutronMsg>> {
    INTEGRATION_TESTS_SUDO_FAILURE_MOCK.save(deps.storage, &state)?;
    Ok(Response::default())
}

pub fn unset_sudo_failure_mock(deps: DepsMut) -> StdResult<Response<NeutronMsg>> {
    INTEGRATION_TESTS_SUDO_FAILURE_MOCK
        .save(deps.storage, &IntegrationTestsSudoFailureMock::Disabled)?;
    Ok(Response::default())
}
