use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub telegram_bot: TelegramBotConfig,
    pub mysql: MySQLConfig,
    pub parse_interval_sec: u64,
}

#[derive(Serialize, Deserialize)]
pub struct TelegramBotConfig {
    pub token: String,
    pub chat_id: i64,
}

#[derive(Serialize, Deserialize)]
pub struct MySQLConfig {
    pub host: String,
    pub port: u32,
    pub database: String,
    pub user: String,
    pub password: String,
}
