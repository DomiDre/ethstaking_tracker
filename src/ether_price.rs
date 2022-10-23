// Module to read transactions from etherscan
use reqwest;
use serde::{de, Deserialize, Deserializer};
use serde_json::Value;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct PriceResponse {
    id: String,
    symbol: String,
    market_data: MarketData,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct MarketData {
    current_price: CurrentPrice,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct CurrentPrice {
    #[serde(deserialize_with = "de_float")]
    eur: f64,
}

fn de_float<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => s.parse().map_err(de::Error::custom)?,
        Value::Number(num) => num.as_f64().ok_or(de::Error::custom("Invalid number"))? as f64,
        _ => return Err(de::Error::custom("wrong type")),
    })
}

pub async fn get_price(
    config: &crate::config::Config,
    datetime: chrono::NaiveDateTime,
) -> Result<f64, String> {
    /* Request price on `datetime` from coingecko */
    // -H 'accept: application/json'
    let transaction_url = format!(
        "{url}/coins/ethereum/history?date={date}&localization=false",
        url = config.coingecko.api_url,
        date = datetime.format("%d-%m-%Y").to_string()
    );
    let response_result = reqwest::get(transaction_url).await;
    let response = match response_result {
        Ok(response) => response,
        Err(_) => return Err("Failed to request transactions.".to_string()),
    };
    match response.status() {
        reqwest::StatusCode::OK => match response.json::<PriceResponse>().await {
            Ok(txs) => Ok(txs.market_data.current_price.eur),
            Err(_) => Err("Unexpected response for transaction.".to_string()),
        },
        other => Err(format!(
            "Failed to request transactions with status code {}",
            other
        )),
    }
}
