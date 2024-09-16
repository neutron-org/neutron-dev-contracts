use cw_storage_plus::Map;

pub const BEGIN_BLOCKER_SHEDULES: Map<String, u64> = Map::new("begin_blocker_shedules");
pub const END_BLOCKER_SHEDULES: Map<String, u64> = Map::new("end_blocker_shedules");
