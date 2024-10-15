use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetPrice {
        base: String,
        quote: String,
    },
    GetPrices {
        currency_pair_ids: Vec<String>,
    },
    GetAllCurrencyPairs {

    },
}