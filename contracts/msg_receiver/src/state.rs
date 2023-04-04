use crate::msg::TestArg;
use cw_storage_plus::Map;

pub const TEST_ARGS: Map<&str, TestArg> = Map::new("test_args");
