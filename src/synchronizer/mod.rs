use std::fs::File;
use std::io::BufReader;
use std::string::String;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use tokio::runtime::Runtime;

use crate::parser::Parser;
use crate::synchronizer::config::Config;
use crate::tg_bot::TgBot;
use crate::tg_bot::message::Message;

pub mod config;

pub struct Synchronizer
{
    pub config: Config,
    pub bot: TgBot,
    parser_main_receiver: Option<Receiver<String>>,
    pub is_started_parser: bool,
}

impl Synchronizer
{
    pub fn new() -> Synchronizer {
        let file = File::open("config.json").expect("Can't open config.json");
        let reader = BufReader::new(file);
        let config: Config = serde_json::from_reader(reader).expect("Can't parse JSON");
        let token = config.telegram_bot.token.clone();

        Synchronizer {
            config,
            bot: TgBot::new(&token),
            parser_main_receiver: None,
            is_started_parser: false,
        }
    }

    pub async fn run(&mut self) {
        self.bot.send_message(Message::new(self.config.telegram_bot.chat_id, "Bot restarted. Syncing db...".to_owned()).set_disable_notification(true)).await;
        self.start_parser();
        self.run_ticker().await;
    }

    async fn run_ticker(&mut self) {
        loop {
            self.tick().await;
        }
    }

    async fn tick(&mut self) {
        self.check_parser_message().await;
    }

    pub fn start_parser(&mut self) {
        if self.is_started_parser == false {
            self.is_started_parser = true;
            let (parser_thread_sender, parser_main_receiver) = mpsc::channel();
            self.parser_main_receiver = Some(parser_main_receiver);
            let cloned_parser_thread_sender = parser_thread_sender.clone();
            let parse_interval = self.config.parse_interval_sec;
            thread::Builder::new().name("process_parser".to_string()).spawn(move || {
                Runtime::new().unwrap().block_on(async {
                    let file = File::open("config.json").expect("Can't open config.json");
                    let reader = BufReader::new(file);
                    let config: Config = serde_json::from_reader(reader).expect("Can't parse JSON");
                    let mut parser = Parser::new();
                    parser.initialize(&config.mysql).await;
                    loop {
                        let sender = cloned_parser_thread_sender.clone();
                        parser.parse("https://icodrops.com/category/active-ico/", 0, &sender).await;
                        parser.parse("https://icodrops.com/category/upcoming-ico/", 1, &sender).await;
                        parser.parse("https://icodrops.com/category/ended-ico/", 2, &sender).await;
                        sleep(Duration::from_secs(parse_interval))
                    }
                });
            }).unwrap();
        }
    }

    async fn check_parser_message(&mut self) {
        if self.parser_main_receiver.is_some() {
            match self.parser_main_receiver.as_ref().unwrap().recv_timeout(Duration::from_millis(50)) {
                Ok(message) => {
                    self.bot.send_message(&mut Message::new(self.config.telegram_bot.chat_id, message).set_disable_web_page_preview(true)).await;
                }
                Err(_) => (),
            }
        }
    }
}