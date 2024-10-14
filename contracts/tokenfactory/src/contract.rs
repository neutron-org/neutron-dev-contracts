use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cosmwasm_std::{coins, entry_point, to_json_binary, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};
use neutron_sdk::NeutronError::Std;
use neutron_sdk::NeutronResult;
use neutron_std::types::cosmos::bank::v1beta1::Metadata;
use neutron_std::types::osmosis::tokenfactory::v1beta1::{MsgChangeAdmin, MsgCreateDenom, MsgMint, TokenfactoryQuerier, MsgBurn, MsgSetBeforeSendHook, MsgForceTransfer, MsgSetDenomMetadata};
use neutron_std::types::cosmos::base::v1beta1::Coin as CosmosCoin;

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    _deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let msg: CosmosMsg = match msg {
        ExecuteMsg::CreateDenom { subdenom } => {
            MsgCreateDenom{
                sender: env.contract.address.to_string(),
                subdenom,
            }.into()
        },
        ExecuteMsg::ChangeAdmin {
            denom,
            new_admin_address,
        } => {
            MsgChangeAdmin{
                sender: env.contract.address.to_string(),
                denom,
                new_admin: new_admin_address
            }.into()
        },
        ExecuteMsg::MintTokens {
            denom,
            amount,
            mint_to_address,
        } => {
            MsgMint{
                sender: info.sender.to_string(), // FIXME
                amount: Some(CosmosCoin{ denom, amount: amount.to_string() }),
                mint_to_address: mint_to_address.unwrap_or(env.contract.address.into()),
            }
            .into()
        },
        ExecuteMsg::BurnTokens {
            denom,
            amount,
            burn_from_address,
        } => {
            MsgBurn{
                sender: info.sender.to_string(),
                amount: Some(CosmosCoin{ denom, amount: amount.to_string() }),
                burn_from_address: burn_from_address.unwrap_or(env.contract.address.into()),
            }.into()
        },
        ExecuteMsg::SetBeforeSendHook {
            denom,
            contract_addr,
        } => {
            MsgSetBeforeSendHook{
                sender: info.sender.to_string(),
                denom,
                contract_addr
            }.into()
        },
        ExecuteMsg::SendTokens {
            recipient,
            denom,
            amount,
        } => BankMsg::Send {
            to_address: recipient,
            amount: coins(amount.u128(), denom),
        }
        .into(),
        ExecuteMsg::ForceTransfer {
            denom,
            amount,
            from,
            to,
        } => {
            MsgForceTransfer{
                sender: info.sender.to_string(),
                amount: Some(CosmosCoin{ denom, amount: amount.to_string() }),
                transfer_from_address: from,
                transfer_to_address: to,
            }.into()
        },
        ExecuteMsg::SetDenomMetadata {
            description,
            denom_units,
            base,
            display,
            name,
            symbol,
            uri,
            uri_hash,
        } => {
            MsgSetDenomMetadata{
                sender: info.sender.to_string(),
                metadata: Some(
                    Metadata {
                        description,
                        denom_units,
                        base,
                        display,
                        name,
                        symbol,
                        uri,
                        uri_hash,
                    }
                )
            }.into()
        },
    };
    Ok(Response::new().add_message(msg))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> NeutronResult<Binary> {
    let querier = TokenfactoryQuerier::new(&deps.querier);
    Ok(match msg {
        QueryMsg::FullDenom {
            creator_addr,
            subdenom,
        } => {
            // TODO
            to_json_binary(&query_full_denom(deps, creator_addr, subdenom)?)?
        },
        QueryMsg::DenomAdmin { creator, subdenom } => {
            let authority = querier.denom_authority_metadata(creator, subdenom)?;
            to_json_binary(&authority.authority_metadata.ok_or(Std(StdError::generic_err("authority metadata not found")))?.admin?)?
        },
        QueryMsg::BeforeSendHook { creator, denom } => {
            to_json_binary(&querier.before_send_hook_address(creator, denom)?)?
        }
    })
}

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::new())
}
