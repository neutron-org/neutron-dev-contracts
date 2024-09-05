use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddBeginBlockerSchedule { name: String },
    AddEndBlockerSchedule { name: String },
    RemoveBeginBlockerSchedule { name: String },
    RemoveEndBlockerSchedule { name: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetBeginBlockerScheduleCounter { name: String },
    GetEndBlockerScheduleCounter { name: String },
}
