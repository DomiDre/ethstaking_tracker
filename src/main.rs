mod config;
mod ether_price;
mod transactions;

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct StakingRecord {
    transaction_id: String,
    date: String,
    eth_value: f64,
    eth_price: f64,
    gains: f64,
}

#[tokio::main]
async fn main() {
    let config = config::read_config().unwrap();
    let records_path = &config.records.path;
    let mut records_updated = false;
    // read existing records

    let mut records = Vec::new();
    let mut already_known_hashs = HashSet::new();
    match csv::Reader::from_path(records_path) {
        Ok(mut reader) => {
            for record in reader.deserialize() {
                let record: StakingRecord = record.expect("Invalid line in staking_records file.");
                already_known_hashs.insert(record.transaction_id.clone());
                records.push(record);
            }
        }
        Err(_) => {}
    };

    let transactions = match transactions::get_transactions(&config).await {
        Ok(result) => result,
        Err(err) => panic!("Failed to fetch transactions: {:?}", err),
    };
    if transactions.len() == 0 {
        return;
    }

    for tx in transactions {
        if already_known_hashs.contains(&tx.id) {
            continue;
        }
        let price = match ether_price::get_price(&config, tx.timestamp).await {
            Ok(result) => result,
            Err(err) => panic!("Failed to fetch price: {:?}", err),
        };
        let record = StakingRecord {
            transaction_id: tx.id,
            date: tx.timestamp.to_string(),
            eth_value: tx.eth_value,
            eth_price: price,
            gains: tx.eth_value * price,
        };
        records.push(record);
        records_updated = true;
        std::thread::sleep(std::time::Duration::from_secs_f64(1.2));  // rate limit in free API is 50 calls per minute
    }

    if records_updated {
        println!("Updating records file.");
        let mut csv_writer =
            csv::Writer::from_path(records_path).expect("Could not init csv file.");
        for record in records {
            csv_writer
                .serialize(record)
                .expect("Failed to write record to file");
        }
        csv_writer.flush().expect("Failed to write to csv file");
    }
}
