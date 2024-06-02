use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut,
    Env, MessageInfo, Response, StdError, StdResult, Uint64, Int128,
};
use cw2::set_contract_version;

use neutron_sdk::bindings::marketmap::query::{MarketMapQuery, MarketMapResponse, MarketResponse};
use neutron_sdk::bindings::oracle::types::CurrencyPair;
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

use neutron_sdk::bindings::oracle::query::{
    GetAllCurrencyPairsResponse, GetPriceResponse, GetPricesResponse, OracleQuery,
};

use crate::msg::{
    InstantiateMsg, ExecuteMsg, QueryMsg,
};

const CONTRACT_NAME: &str = concat!("crates.io:neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: instantiate");
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> StdResult<Response<NeutronMsg>> {
    Ok(Default::default())
}

#[entry_point]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPrice {
            base_symbol,
            quote_currency,
            max_blocks_old,
        } => query_recent_valid_price(deps, env, base_symbol, quote_currency, max_blocks_old),
        QueryMsg::GetPrices {
            base_symbols,
            quote_currency,
            max_blocks_old,
        } => query_recent_valid_prices(deps, env, base_symbols, quote_currency, max_blocks_old),
    }
}

fn query_recent_valid_price(
    deps: Deps<NeutronQuery>,
    env: Env,
    base_symbol: String,
    quote_currency: String,
    max_blocks_old: Uint64,
) -> StdResult<Binary> {
    // 1. check if "symbol" in x/oracle and x/marketmap

    let currency_pair: CurrencyPair = CurrencyPair{
        base: base_symbol.clone(), quote: quote_currency.clone(),
    };

    let oracle_currency_pairs_query: OracleQuery = OracleQuery::GetAllCurrencyPairs{};
    let oracle_currency_pairs_response: GetAllCurrencyPairsResponse = deps.querier.query(
        &oracle_currency_pairs_query.into(),
    )?;
    if oracle_currency_pairs_response.currency_pairs.contains(&currency_pair) == false {
        StdError::generic_err(format!(
            "Market {base_symbol}, {quote_currency} not found in x/oracle module"
        ));
    }

    let marketmap_currency_pairs_query: MarketMapQuery = MarketMapQuery::MarketMap{};
    let marketmap_currency_pairs_response: MarketMapResponse = deps.querier.query(
        &marketmap_currency_pairs_query.into(),
    )?;
    if marketmap_currency_pairs_response.market_map.markets.contains_key(&base_symbol.clone()) == false {
        StdError::generic_err(format!(
            "Market {base_symbol}, {quote_currency} not found in x/marketmap module"
        ));
    }

    // 2. check if "symbol" enabled in x/marketmap

    let marketmap_market_query: MarketMapQuery = MarketMapQuery::Market{
        currency_pair: currency_pair.clone(),
    };
    let marketmap_market_response: MarketResponse = deps.querier.query(
        &marketmap_market_query.into(),
    )?;
    if marketmap_market_response.market.ticker.enabled == false {
        StdError::generic_err(format!(
            "Market {base_symbol}, {quote_currency} not enabled in x/marketmap module"
        ));
    }

    // 3. check if block_timestamp is not too old

    // get current_block_height
    let current_block_height: u64 = env.block.height;

    let oracle_price_query: OracleQuery = OracleQuery::GetPrice{
        currency_pair: currency_pair.clone(),
    };
    let oracle_price_response: GetPriceResponse = deps.querier.query(
        &oracle_price_query.into(),
    )?;
    if (current_block_height - oracle_price_response.price.block_height) > max_blocks_old.u64() {
        StdError::generic_err(format!(
            "Market {base_symbol}, {quote_currency} price is older than {max_blocks_old} blocks"
        ));
    }

    // 4. fetch the price from x/oracle module
    let market_price: Int128 = oracle_price_response.price.price;

    // 5. make sure the price value is not None
    if oracle_price_response.nonce == 0 {
        StdError::generic_err(format!(
            "Market {base_symbol}, {quote_currency} price is nil"
        ));
    }

    // 6. return the price as response with proper metadata
    Ok(
        to_json_binary(&oracle_price_response)?
    )
}

