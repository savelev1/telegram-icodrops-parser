pub mod message;

use std::collections::HashMap;
use crate::tg_bot::message::Message;

pub struct TgBot {
    token: String,
}

impl TgBot {
    pub fn new(token: &String) -> TgBot {
        TgBot {
            token: token.clone(),
        }
    }

    pub async fn send_message(&self, message: &mut Message) {
        let mut map = HashMap::new();
        let chat_id = &message.chat_id.to_string();
        let disable_web_page_preview = &message.disable_web_page_preview.to_string();
        let disable_notification = &message.disable_notification.to_string();
        map.insert("text", &message.text);
        map.insert("chat_id", chat_id);
        map.insert("parse_mode", &message.parse_mode);
        map.insert("disable_web_page_preview", disable_web_page_preview);
        map.insert("disable_notification", disable_notification);

        let client = reqwest::Client::new();
        let url = format!("https://api.telegram.org/bot{}/sendMessage", &self.token);
        client.post(&url)
            .json(&map)
            .send()
            .await.unwrap();
    }
}