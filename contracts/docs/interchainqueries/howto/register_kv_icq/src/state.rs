use cw_storage_plus::Map;
use neutron_sdk::interchain_queries::v047::types::Balances;

/// Contains addresses of all watched addresses mapped to respective Interchain query ID.
pub const ICQ_ID_TO_WATCHED_ADDR: Map<u64, String> = Map::new("icq_id_to_watched_addr");
/// Contains last submitted balances of remote addresses.
pub const REMOTE_BALANCES: Map<String, Balances> = Map::new("remote_balances");
