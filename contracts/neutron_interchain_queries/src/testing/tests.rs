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

use std::str::FromStr;

use super::mock_querier::mock_dependencies as dependencies;
use crate::contract::{execute, query, sudo_tx_query_result};
use crate::msg::{ExecuteMsg, QueryMsg};
use crate::state::{Transfer, RECIPIENT_TXS};
use crate::testing::mock_querier::WasmMockQuerier;
use base64::prelude::*;
use cosmos_sdk_proto::cosmos::base::v1beta1::{Coin as CosmosCoin, DecCoin as CosmosDecCoin};
use cosmos_sdk_proto::cosmos::distribution::v1beta1::FeePool as CosmosFeePool;
use cosmos_sdk_proto::cosmos::gov::v1beta1::{
    Proposal as CosmosProposal, TallyResult as CosmosTallyResult, Vote,
    WeightedVoteOption as CosmosWeightedVoteOption,
};
use cosmos_sdk_proto::cosmos::slashing::v1beta1::ValidatorSigningInfo as CosmosValidatorSigningInfo;
use cosmos_sdk_proto::cosmos::staking::v1beta1::Validator as CosmosValidator;
use cosmos_sdk_proto::Any;
use cosmwasm_std::testing::{message_info, mock_env, MockApi, MockStorage};
use cosmwasm_std::{
    from_json, Addr, Binary, Coin, Decimal, Env, MessageInfo, OwnedDeps, StdError, Uint128,
};
use neutron_sdk::interchain_queries::helpers::{
    decode_and_convert, decode_hex, kv_key_from_string,
};
use neutron_sdk::interchain_queries::types::{
    QueryType, TransactionFilterItem, TransactionFilterOp, TransactionFilterValue,
};
use neutron_sdk::interchain_queries::v047::helpers::{
    create_account_denom_balance_key, create_fee_pool_key, create_gov_proposal_key,
    create_gov_proposal_votes_key, create_total_denom_key, create_validator_key,
    create_validator_signing_info_key,
};
use neutron_sdk::interchain_queries::v047::queries::{
    BalanceResponse, DelegatorDelegationsResponse, FeePoolResponse, ProposalResponse,
    ProposalVotesResponse, TotalSupplyResponse, ValidatorResponse, ValidatorSigningInfoResponse,
};
use neutron_sdk::interchain_queries::v047::types::{
    Balances, FeePool, GovernmentProposal, GovernmentProposalVotes, Proposal, ProposalVote,
    SigningInfo, StakingValidator, StdDelegation, TallyResult, TotalSupply, Validator,
    ValidatorSigningInfo, WeightedVoteOption, DECIMAL_PLACES, RECIPIENT_FIELD,
};
use neutron_sdk::sudo::msg::Height as QueriesHeight;
use neutron_sdk::NeutronError;
use neutron_std::types::cosmos::base::v1beta1::Coin as StdCoin;
use neutron_std::types::ibc::core::client::v1::Height;
use neutron_std::types::neutron::interchainqueries::{
    KvKey, QueryRegisteredQueryResponse, QueryRegisteredQueryResultResponse, QueryResult,
    RegisteredQuery, StorageValue,
};
use prost::Message as ProstMessage;
use schemars::_serde_json::to_string;
use std::ops::Mul;

enum QueryParam {
    Keys(Vec<KvKey>),
    TransactionsFilter(String),
}

fn build_registered_query_response(
    id: u64,
    param: QueryParam,
    query_type: QueryType,
    last_submitted_result_local_height: u64,
) -> Binary {
    let mut registered_keys = vec![];
    let mut transactions_filter = "".to_string();
    match param {
        QueryParam::Keys(keys) => registered_keys = keys,
        QueryParam::TransactionsFilter(filter) => transactions_filter = filter,
    }

    let resp = QueryRegisteredQueryResponse {
        registered_query: Some(RegisteredQuery {
            id,
            owner: "".to_string(),
            keys: registered_keys,
            query_type: query_type.into(),
            transactions_filter,
            connection_id: "".to_string(),
            update_period: 0,
            last_submitted_result_local_height,
            last_submitted_result_remote_height: Some(Height {
                revision_number: 0,
                revision_height: 0,
            }),
            deposit: Vec::from([StdCoin {
                denom: "stake".to_string(),
                amount: "100".to_string(),
            }]),
            submit_timeout: 0,
            registered_at_height: 0,
        }),
    };
    let res = ::prost::Message::encode_to_vec(&resp);
    Binary::from(res.as_slice())
}

fn build_interchain_query_bank_total_denom_value(denom: String, amount: String) -> StorageValue {
    let bank_total_key = create_total_denom_key(denom).unwrap();

    let amount = amount.as_bytes().to_vec();

    StorageValue {
        storage_prefix: "".to_string(),
        key: bank_total_key,
        value: amount,
        proof: None,
    }
}

fn build_interchain_query_distribution_fee_pool_response(denom: String, amount: String) -> Binary {
    let fee_pool_key = create_fee_pool_key().unwrap();

    let adjust: Uint128 = Uint128::one().mul(Uint128::from(10u64).pow(DECIMAL_PLACES));
    let amount = Uint128::from_str(&amount)
        .unwrap()
        .mul(adjust) // adjust to Dec gogo proto format
        .to_string();

    let community_pool_amount = CosmosDecCoin { denom, amount };

    let fee_pool = CosmosFeePool {
        community_pool: vec![community_pool_amount],
    };

    let s = StorageValue {
        storage_prefix: "".to_string(),
        key: fee_pool_key,
        value: fee_pool.encode_to_vec(),
        proof: None,
    };

    let res = ::prost::Message::encode_to_vec(&QueryRegisteredQueryResultResponse {
        result: Some(QueryResult {
            kv_results: vec![s],
            block: None,
            height: 123456,
            revision: 1,
            allow_kv_callbacks: false,
        }),
    });
    Binary::from(res.as_slice())
}

fn build_interchain_query_staking_validator_value(validator: String) -> StorageValue {
    let operator_address = decode_and_convert(validator.as_str()).unwrap();
    let validator_key = create_validator_key(operator_address).unwrap();

    let validator = CosmosValidator {
        operator_address: validator,
        consensus_pubkey: Some(Any {
            type_url: "".to_string(),
            value: vec![],
        }),
        status: 1,
        tokens: "1".to_string(),
        jailed: false,
        delegator_shares: "1".to_string(),
        description: None,
        unbonding_height: 0,
        unbonding_time: None,
        commission: None,
        min_self_delegation: "1".to_string(),
    };

    StorageValue {
        storage_prefix: "".to_string(),
        key: validator_key,
        value: validator.encode_to_vec(),
        proof: None,
    }
}

