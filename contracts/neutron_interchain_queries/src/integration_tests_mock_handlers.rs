use crate::state::{IntegrationTestsQueryMock, INTEGRATION_TESTS_QUERY_MOCK};
use cosmwasm_std::{DepsMut, Response};
use neutron_sdk::NeutronResult;

pub fn set_query_mock(deps: DepsMut) -> NeutronResult<Response> {
    INTEGRATION_TESTS_QUERY_MOCK.save(deps.storage, &IntegrationTestsQueryMock::Enabled)?;
    Ok(Response::default())
}

pub fn unset_query_mock(deps: DepsMut) -> NeutronResult<Response> {
    INTEGRATION_TESTS_QUERY_MOCK.save(deps.storage, &IntegrationTestsQueryMock::Disabled)?;
    Ok(Response::default())
}
