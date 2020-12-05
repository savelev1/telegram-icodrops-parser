use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub telegram_bot_token: String,
    pub telegram_user_ids: Vec<u64>,
    pub mysql: MySQLConfig,
    pub parse_interval_sec: u64,
}

#[derive(Serialize, Deserialize)]
pub struct MySQLConfig {
    pub host: String,
    pub port: u32,
    pub database: String,
    pub user: String,
    pub password: String,
}