fn build_interchain_query_validator_signing_info_value(validator: String) -> StorageValue {
    let operator_address = decode_and_convert(validator.as_str()).unwrap();
    let validator_key = create_validator_signing_info_key(operator_address).unwrap();

    let validator = CosmosValidatorSigningInfo {
        address: validator,
        start_height: 1,
        index_offset: 1,
        jailed_until: None,
        tombstoned: false,
        missed_blocks_counter: 987675,
    };

    StorageValue {
        storage_prefix: "".to_string(),
        key: validator_key,
        value: validator.encode_to_vec(),
        proof: None,
    }
}

fn build_interchain_query_gov_proposal_value(proposal_id: u64) -> StorageValue {
    let proposal_key = create_gov_proposal_key(proposal_id).unwrap();

    let proposal = CosmosProposal {
        proposal_id,
        content: Some(Any {
            type_url: "/cosmos.gov.v1beta1.TextProposal".to_string(),
            value: vec![],
        }),
        status: 1,
        final_tally_result: Some(CosmosTallyResult {
            abstain: "0".to_string(),
            yes: "0".to_string(),
            no: "0".to_string(),
            no_with_veto: "0".to_string(),
        }),
        deposit_end_time: None,
        submit_time: None,
        total_deposit: Vec::from([CosmosCoin {
            denom: "stake".to_string(),
            amount: "100".to_string(),
        }]),
        voting_start_time: None,
        voting_end_time: None,
    };

    StorageValue {
        storage_prefix: "".to_string(),
        key: proposal_key,
        value: proposal.encode_to_vec(),
        proof: None,
    }
}

#[allow(deprecated)]
fn build_interchain_query_gov_proposal_votes_value(proposal_id: u64) -> StorageValue {
    let votes_key = create_gov_proposal_votes_key(proposal_id).unwrap();

    let vote = Vote {
        proposal_id,
        voter: "cosmos1yz54ncxj9csp7un3xled03q6thrrhy9cztkfzs".to_string(),
        option: 0,
        options: vec![CosmosWeightedVoteOption {
            weight: "1000000000000000000".to_string(),
            option: 1,
        }],
    };

    StorageValue {
        storage_prefix: "".to_string(),
        key: votes_key,
        value: vote.encode_to_vec(),
        proof: None,
    }
}

fn build_interchain_query_balances_response(addr: Addr, balances: Vec<Coin>) -> Binary {
    let converted_addr_bytes = decode_and_convert(addr.as_str()).unwrap();

    let s: Vec<StorageValue> = balances
        .iter()
        .map(|c| {
            let balance_key =
                create_account_denom_balance_key(converted_addr_bytes.clone(), c.denom.clone())
                    .unwrap();
            StorageValue {
                storage_prefix: "".to_string(),
                key: balance_key,
                value: c.amount.to_string().into_bytes(),
                proof: None,
            }
        })
        .collect();

    let resp = QueryRegisteredQueryResultResponse {
        result: Some(QueryResult {
            kv_results: s,
            block: None,
            height: 123456,
            revision: 2,
            allow_kv_callbacks: false,
        }),
    };
    let res = ::prost::Message::encode_to_vec(&resp);
    Binary::from(res.as_slice())
}

