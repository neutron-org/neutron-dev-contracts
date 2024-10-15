use neutron_std::types::slinky::types::v1::CurrencyPair;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Params { },
    LastUpdated { },
    MarketMap { },
    Market {
        currency_pair: CurrencyPair,
    },
}