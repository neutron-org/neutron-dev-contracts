use crate::msg::TestArg;
use cw_storage_plus::{Item, Map};

pub const TEST_ARGS: Map<&str, TestArg> = Map::new("test_args");

pub const STARGATE_QUERY_ID: Item<u64> = Item::new("stargate_query_id");

pub const STARGATE_REPLIES: Map<u64, String> = Map::new("stargate_replie");
