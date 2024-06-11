use cosmwasm_std::{entry_point, Addr, DepsMut, Env, MessageInfo, Response, StdError, StdResult};
use cw2::set_contract_version;
use cw_storage_plus::Item;
#[allow(deprecated)]
use neutron_sdk::bindings::msg::{ClientUpdateProposal, NeutronMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {}

const CONTRACT_NAME: &str = concat!("crates.io:neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const OWNER: Item<Addr> = Item::new("owner");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SubmitClientUpdateProposal {
        title: String,
        description: String,
        subject_client_id: String,
        substitute_client_id: String,
    },
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    let owner = info.sender;
    OWNER.save(deps.storage, &owner)?;
    deps.api.debug("WASMDEBUG: instantiate");
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response<NeutronMsg>> {
    deps.api
        .debug(format!("WASMDEBUG: execute: received msg: {:?}", msg).as_str());

    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let mut resp = Response::default();
    match msg {
        ExecuteMsg::SubmitClientUpdateProposal {
            title,
            description,
            subject_client_id,
            substitute_client_id,
        } => {
            #[allow(deprecated)]
            let update_proposal_msg =
                NeutronMsg::submit_client_update_proposal(ClientUpdateProposal {
                    title,
                    description,
                    subject_client_id,
                    substitute_client_id,
                });

            resp = resp.add_message(update_proposal_msg);
        }
    }
    Ok(resp)
}
