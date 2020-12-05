use std::fs::File;
use std::io::BufReader;
use std::string::String;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use telegram_bot::*;
use tokio::runtime::Runtime;

use crate::logic::on_bot_message;
use crate::parser::Parser;
use crate::synchronizer::config::Config;
use crate::tg_bot::TgBot;

pub mod config;

pub struct Synchronizer
{
    pub config: Config,
    pub bot: TgBot,
    bot_main_receiver: Option<Receiver<Message>>,
    parser_main_receiver: Option<Receiver<String>>,
    pub is_started_parser: bool,
}

impl Synchronizer
{
    pub fn new() -> Synchronizer {
        let file = File::open("config.json").expect("Can't open config.json");
        let reader = BufReader::new(file);
        let config: Config = serde_json::from_reader(reader).expect("Can't parse JSON");
        let token = config.telegram_bot_token.clone();

        Synchronizer {
            config,
            bot: TgBot::new(&token),
            bot_main_receiver: None,
            parser_main_receiver: None,
            is_started_parser: false,
        }
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        self.run_bot().await;
        self.run_ticker().await;

        Ok(())
    }

    async fn run_bot(&mut self) {
        let (bot_thread_sender, bot_main_receiver) = mpsc::channel();
        self.bot_main_receiver = Some(bot_main_receiver);
        self.bot.run(bot_thread_sender).await;
    }

    async fn run_ticker(&mut self) {
        let mut messages: Vec<Message> = Vec::new();
        loop {
            self.tick(&mut messages).await;
        }
    }

    async fn tick(&mut self, mut messages: &mut Vec<Message>) {
        self.check_bot_messages(&mut messages).await;
        self.check_parser_message(&mut messages).await;
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
                        parser.parse(cloned_parser_thread_sender.clone()).await;
                        sleep(Duration::from_secs(parse_interval))
                    }
                });
            }).unwrap();
        }
    }

    async fn check_bot_messages(&mut self, messages: &mut Vec<Message>) {
        match self.bot_main_receiver.as_ref().unwrap().recv_timeout(Duration::from_millis(50)) {
            Ok(message) => {
                if messages.len() == 0 {
                    messages.push(message);
                } else {
                    messages[0] = message;
                }

                on_bot_message(self, &messages[0]).await;
            }
            Err(_) => (),
        }
    }

    async fn check_parser_message(&mut self, messages: &mut Vec<Message>) {
        if self.parser_main_receiver.is_some() {
            match self.parser_main_receiver.as_ref().unwrap().recv_timeout(Duration::from_millis(50)) {
                Ok(message) => {
                    self.bot.send(&messages[0], &message).await;
                }
                Err(_) => (),
            }
        }
    }
}