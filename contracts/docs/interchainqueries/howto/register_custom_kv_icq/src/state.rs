use crate::types::BaseAccount;
use cw_storage_plus::Map;

/// Contains addresses of all watched addresses mapped to respective Interchain query ID.
pub const ICQ_ID_TO_WATCHED_ADDR: Map<u64, String> = Map::new("icq_id_to_watched_addr");
/// Contains last submitted accounts of remote addresses.
pub const REMOTE_ACCOUNTS: Map<String, BaseAccount> = Map::new("remote_accounts");
