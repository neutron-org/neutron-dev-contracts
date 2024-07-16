use cw_storage_plus::Map;

pub const CHUNKS: Map<u64, String> = Map::new("chunks");
