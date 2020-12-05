use telegram_bot::{Message, MessageKind};

use crate::synchronizer::Synchronizer;

pub async fn on_bot_message(synchronizer: &mut Synchronizer, message: &Message) {
    if let MessageKind::Text { ref data, .. } = message.kind {
        let message_from_id = message.from.id.to_string().parse::<u64>().unwrap_or(0);
        if synchronizer.config.telegram_user_ids.len() == 0 || synchronizer.config.telegram_user_ids.contains(&message_from_id) {
            if synchronizer.bot.is_started {
                if data == "/start" {
                    synchronizer.bot.send(&message, &String::from("Already started")).await;
                } else {
                    synchronizer.bot.send(&message, &format!("I don't know a <code>{}</code> command.", data)).await;
                }
            } else {
                if data == "/start" {
                    synchronizer.bot.is_started = true;
                    synchronizer.start_parser();
                    synchronizer.bot.send(&message, &format!("Parser started")).await;
                } else {
                    synchronizer.bot.send(&message, &String::from("Please type <code>/start</code>.")).await;
                }
            }
        } else {
            synchronizer.bot.send(&message, &format!("Sorry, I do not talk to strangers. Your id <code>{}</code>.", message.from.id)).await;
        }
    }
}