// registers an interchain query
fn register_query(
    deps: &mut OwnedDeps<MockStorage, MockApi, WasmMockQuerier>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Vec<KvKey> {
    let reg_msgs = execute(deps.as_mut(), env, info, msg).unwrap();
    for attr in reg_msgs.attributes {
        if attr.key == "kv_keys" && !attr.value.is_empty() {
            return vec![kv_key_from_string(attr.value).unwrap()];
        }
    }

    vec![]
}

#[test]
fn test_query_balance() {
    let mut deps = dependencies(&[]);

    let msg = ExecuteMsg::RegisterBalancesQuery {
        connection_id: "connection".to_string(),
        update_period: 10,
        addr: "osmo1yz54ncxj9csp7un3xled03q6thrrhy9cztkfzs".to_string(),
        denoms: vec!["uosmo".to_string()],
    };

    let keys = register_query(
        &mut deps,
        mock_env(),
        message_info(&Addr::unchecked(""), &[]),
        msg,
    );

    let registered_query =
        build_registered_query_response(1, QueryParam::Keys(keys), QueryType::KV, 987);

    deps.querier.add_registered_queries(1, registered_query);
    deps.querier.add_query_response(
        1,
        build_interchain_query_balances_response(
            Addr::unchecked("osmo1yz54ncxj9csp7un3xled03q6thrrhy9cztkfzs"),
            vec![Coin::new(8278104u128, "uosmo")],
        ),
    );
    let query_balance = QueryMsg::Balance { query_id: 1 };
    let resp: BalanceResponse =
        from_json(query(deps.as_ref(), mock_env(), query_balance).unwrap()).unwrap();
    assert_eq!(
        resp,
        BalanceResponse {
            last_submitted_local_height: 987,
            balances: Balances {
                coins: vec![Coin::new(8278104u128, "uosmo")]
            },
        }
    )
}

#[test]
fn test_query_balances() {
    let mut deps = dependencies(&[]);

    let msg = ExecuteMsg::RegisterBalancesQuery {
        connection_id: "connection".to_string(),
        update_period: 10,
        addr: "osmo1yz54ncxj9csp7un3xled03q6thrrhy9cztkfzs".to_string(),
        denoms: vec!["uosmo".to_string(), "uatom".to_string()],
    };

    let keys = register_query(
        &mut deps,
        mock_env(),
        message_info(&Addr::unchecked(""), &[]),
        msg,
    );

    let registered_query =
        build_registered_query_response(1, QueryParam::Keys(keys), QueryType::KV, 987);

    deps.querier.add_registered_queries(1, registered_query);
    deps.querier.add_query_response(
        1,
        build_interchain_query_balances_response(
            Addr::unchecked("osmo1yz54ncxj9csp7un3xled03q6thrrhy9cztkfzs"),
            vec![
                Coin::new(8278104u128, "uosmo"),
                Coin::new(1234567u128, "uatom"),
            ],
        ),
    );
    let query_balance = QueryMsg::Balance { query_id: 1 };
    let res = query(deps.as_ref(), mock_env(), query_balance);
    let resp: BalanceResponse = from_json(res.unwrap()).unwrap();
    assert_eq!(
        resp,
        BalanceResponse {
            last_submitted_local_height: 987,
            balances: Balances {
                coins: vec![
                    Coin::new(8278104u128, "uosmo"),
                    Coin::new(1234567u128, "uatom")
                ]
            },
        }
    )
}

#[test]
fn test_bank_total_supply_query() {
    let mut deps = dependencies(&[]);

    let denoms = vec!["uosmo".to_string(), "uatom".to_string()];

    let msg = ExecuteMsg::RegisterBankTotalSupplyQuery {
        connection_id: "connection".to_string(),
        update_period: 10,
        denoms: denoms.clone(),
    };

    let keys = register_query(
        &mut deps,
        mock_env(),
        message_info(&Addr::unchecked(""), &[]),
        msg,
    );

    let registered_query =
        build_registered_query_response(1, QueryParam::Keys(keys), QueryType::KV, 987);

    let mut kv_results: Vec<StorageValue> = vec![];

    for denom in denoms {
        let value =
            build_interchain_query_bank_total_denom_value(denom.to_string(), "8278104".to_string());
        kv_results.push(value);
    }

    let total_supply_response = QueryRegisteredQueryResultResponse {
        result: Some(QueryResult {
            kv_results,
            block: None,
            height: 0,
            revision: 0,
            allow_kv_callbacks: false,
        }),
    };

    deps.querier.add_registered_queries(1, registered_query);
    deps.querier.add_query_response(
        1,
        Binary::from(::prost::Message::encode_to_vec(&total_supply_response).as_slice()),
    );
    let bank_total_balance = QueryMsg::BankTotalSupply { query_id: 1 };

    let resp: TotalSupplyResponse =
        from_json(query(deps.as_ref(), mock_env(), bank_total_balance).unwrap()).unwrap();
    assert_eq!(
        resp,
        TotalSupplyResponse {
            last_submitted_local_height: 987,
            supply: TotalSupply {
                coins: vec![
                    Coin::new(8278104u128, "uosmo"),
                    Coin::new(8278104u128, "uatom"),
                ]
            },
        }
    );
}

#[test]
fn test_distribution_fee_pool_query() {
    let mut deps = dependencies(&[]);

    let msg = ExecuteMsg::RegisterDistributionFeePoolQuery {
        connection_id: "connection".to_string(),
        update_period: 10,
    };

    let keys = register_query(
        &mut deps,
        mock_env(),
        message_info(&Addr::unchecked(""), &[]),
        msg,
    );

    let registered_query =
        build_registered_query_response(1, QueryParam::Keys(keys), QueryType::KV, 987);

    deps.querier.add_registered_queries(1, registered_query);
    deps.querier.add_query_response(
        1,
        build_interchain_query_distribution_fee_pool_response(
            "uosmo".to_string(),
            "8278104".to_string(),
        ),
    );
    let fee_pool_balance = QueryMsg::DistributionFeePool { query_id: 1 };
    let resp: FeePoolResponse =
        from_json(query(deps.as_ref(), mock_env(), fee_pool_balance).unwrap()).unwrap();
    assert_eq!(
        resp,
        FeePoolResponse {
            last_submitted_local_height: 987,
            pool: FeePool {
                coins: vec![Coin::new(8278104u128, "uosmo")]
            },
        }
    )
}

#[test]
fn test_gov_proposals_query() {
    let mut deps = dependencies(&[]);

    let proposals_ids = vec![1, 2, 3];

    let msg = ExecuteMsg::RegisterGovernmentProposalsQuery {
        connection_id: "connection".to_string(),
        proposals_ids: proposals_ids.clone(),
        update_period: 10,
    };

    let keys = register_query(
        &mut deps,
        mock_env(),
        message_info(&Addr::unchecked(""), &[]),
        msg,
    );

    let registered_query =
        build_registered_query_response(1, QueryParam::Keys(keys), QueryType::KV, 987);

    let mut kv_results: Vec<StorageValue> = vec![];

    for id in proposals_ids {
        let value = build_interchain_query_gov_proposal_value(id);
        kv_results.push(value);
    }

    let proposals_response = QueryRegisteredQueryResultResponse {
        result: Some(QueryResult {
            kv_results,
            block: None,
            height: 0,
            revision: 0,
            allow_kv_callbacks: false,
        }),
    };

    deps.querier.add_registered_queries(1, registered_query);
    deps.querier.add_query_response(
        1,
        Binary::from(::prost::Message::encode_to_vec(&proposals_response).as_slice()),
    );

    let government_proposal = QueryMsg::GovernmentProposals { query_id: 1 };
    let resp: ProposalResponse =
        from_json(query(deps.as_ref(), mock_env(), government_proposal).unwrap()).unwrap();
    assert_eq!(
        resp,
        ProposalResponse {
            last_submitted_local_height: 987,
            proposals: GovernmentProposal {
                proposals: vec![
                    Proposal {
                        proposal_id: 1,
                        proposal_type: Some("/cosmos.gov.v1beta1.TextProposal".to_string()),
                        total_deposit: Vec::from([Coin {
                            denom: "stake".to_string(),
                            amount: Uint128::from_str("100").unwrap(),
                        }]),
                        status: 1,
                        submit_time: None,
                        deposit_end_time: None,
                        voting_end_time: None,
                        voting_start_time: None,
                        final_tally_result: Some(TallyResult {
                            abstain: Uint128::zero(),
                            yes: Uint128::zero(),
                            no: Uint128::zero(),
                            no_with_veto: Uint128::zero(),
                        }),
                    },
                    Proposal {
                        proposal_id: 2,
                        proposal_type: Some("/cosmos.gov.v1beta1.TextProposal".to_string()),
                        total_deposit: Vec::from([Coin {
                            denom: "stake".to_string(),
                            amount: Uint128::from_str("100").unwrap(),
                        }]),
                        status: 1,
                        submit_time: None,
                        deposit_end_time: None,
                        voting_end_time: None,
                        voting_start_time: None,
                        final_tally_result: Some(TallyResult {
                            abstain: Uint128::zero(),
                            yes: Uint128::zero(),
                            no: Uint128::zero(),
                            no_with_veto: Uint128::zero(),
                        }),
                    },
                    Proposal {
                        proposal_id: 3,
                        proposal_type: Some("/cosmos.gov.v1beta1.TextProposal".to_string()),
                        total_deposit: Vec::from([Coin {
                            denom: "stake".to_string(),
                            amount: Uint128::from_str("100").unwrap(),
                        }]),
                        status: 1,
                        submit_time: None,
                        deposit_end_time: None,
                        voting_end_time: None,
                        voting_start_time: None,
                        final_tally_result: Some(TallyResult {
                            abstain: Uint128::zero(),
                            yes: Uint128::zero(),
                            no: Uint128::zero(),
                            no_with_veto: Uint128::zero(),
                        }),
                    },
                ]
            },
        }
    )
}

#[test]
fn test_gov_proposal_votes_query() {
    let mut deps = dependencies(&[]);

    let proposals_ids = vec![1, 2, 3];

    let msg = ExecuteMsg::RegisterGovernmentProposalVotesQuery {
        connection_id: "connection".to_string(),
        proposals_ids: proposals_ids.clone(),
        voters: vec![
            "osmo1yz54ncxj9csp7un3xled03q6thrrhy9cztkfzs".to_string(),
            "osmo1yz54ncxj9csp7un3xled03q6thrrhy9cztkfzs".to_string(),
            "osmo1yz54ncxj9csp7un3xled03q6thrrhy9cztkfzs".to_string(),
        ],
        update_period: 10,
    };

    let keys = register_query(
        &mut deps,
        mock_env(),
        message_info(&Addr::unchecked(""), &[]),
        msg,
    );

    let registered_query =
        build_registered_query_response(1, QueryParam::Keys(keys), QueryType::KV, 325);

    let mut kv_results: Vec<StorageValue> = vec![];

    for id in proposals_ids {
        let value = build_interchain_query_gov_proposal_votes_value(id);
        kv_results.push(value);
    }

    let proposals_votes_response = QueryRegisteredQueryResultResponse {
        result: Some(QueryResult {
            kv_results,
            block: None,
            height: 0,
            revision: 0,
            allow_kv_callbacks: false,
        }),
    };

    deps.querier.add_registered_queries(1, registered_query);
    deps.querier.add_query_response(
        1,
        Binary::from(::prost::Message::encode_to_vec(&proposals_votes_response).as_slice()),
    );

    let government_proposal_votes = QueryMsg::GovernmentProposalVotes { query_id: 1 };
    let resp: ProposalVotesResponse =
        from_json(query(deps.as_ref(), mock_env(), government_proposal_votes).unwrap()).unwrap();
    assert_eq!(
        resp,
        ProposalVotesResponse {
            last_submitted_local_height: 325,
            votes: GovernmentProposalVotes {
                proposal_votes: vec![
                    ProposalVote {
                        proposal_id: 1,
                        voter: "cosmos1yz54ncxj9csp7un3xled03q6thrrhy9cztkfzs".to_string(),
                        options: vec![WeightedVoteOption {
                            weight: "1000000000000000000".to_string(),
                            option: 1,
                        }],
                    },
                    ProposalVote {
                        proposal_id: 2,
                        voter: "cosmos1yz54ncxj9csp7un3xled03q6thrrhy9cztkfzs".to_string(),
                        options: vec![WeightedVoteOption {
                            weight: "1000000000000000000".to_string(),
                            option: 1,
                        }],
                    },
                    ProposalVote {
                        proposal_id: 3,
                        voter: "cosmos1yz54ncxj9csp7un3xled03q6thrrhy9cztkfzs".to_string(),
                        options: vec![WeightedVoteOption {
                            weight: "1000000000000000000".to_string(),
                            option: 1,
                        }],
                    }
                ]
            }
        }
    )
}

#[test]
fn test_staking_validators_query() {
    let mut deps = dependencies(&[]);
    let validators = vec![
        "cosmosvaloper132juzk0gdmwuxvx4phug7m3ymyatxlh9734g4w".to_string(),
        "cosmosvaloper1sjllsnramtg3ewxqwwrwjxfgc4n4ef9u2lcnj0".to_string(),
    ];

    let msg = ExecuteMsg::RegisterStakingValidatorsQuery {
        connection_id: "connection".to_string(),
        update_period: 10,
        validators: validators.clone(),
    };

    let keys = register_query(
        &mut deps,
        mock_env(),
        message_info(&Addr::unchecked(""), &[]),
        msg,
    );

    let registered_query =
        build_registered_query_response(1, QueryParam::Keys(keys), QueryType::KV, 987);

    let mut kv_results: Vec<StorageValue> = vec![];

    for validator in validators {
        let value = build_interchain_query_staking_validator_value(validator);
        kv_results.push(value);
    }

    let validators_response = QueryRegisteredQueryResultResponse {
        result: Some(QueryResult {
            kv_results,
            block: None,
            height: 0,
            revision: 0,
            allow_kv_callbacks: false,
        }),
    };

    deps.querier.add_registered_queries(1, registered_query);
    deps.querier.add_query_response(
        1,
        Binary::from(::prost::Message::encode_to_vec(&validators_response).as_slice()),
    );
    let staking_validators = QueryMsg::StakingValidators { query_id: 1 };
    let resp: ValidatorResponse =
        from_json(query(deps.as_ref(), mock_env(), staking_validators).unwrap()).unwrap();
    assert_eq!(
        resp,
        ValidatorResponse {
            last_submitted_local_height: 987,
            validator: StakingValidator {
                validators: vec![
                    Validator {
                        operator_address: "cosmosvaloper132juzk0gdmwuxvx4phug7m3ymyatxlh9734g4w"
                            .to_string(),
                        consensus_pubkey: Some(vec!()),
                        status: 1,
                        tokens: "1".to_string(),
                        jailed: false,
                        delegator_shares: "1".to_string(),
                        unbonding_height: 0,
                        unbonding_time: None,
                        min_self_delegation: Decimal::from_str("1").unwrap(),
                        moniker: None,
                        identity: None,
                        website: None,
                        security_contact: None,
                        details: None,
                        rate: None,
                        max_rate: None,
                        max_change_rate: None,
                        update_time: None,
                    },
                    Validator {
                        operator_address: "cosmosvaloper1sjllsnramtg3ewxqwwrwjxfgc4n4ef9u2lcnj0"
                            .to_string(),
                        consensus_pubkey: Some(vec!()),
                        status: 1,
                        tokens: "1".to_string(),
                        jailed: false,
                        delegator_shares: "1".to_string(),
                        unbonding_height: 0,
                        unbonding_time: None,
                        min_self_delegation: Decimal::from_str("1").unwrap(),
                        moniker: None,
                        identity: None,
                        website: None,
                        security_contact: None,
                        details: None,
                        rate: None,
                        max_rate: None,
                        max_change_rate: None,
                        update_time: None,
                    },
                ]
            },
        }
    )
}

#[test]
fn test_validators_signing_infos_query() {
    let mut deps = dependencies(&[]);
    let validators = vec![
        "cosmosvalcons1yjf46k064988jdjje068zmrqg8xh4fqqe2wwnl".to_string(),
        "cosmosvalcons16tnak7apushwznnd3wtku8gm0rt3xytz6ut006".to_string(),
    ];

    let msg = ExecuteMsg::RegisterValidatorsSigningInfoQuery {
        connection_id: "connection".to_string(),
        update_period: 10,
        validators: validators.clone(),
    };

    let keys = register_query(
        &mut deps,
        mock_env(),
        message_info(&Addr::unchecked(""), &[]),
        msg,
    );

    let registered_query =
        build_registered_query_response(1, QueryParam::Keys(keys), QueryType::KV, 987);

    let mut kv_results: Vec<StorageValue> = vec![];

    for validator in validators {
        let value = build_interchain_query_validator_signing_info_value(validator);
        kv_results.push(value);
    }

    let validators_response = QueryRegisteredQueryResultResponse {
        result: Some(QueryResult {
            kv_results,
            block: None,
            height: 0,
            revision: 0,
            allow_kv_callbacks: false,
        }),
    };

    deps.querier.add_registered_queries(1, registered_query);
    deps.querier.add_query_response(
        1,
        Binary::from(::prost::Message::encode_to_vec(&validators_response).as_slice()),
    );
    let validators_signing_infos = QueryMsg::ValidatorsSigningInfos { query_id: 1 };
    let resp: ValidatorSigningInfoResponse =
        from_json(query(deps.as_ref(), mock_env(), validators_signing_infos).unwrap()).unwrap();
    assert_eq!(
        resp,
        ValidatorSigningInfoResponse {
            last_submitted_local_height: 987,
            signing_infos: SigningInfo {
                signing_infos: vec![
                    ValidatorSigningInfo {
                        address: "cosmosvalcons1yjf46k064988jdjje068zmrqg8xh4fqqe2wwnl".to_string(),
                        start_height: 1,
                        index_offset: 1,
                        jailed_until: None,
                        tombstoned: false,
                        missed_blocks_counter: 987675,
                    },
                    ValidatorSigningInfo {
                        address: "cosmosvalcons16tnak7apushwznnd3wtku8gm0rt3xytz6ut006".to_string(),
                        start_height: 1,
                        index_offset: 1,
                        jailed_until: None,
                        tombstoned: false,
                        missed_blocks_counter: 987675,
                    },
                ]
            },
        }
    )
}

#[test]
fn test_query_delegator_delegations() {
    let mut deps = dependencies(&[]);

    let msg = ExecuteMsg::RegisterDelegatorDelegationsQuery {
        connection_id: "connection".to_string(),
        update_period: 10,
        delegator: "osmo1yz54ncxj9csp7un3xled03q6thrrhy9cztkfzs".to_string(),
        validators: vec![
            "osmovaloper1r2u5q6t6w0wssrk6l66n3t2q3dw2uqny4gj2e3".to_string(),
            "osmovaloper1ej2es5fjztqjcd4pwa0zyvaevtjd2y5w37wr9t".to_string(),
            "osmovaloper1lzhlnpahvznwfv4jmay2tgaha5kmz5qxwmj9we".to_string(),
        ],
    };

    let keys = register_query(
        &mut deps,
        mock_env(),
        message_info(&Addr::unchecked(""), &[]),
        msg,
    );

    let delegations_response = QueryRegisteredQueryResultResponse {
        result: Some(QueryResult {
            // response for `RegisterDelegatorDelegationsQuery` with necessary KV values to test reconstruction logic.
            // The values are taken from osmosis network
            kv_results: vec![
                // params value of staking module for key 'staking/params'
                // value: Params
                StorageValue {
                    storage_prefix: "staking".to_string(),
                    key: BASE64_STANDARD.decode("UQ==").unwrap(),
                    value: BASE64_STANDARD.decode("CgQIgN9uEGQYByCQTioFdWF0b20yATA6FC0xMDAwMDAwMDAwMDAwMDAwMDAwQhMxMDAwMDAwMDAwMDAwMDAwMDAwShMxMDAwMDAwMDAwMDAwMDAwMDAw").unwrap(),
                    proof: None,
                },
                // delegation
                // from: osmo1yz54ncxj9csp7un3xled03q6thrrhy9cztkfzs
                // to: osmovaloper1r2u5q6t6w0wssrk6l66n3t2q3dw2uqny4gj2e3
                // delegation_shares: "5177628000000000000000000"
                StorageValue {
                    storage_prefix: "staking".to_string(),
                    key: decode_hex("311420a959e0d22e201f727137f2d7c41a5dc63b90b8141ab940697a73dd080edafeb538ad408b5cae0264").unwrap(),
                    value: BASE64_STANDARD.decode("Citvc21vMXl6NTRuY3hqOWNzcDd1bjN4bGVkMDNxNnRocnJoeTljenRrZnpzEjJvc21vdmFsb3BlcjFyMnU1cTZ0Nncwd3Nzcms2bDY2bjN0MnEzZHcydXFueTRnajJlMxoZNTE3NzYyODAwMDAwMDAwMDAwMDAwMDAwMA==").unwrap(),
                    proof: None,
                },
                // validator: osmovaloper1r2u5q6t6w0wssrk6l66n3t2q3dw2uqny4gj2e3
                // delegator_shares: "2845862840643000000000000000000"
                // total tokens: "2845862840643"
                StorageValue {
                    storage_prefix: "staking".to_string(),
                    key: decode_hex("21141ab940697a73dd080edafeb538ad408b5cae0264").unwrap(),
                    value: BASE64_STANDARD.decode("CjJvc21vdmFsb3BlcjFyMnU1cTZ0Nncwd3Nzcms2bDY2bjN0MnEzZHcydXFueTRnajJlMxJDCh0vY29zbW9zLmNyeXB0by5lZDI1NTE5LlB1YktleRIiCiCaZhCbacCetQorko3LfUUJX2UEyX38qBGVri8GyH8lcCADKg0yODQ1ODYyODQwNjQzMh8yODQ1ODYyODQwNjQzMDAwMDAwMDAwMDAwMDAwMDAwOqQCChRzdHJhbmdlbG92ZS12ZW50dXJlcxIQRDBEOEI4MEYxQzVDNzBCNRocaHR0cHM6Ly9zdHJhbmdlbG92ZS52ZW50dXJlcyrbAScuLi5iZWNhdXNlIG9mIHRoZSBhdXRvbWF0ZWQgYW5kIGlycmV2b2NhYmxlIGRlY2lzaW9uLW1ha2luZyBwcm9jZXNzIHdoaWNoIHJ1bGVzIG91dCBodW1hbiBtZWRkbGluZywgdGhlIERvb21zZGF5IG1hY2hpbmUgaXMgdGVycmlmeWluZyBhbmQgc2ltcGxlIHRvIHVuZGVyc3RhbmQgYW5kIGNvbXBsZXRlbHkgY3JlZGlibGUgYW5kIGNvbnZpbmNpbmcuJyAtIERyLiBTdHJhbmdlbG92ZUoAUkwKPAoRNTAwMDAwMDAwMDAwMDAwMDASEzEwMDAwMDAwMDAwMDAwMDAwMDAaEjUwMDAwMDAwMDAwMDAwMDAwMBIMCPetyYYGEKPoosUCWgEx").unwrap(),
                    proof: None,
                },
                // delegation
                // from: osmo1yz54ncxj9csp7un3xled03q6thrrhy9cztkfzs
                // to: osmovaloper1ej2es5fjztqjcd4pwa0zyvaevtjd2y5w37wr9t
                // delegation_shares: "29620221000000000000000000"
                StorageValue {
                    storage_prefix: "staking".to_string(),
                    key: decode_hex("311420a959e0d22e201f727137f2d7c41a5dc63b90b814cc9598513212c12c36a1775e2233b962e4d5128e").unwrap(),
                    value: BASE64_STANDARD.decode("Citvc21vMXl6NTRuY3hqOWNzcDd1bjN4bGVkMDNxNnRocnJoeTljenRrZnpzEjJvc21vdmFsb3BlcjFlajJlczVmanp0cWpjZDRwd2Ewenl2YWV2dGpkMnk1dzM3d3I5dBoaMjk2MjAyMjEwMDAwMDAwMDAwMDAwMDAwMDA=").unwrap(),
                    proof: None,
                },
                // validator: osmovaloper1ej2es5fjztqjcd4pwa0zyvaevtjd2y5w37wr9t
                // delegator_shares: "3054477259038000000000000000000"
                // total tokens: "3054477259038"
                StorageValue {
                    storage_prefix: "staking".to_string(),
                    key: decode_hex("2114cc9598513212c12c36a1775e2233b962e4d5128e").unwrap(),
                    value: BASE64_STANDARD.decode("CjJvc21vdmFsb3BlcjFlajJlczVmanp0cWpjZDRwd2Ewenl2YWV2dGpkMnk1dzM3d3I5dBJDCh0vY29zbW9zLmNyeXB0by5lZDI1NTE5LlB1YktleRIiCiA27dgAuZV/uS9FdsILGWLBw8eYPy+ZEyv1Df2VsrjXDiADKg0zMDU0NDc3MjU5MDM4Mh8zMDU0NDc3MjU5MDM4MDAwMDAwMDAwMDAwMDAwMDAwOoEBChFGcmVucyAo8J+knSzwn6SdKRIQQzQ3ODQ1MjI2NjYyQUY0NxoSaHR0cHM6Ly9mcmVucy5hcm15IhtzZWN1cml0eUBraWRzb250aGVibG9jay54eXoqKVlvdXIgZnJpZW5kbHkgdmFsaWRhdG9yIGZvciBjb3Ntb3MgY2hhaW5zQP3HpQFKCwj3zq6PBhCfrO86UkoKOgoRNTAwMDAwMDAwMDAwMDAwMDASEjUwMDAwMDAwMDAwMDAwMDAwMBoRNTAwMDAwMDAwMDAwMDAwMDASDAjg1rSQBhDkudCDAVoDNTAw").unwrap(),
                    proof: None,
                },
                // delegation
                // from: osmo1yz54ncxj9csp7un3xled03q6thrrhy9cztkfzs
                // to: osmovaloper1lzhlnpahvznwfv4jmay2tgaha5kmz5qxwmj9we
                // delegation_shares: "219920000000000000000000"
                StorageValue {
                    storage_prefix: "staking".to_string(),
                    key: decode_hex("311420a959e0d22e201f727137f2d7c41a5dc63b90b814f8aff987b760a6e4b2b2df48a5a3b7ed2db15006").unwrap(),
                    value: BASE64_STANDARD.decode("Citvc21vMXl6NTRuY3hqOWNzcDd1bjN4bGVkMDNxNnRocnJoeTljenRrZnpzEjJvc21vdmFsb3BlcjFsemhsbnBhaHZ6bndmdjRqbWF5MnRnYWhhNWttejVxeHdtajl3ZRoYMjE5OTIwMDAwMDAwMDAwMDAwMDAwMDAw").unwrap(),
                    proof: None,
                },
                // validator: osmovaloper1lzhlnpahvznwfv4jmay2tgaha5kmz5qxwmj9we
                // delegator_shares: "3201438898476000000000000000000"
                // total tokens: "3201438898476"
                StorageValue {
                    storage_prefix: "staking".to_string(),
                    key: decode_hex("2114f8aff987b760a6e4b2b2df48a5a3b7ed2db15006").unwrap(),
                    value: BASE64_STANDARD.decode("CjJvc21vdmFsb3BlcjFsemhsbnBhaHZ6bndmdjRqbWF5MnRnYWhhNWttejVxeHdtajl3ZRJDCh0vY29zbW9zLmNyeXB0by5lZDI1NTE5LlB1YktleRIiCiBPXCnkQvO+pU6oGbp4ZiJBBZ7RNoLYtXYFOEdpXGH+uSADKg0zMjAxNDM4ODk4NDc2Mh8zMjAxNDM4ODk4NDc2MDAwMDAwMDAwMDAwMDAwMDAwOp8CCgtDaXRhZGVsLm9uZRIQRUJCMDNFQjRCQjRDRkNBNxoTaHR0cHM6Ly9jaXRhZGVsLm9uZSroAUNpdGFkZWwub25lIGlzIGEgbXVsdGktYXNzZXQgbm9uLWN1c3RvZGlhbCBzdGFraW5nIHBsYXRmb3JtIHRoYXQgbGV0cyBhbnlvbmUgYmVjb21lIGEgcGFydCBvZiBkZWNlbnRyYWxpemVkIGluZnJhc3RydWN0dXJlIGFuZCBlYXJuIHBhc3NpdmUgaW5jb21lLiBTdGFrZSB3aXRoIG91ciBub2RlcyBvciBhbnkgb3RoZXIgdmFsaWRhdG9yIGFjcm9zcyBtdWx0aXBsZSBuZXR3b3JrcyBpbiBhIGZldyBjbGlja3NKAFJECjoKETUwMDAwMDAwMDAwMDAwMDAwEhIyMDAwMDAwMDAwMDAwMDAwMDAaETMwMDAwMDAwMDAwMDAwMDAwEgYIkKKzhgZaATE=").unwrap(),
                    proof: None,
                },
            ],
            block: None,
            height: 0,
            revision: 0,
            allow_kv_callbacks: false,
        }),
    };

    let registered_query =
        build_registered_query_response(1, QueryParam::Keys(keys), QueryType::KV, 987);

    deps.querier.add_query_response(
        1,
        Binary::from(::prost::Message::encode_to_vec(&delegations_response).as_slice()),
    );
    deps.querier.add_registered_queries(1, registered_query);

    let query_delegations = QueryMsg::GetDelegations { query_id: 1 };
    let resp: DelegatorDelegationsResponse =
        from_json(query(deps.as_ref(), mock_env(), query_delegations).unwrap()).unwrap();

    assert_eq!(
        resp,
        DelegatorDelegationsResponse {
            last_submitted_local_height: 987,
            delegations: vec![
                StdDelegation {
                    delegator: Addr::unchecked("osmo1yz54ncxj9csp7un3xled03q6thrrhy9cztkfzs"),
                    validator: "osmovaloper1r2u5q6t6w0wssrk6l66n3t2q3dw2uqny4gj2e3".to_string(),
                    amount: Coin::new(5177628u128, "uatom".to_string()),
                },
                StdDelegation {
                    delegator: Addr::unchecked("osmo1yz54ncxj9csp7un3xled03q6thrrhy9cztkfzs"),
                    validator: "osmovaloper1ej2es5fjztqjcd4pwa0zyvaevtjd2y5w37wr9t".to_string(),
                    amount: Coin::new(29620221u128, "uatom".to_string()),
                },
                StdDelegation {
                    delegator: Addr::unchecked("osmo1yz54ncxj9csp7un3xled03q6thrrhy9cztkfzs"),
                    validator: "osmovaloper1lzhlnpahvznwfv4jmay2tgaha5kmz5qxwmj9we".to_string(),
                    amount: Coin::new(219920u128, "uatom".to_string()),
                },
            ],
        }
    )
}

#[test]
fn test_sudo_tx_query_result_callback() {
    let mut deps = dependencies(&[]);
    let env = mock_env();
    let watched_addr: String = "neutron1fj6yqrkpw6fmp7f7jhj57dujfpwal4m25dafzx".to_string();
    let query_id: u64 = 1u64;
    let height = QueriesHeight {
        revision_number: 0u64,
        revision_height: 1u64,
    };
    let msg = ExecuteMsg::RegisterTransfersQuery {
        connection_id: "connection".to_string(),
        update_period: 1u64,
        recipients: vec![watched_addr.clone()],
        min_height: None,
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&Addr::unchecked(""), &[]),
        msg,
    )
    .unwrap();
    let registered_query = build_registered_query_response(
        1,
        QueryParam::TransactionsFilter(
            to_string(&vec![&TransactionFilterItem {
                field: RECIPIENT_FIELD.to_string(),
                op: TransactionFilterOp::Eq,
                value: TransactionFilterValue::String(watched_addr.clone()),
            }])
            .unwrap(),
        ),
        QueryType::TX,
        0,
    );
    deps.querier.add_registered_queries(1, registered_query);

    // simulate neutron's SudoTxQueryResult call with the following payload:
    // a sending from neutron10h9stc5v6ntgeygf5xf945njqq5h32r54rf7kf to watched_addr of 10000 stake
    let data: Binary = Binary::from(BASE64_STANDARD.decode("CpMBCpABChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEnAKLm5ldXRyb24xMGg5c3RjNXY2bnRnZXlnZjV4Zjk0NW5qcXE1aDMycjU0cmY3a2YSLm5ldXRyb24xZmo2eXFya3B3NmZtcDdmN2poajU3ZHVqZnB3YWw0bTI1ZGFmengaDgoFc3Rha2USBTEwMDAwEmcKUApGCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQJPYibh+Zef13ZkulPqI27rV5xswZ0H/vh1Tnymp1RHPhIECgIIARgAEhMKDQoFc3Rha2USBDEwMDAQwJoMGkAIiXNJXmA57KhyaWpKcLLr3602A5+hlvv/b4PgcDDm9y0qikC+biNZXin1dEMpHOvX9DwOWJ9utv6EKljiSyfT").unwrap());
    sudo_tx_query_result(deps.as_mut(), env.clone(), query_id, height.clone(), data).unwrap();

    // ensure the callback has worked and contract's state has changed
    let txs = RECIPIENT_TXS.load(&deps.storage, &watched_addr).unwrap();
    assert_eq!(
        txs,
        Vec::from([Transfer {
            recipient: watched_addr.clone(),
            sender: "neutron10h9stc5v6ntgeygf5xf945njqq5h32r54rf7kf".to_string(),
            denom: "stake".to_string(),
            amount: "10000".to_string(),
        }])
    );

    // simulate neutron's SudoTxQueryResult call with the following payload:
    // a sending from neutron10h9stc5v6ntgeygf5xf945njqq5h32r54rf7kf to another addr of 10000 stake
    let data: Binary = Binary::from(BASE64_STANDARD.decode("CpMBCpABChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEnAKLm5ldXRyb24xMGg5c3RjNXY2bnRnZXlnZjV4Zjk0NW5qcXE1aDMycjU0cmY3a2YSLm5ldXRyb24xNHV4dnUyMmxocmF6eXhhZGFxdjVkNmxzd3UwcDI3NmxsN2hya2waDgoFc3Rha2USBTEwMDAwEmcKUApGCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQJPYibh+Zef13ZkulPqI27rV5xswZ0H/vh1Tnymp1RHPhIECgIIARgAEhMKDQoFc3Rha2USBDEwMDAQwJoMGkBEv2CW/0gIrankNl4aGs9LXy2BKA6kAWyl4MUxmXnbnjRpgaNbQIyo4i7nUgVsuOpqzAdudM2M53OSU0Dmo5tF").unwrap());
    let res = sudo_tx_query_result(deps.as_mut(), env.clone(), query_id, height.clone(), data);

    // ensure the callback has returned an error and contract's state hasn't changed
    assert_eq!(
        res.unwrap_err(),
        NeutronError::Std(StdError::generic_err(
            "failed to find a matching transaction message",
        ))
    );
    let txs = RECIPIENT_TXS.load(&deps.storage, &watched_addr).unwrap();
    assert_eq!(
        txs,
        Vec::from([Transfer {
            recipient: watched_addr.clone(),
            sender: "neutron10h9stc5v6ntgeygf5xf945njqq5h32r54rf7kf".to_string(),
            denom: "stake".to_string(),
            amount: "10000".to_string(),
        }])
    );

    // simulate neutron's SudoTxQueryResult call with the following payload:
    // a sending from neutron10h9stc5v6ntgeygf5xf945njqq5h32r54rf7kf to watched_addr of 10000 stake
    let data: Binary = Binary::from(BASE64_STANDARD.decode("CpMBCpABChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEnAKLm5ldXRyb24xMGg5c3RjNXY2bnRnZXlnZjV4Zjk0NW5qcXE1aDMycjU0cmY3a2YSLm5ldXRyb24xZmo2eXFya3B3NmZtcDdmN2poajU3ZHVqZnB3YWw0bTI1ZGFmengaDgoFc3Rha2USBTEwMDAwEmcKUApGCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQJPYibh+Zef13ZkulPqI27rV5xswZ0H/vh1Tnymp1RHPhIECgIIARgAEhMKDQoFc3Rha2USBDEwMDAQwJoMGkAIiXNJXmA57KhyaWpKcLLr3602A5+hlvv/b4PgcDDm9y0qikC+biNZXin1dEMpHOvX9DwOWJ9utv6EKljiSyfT").unwrap());
    sudo_tx_query_result(deps.as_mut(), env, query_id, height, data).unwrap();

    // ensure the callback has worked and contract's state has changed again
    let txs = RECIPIENT_TXS.load(&deps.storage, &watched_addr).unwrap();
    assert_eq!(
        txs,
        Vec::from([
            Transfer {
                recipient: watched_addr.clone(),
                sender: "neutron10h9stc5v6ntgeygf5xf945njqq5h32r54rf7kf".to_string(),
                denom: "stake".to_string(),
                amount: "10000".to_string(),
            },
            Transfer {
                recipient: watched_addr,
                sender: "neutron10h9stc5v6ntgeygf5xf945njqq5h32r54rf7kf".to_string(),
                denom: "stake".to_string(),
                amount: "10000".to_string(),
            }
        ])
    );
}