fn query_recent_valid_prices(
    deps: Deps<NeutronQuery>,
    env: Env,
    base_symbols: Vec<String>,
    quote_currency: String,
    max_blocks_old: Uint64,
) -> StdResult<Binary> {
    // 1. check if all vec<"symbol"> in x/oracle and x/marketmap

    let currency_pairs: Vec<CurrencyPair> = base_symbols.iter().map(|symbol| CurrencyPair{
        base: symbol.to_string(),
        quote: quote_currency.clone(),
    }).collect();

    let oracle_currency_pairs_query: OracleQuery = OracleQuery::GetAllCurrencyPairs{};
    let oracle_currency_pairs_response: GetAllCurrencyPairsResponse = deps.querier.query(
        &oracle_currency_pairs_query.into(),
    )?;

    let _ = currency_pairs.iter().map(
        |curr_pair| 
        if oracle_currency_pairs_response.currency_pairs.contains(curr_pair) == false {
            StdError::generic_err(format!(
                "Market {0}, {1} not found in x/oracle module", curr_pair.base, curr_pair.quote,
            ));
        }
    );

    let marketmap_currency_pairs_query: MarketMapQuery = MarketMapQuery::MarketMap{};
    let marketmap_currency_pairs_response: MarketMapResponse = deps.querier.query(
        &marketmap_currency_pairs_query.into(),
    )?;

    let _ = currency_pairs.iter().map(
        |curr_pair| 
        if marketmap_currency_pairs_response.market_map.markets.contains_key(&curr_pair.base) == false {
            StdError::generic_err(format!(
                "Market {0}, {1} not found in x/oracle module", curr_pair.base, curr_pair.quote,
            ));
        }
    );

    // 2. check if all vec<"symbol"> enabled in x/marketmap

    let _ = currency_pairs.iter().map(
        |curr_pair| 
        if marketmap_currency_pairs_response.market_map.markets.get(
            &curr_pair.base
        ).unwrap().ticker.enabled == false {
            StdError::generic_err(format!(
                "Market {0}, {1} not enabled in x/oracle module", curr_pair.base, curr_pair.quote,
            ));
        }
    );

    // 3. check if block_timestamp is not too old

    // TODO: use GetPrices { currency_pair_ids } when calculation of 
    // `currency_pair_ids` is known

    // get current_block_height
    let current_block_height: u64 = env.block.height;

    let oracle_price_responses: Vec<GetPriceResponse> = currency_pairs.iter().map(|curr_pair| 
        deps.querier.query(
            &OracleQuery::GetPrice {
                currency_pair: curr_pair.clone(),
            }.into(),
        ).unwrap()
    ).collect();

    let _ = oracle_price_responses.iter().enumerate().map(
        |(index, price_response)| 
        if (current_block_height - price_response.price.block_height) 
                > max_blocks_old.u64() {
            StdError::generic_err(format!(
                "Market {0}, {1} not enabled in x/oracle module", currency_pairs[index].base, currency_pairs[index].quote,
            ));
        }
    );

    // 4. fetch the price from x/oracle module
    let market_prices: Vec<Int128> = oracle_price_responses.iter().map(
        |price_response| price_response.price.price
    ).collect();

    // 5. make sure the price value is not None
    let _ = oracle_price_responses.iter().enumerate().map(
        |(index, price_response)| 
        if price_response.nonce == 0 {
            StdError::generic_err(format!(
                "Market {0}, {1} price is nil", currency_pairs[index].base, currency_pairs[index].quote,
            ));
        }
    );

    // 6. return the price as response with proper metadata
    Ok(
        to_json_binary(&GetPricesResponse {
            prices: oracle_price_responses,
        }
    )?)
}
