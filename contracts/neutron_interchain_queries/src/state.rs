use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub type Recipient = str;

/// contains all transfers mapped by a recipient address observed by the contract.
pub const RECIPIENT_TXS: Map<&Recipient, Vec<Transfer>> = Map::new("recipient_txs");
/// contains number of transfers to addresses observed by the contract.
pub const TRANSFERS: Item<u64> = Item::new("transfers");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Transfer {
    pub recipient: String,
    pub sender: String,
    pub denom: String,
    pub amount: String,
}

pub const INTEGRATION_TESTS_QUERY_MOCK: Item<IntegrationTestsQueryMock> =
    Item::new("integration_tests_query_mock");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum IntegrationTestsQueryMock {
    Enabled,
    Disabled,
}

pub const KV_CALLBACK_STATS: Map<u64, u64> = Map::new("kv_callback_stats");

pub const KV_QUERY_ID_TO_CALLBACKS: Map<u64, QueryKind> = Map::new("kv_query_id_to_callbacks");

// contains query kinds that we expect to handle in `sudo_kv_query_result`
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum QueryKind {
    // Balance query
    Balance,
    // You can add your handlers to understand what query to deserialize by query_id in sudo callback
}