#[test]
fn test_sudo_tx_query_result_min_height_callback() {
    let mut deps = dependencies(&[]);
    let env = mock_env();
    let watched_addr: String = "neutron1fj6yqrkpw6fmp7f7jhj57dujfpwal4m25dafzx".to_string();
    let query_id: u64 = 1u64;
    let height = QueriesHeight {
        revision_number: 0u64,
        revision_height: 1u64,
    };
    let msg = ExecuteMsg::RegisterTransfersQuery {
        connection_id: "connection".to_string(),
        update_period: 1u64,
        recipients: vec![watched_addr.clone()],
        min_height: Some(100000),
    };
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&Addr::unchecked(""), &[]),
        msg,
    )
    .unwrap();
    let registered_query = build_registered_query_response(
        1,
        QueryParam::TransactionsFilter(
            to_string(&vec![&TransactionFilterItem {
                field: RECIPIENT_FIELD.to_string(),
                op: TransactionFilterOp::Eq,
                value: TransactionFilterValue::String(watched_addr.clone()),
            }])
            .unwrap(),
        ),
        QueryType::TX,
        0,
    );
    deps.querier.add_registered_queries(1, registered_query);

    // simulate neutron's SudoTxQueryResult call with the following payload:
    // a sending from neutron10h9stc5v6ntgeygf5xf945njqq5h32r54rf7kf to watched_addr of 10000 stake
    let data: Binary = Binary::from(BASE64_STANDARD.decode("CpMBCpABChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEnAKLm5ldXRyb24xMGg5c3RjNXY2bnRnZXlnZjV4Zjk0NW5qcXE1aDMycjU0cmY3a2YSLm5ldXRyb24xZmo2eXFya3B3NmZtcDdmN2poajU3ZHVqZnB3YWw0bTI1ZGFmengaDgoFc3Rha2USBTEwMDAwEmcKUApGCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQJPYibh+Zef13ZkulPqI27rV5xswZ0H/vh1Tnymp1RHPhIECgIIARgAEhMKDQoFc3Rha2USBDEwMDAQwJoMGkAIiXNJXmA57KhyaWpKcLLr3602A5+hlvv/b4PgcDDm9y0qikC+biNZXin1dEMpHOvX9DwOWJ9utv6EKljiSyfT").unwrap());
    sudo_tx_query_result(deps.as_mut(), env.clone(), query_id, height.clone(), data).unwrap();

    // ensure the callback has worked and contract's state has changed
    let txs = RECIPIENT_TXS.load(&deps.storage, &watched_addr).unwrap();
    assert_eq!(
        txs,
        Vec::from([Transfer {
            recipient: watched_addr.clone(),
            sender: "neutron10h9stc5v6ntgeygf5xf945njqq5h32r54rf7kf".to_string(),
            denom: "stake".to_string(),
            amount: "10000".to_string(),
        }])
    );

    // simulate neutron's SudoTxQueryResult call with the following payload:
    // a sending from neutron10h9stc5v6ntgeygf5xf945njqq5h32r54rf7kf to another addr of 10000 stake
    let data: Binary = Binary::from(BASE64_STANDARD.decode("CpMBCpABChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEnAKLm5ldXRyb24xMGg5c3RjNXY2bnRnZXlnZjV4Zjk0NW5qcXE1aDMycjU0cmY3a2YSLm5ldXRyb24xNHV4dnUyMmxocmF6eXhhZGFxdjVkNmxzd3UwcDI3NmxsN2hya2waDgoFc3Rha2USBTEwMDAwEmcKUApGCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQJPYibh+Zef13ZkulPqI27rV5xswZ0H/vh1Tnymp1RHPhIECgIIARgAEhMKDQoFc3Rha2USBDEwMDAQwJoMGkBEv2CW/0gIrankNl4aGs9LXy2BKA6kAWyl4MUxmXnbnjRpgaNbQIyo4i7nUgVsuOpqzAdudM2M53OSU0Dmo5tF").unwrap());
    let res = sudo_tx_query_result(deps.as_mut(), env.clone(), query_id, height.clone(), data);

    // ensure the callback has returned an error and contract's state hasn't changed
    assert_eq!(
        res.unwrap_err(),
        NeutronError::Std(StdError::generic_err(
            "failed to find a matching transaction message",
        ))
    );
    let txs = RECIPIENT_TXS.load(&deps.storage, &watched_addr).unwrap();
    assert_eq!(
        txs,
        Vec::from([Transfer {
            recipient: watched_addr.clone(),
            sender: "neutron10h9stc5v6ntgeygf5xf945njqq5h32r54rf7kf".to_string(),
            denom: "stake".to_string(),
            amount: "10000".to_string(),
        }])
    );

    // simulate neutron's SudoTxQueryResult call with the following payload:
    // a sending from neutron10h9stc5v6ntgeygf5xf945njqq5h32r54rf7kf to watched_addr of 10000 stake
    let data: Binary = Binary::from(BASE64_STANDARD.decode("CpMBCpABChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEnAKLm5ldXRyb24xMGg5c3RjNXY2bnRnZXlnZjV4Zjk0NW5qcXE1aDMycjU0cmY3a2YSLm5ldXRyb24xZmo2eXFya3B3NmZtcDdmN2poajU3ZHVqZnB3YWw0bTI1ZGFmengaDgoFc3Rha2USBTEwMDAwEmcKUApGCh8vY29zbW9zLmNyeXB0by5zZWNwMjU2azEuUHViS2V5EiMKIQJPYibh+Zef13ZkulPqI27rV5xswZ0H/vh1Tnymp1RHPhIECgIIARgAEhMKDQoFc3Rha2USBDEwMDAQwJoMGkAIiXNJXmA57KhyaWpKcLLr3602A5+hlvv/b4PgcDDm9y0qikC+biNZXin1dEMpHOvX9DwOWJ9utv6EKljiSyfT").unwrap());
    sudo_tx_query_result(deps.as_mut(), env, query_id, height, data).unwrap();

    // ensure the callback has worked and contract's state has changed again
    let txs = RECIPIENT_TXS.load(&deps.storage, &watched_addr).unwrap();
    assert_eq!(
        txs,
        Vec::from([
            Transfer {
                recipient: watched_addr.clone(),
                sender: "neutron10h9stc5v6ntgeygf5xf945njqq5h32r54rf7kf".to_string(),
                denom: "stake".to_string(),
                amount: "10000".to_string(),
            },
            Transfer {
                recipient: watched_addr,
                sender: "neutron10h9stc5v6ntgeygf5xf945njqq5h32r54rf7kf".to_string(),
                denom: "stake".to_string(),
                amount: "10000".to_string(),
            }
        ])
    );
}
