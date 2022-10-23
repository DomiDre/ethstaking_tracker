use serde::Deserialize;
use toml;

#[derive(Deserialize)]
pub struct Config {
    pub etherscan: EtherscanConfig,
    pub account: Account,
    pub coingecko: CoingeckoConfig,
    pub records: Records,
}

#[derive(Deserialize)]
pub struct Records {
    pub path: String,
}

#[derive(Deserialize)]
pub struct EtherscanConfig {
    pub api_token: String,
    pub api_url: String,
}

#[derive(Deserialize)]
pub struct CoingeckoConfig {
    pub api_url: String,
}

#[derive(Deserialize)]
pub struct Account {
    pub address: String,
}

pub fn read_config() -> std::io::Result<Config> {
    /* Read config.toml from assets folder */
    let content = std::fs::read_to_string("assets/config.toml")?;
    Ok(toml::from_str(&content)?)
}
