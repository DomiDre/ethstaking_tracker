// Module to read transactions from etherscan
use reqwest;
use serde::{de, Deserialize, Deserializer};
use serde_json::Value;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct TransactionResponse {
    status: String,
    message: String,
    result: Vec<Transaction>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
#[allow(non_snake_case)]
struct Transaction {
    #[serde(deserialize_with = "de_int")]
    blockNumber: u64,
    #[serde(deserialize_with = "de_int")]
    timeStamp: u64,
    hash: String,
    nonce: String,
    blockHash: String,
    transactionIndex: String,
    from: String,
    to: String,
    #[serde(deserialize_with = "de_int")]
    value: u64,
    gas: String,
    #[serde(deserialize_with = "de_int")]
    gasPrice: u64,
    isError: String,
    txreceipt_status: String,
    input: String,
    contractAddress: String,
    cumulativeGasUsed: String,
    #[serde(deserialize_with = "de_int")]
    gasUsed: u64,
    confirmations: String,
    methodId: String,
    functionName: String,
}

#[derive(Debug)]
pub struct ParsedTransaction {
    pub id: String,
    pub timestamp: chrono::NaiveDateTime,
    pub eth_value: f64,
}

fn de_int<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => s.parse().map_err(de::Error::custom)?,
        Value::Number(num) => num.as_u64().ok_or(de::Error::custom("Invalid number"))? as u64,
        _ => return Err(de::Error::custom("wrong type")),
    })
}

fn parse_transactions(transactions: Vec<Transaction>) -> Vec<ParsedTransaction> {
    let mut parsed_transactions = Vec::new();
    for transaction in transactions {
        let timestamp = chrono::NaiveDateTime::from_timestamp(transaction.timeStamp as i64, 0);
        let eth_value = transaction.value as f64 / 1e18;
        let tx = ParsedTransaction {
            id: transaction.hash,
            timestamp,
            eth_value,
        };
        parsed_transactions.push(tx);
    }
    parsed_transactions
}

pub async fn get_transactions(
    config: &crate::config::Config,
) -> Result<Vec<ParsedTransaction>, String> {
    /* Get list of transactions from Etherscan for account specified in config */
    let transaction_url = format!(
        "{url}?module=account&action=txlist&address={address}&startblock=0&endblock=99999999&page=1&offset=10&sort=asc&apikey={token}",
        url=config.etherscan.api_url, address=config.account.address, token=config.etherscan.api_token
    );
    let response_result = reqwest::get(transaction_url).await;
    let response = match response_result {
        Ok(response) => response,
        Err(_) => return Err("Failed to request transactions.".to_string()),
    };
    match response.status() {
        reqwest::StatusCode::OK => match response.json::<TransactionResponse>().await {
            Ok(txs) => Ok(parse_transactions(txs.result)),
            Err(_) => Err("Unexpected response for transaction.".to_string()),
        },
        other => Err(format!(
            "Failed to request transactions with status code {}",
            other
        )),
    }
}
