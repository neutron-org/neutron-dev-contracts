use crate::state::PriceFeedRate;
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(String)]
    GetError {},
    #[returns(Vec<PriceFeedRate>)]
    // Returns the RefData of a given symbol
    GetRate {},
}

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Request {},

    // Only for integration tests
    SetRate { symbol: String, rate: PriceFeedRate },
}
