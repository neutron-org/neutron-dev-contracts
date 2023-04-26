use crate::state::{IntegrationTestsQueryMock, INTEGRATION_TESTS_QUERY_MOCK};
use cosmwasm_std::{DepsMut, Response};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    NeutronResult,
};

pub fn set_query_mock(deps: DepsMut<NeutronQuery>) -> NeutronResult<Response<NeutronMsg>> {
    INTEGRATION_TESTS_QUERY_MOCK.save(deps.storage, &IntegrationTestsQueryMock::Enabled)?;
    Ok(Response::default())
}

pub fn unset_query_mock(deps: DepsMut<NeutronQuery>) -> NeutronResult<Response<NeutronMsg>> {
    INTEGRATION_TESTS_QUERY_MOCK.save(deps.storage, &IntegrationTestsQueryMock::Disabled)?;
    Ok(Response::default())
}
