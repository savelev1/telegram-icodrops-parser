use std::sync::mpsc::Sender;

use mysql_async::Conn;
use mysql_async::prelude::*;
use scraper::{Html, Selector};

use crate::parser::ico::{get_ico_text_status, ICO};
use crate::synchronizer::config::MySQLConfig;

pub mod ico;
use urlencoding::encode;

pub struct Parser {
    url: String,
    mysql_connection: Option<Conn>,
    ico: Vec<ICO>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            url: String::from("https://icodrops.com"),
            mysql_connection: None,
            ico: Vec::new(),
        }
    }

    pub async fn initialize(&mut self, config: &MySQLConfig) {
        let database_url = format!("mysql://{}:{}@{}:{}/{}", config.user, encode(&config.password), config.host, config.port, config.database);
        let pool = mysql_async::Pool::new(database_url);
        self.mysql_connection = Some(pool.get_conn().await.unwrap());
    }

    pub async fn parse(&mut self, sender: Sender<String>) {
        self.update_ico_list().await;

        let raw_html = reqwest::get(self.url.as_str()).await.unwrap().text().await.unwrap();
        let document = Html::parse_document(&raw_html);
        let cols_selector = Selector::parse(r#"div.col-lg-4"#).unwrap();
        let ico_card_selector = Selector::parse(r#"div.ico-card"#).unwrap();
        let ico_name_selector = Selector::parse(r#"h3>a"#).unwrap();
        let ico_interest_selector = Selector::parse(r#".interest>div"#).unwrap();
        let mut current_ico_status = 0;
        for col in document.select(&cols_selector) {
            let text_current_status = get_ico_text_status(current_ico_status as usize);

            for ico_card in col.select(&ico_card_selector) {
                let ico_name_option = ico_card.select(&ico_name_selector).next();
                if ico_name_option.is_some() {
                    let raw_ico_name = ico_name_option.unwrap().inner_html();
                    let ico_name = raw_ico_name.trim();
                    let ico_link_option = ico_name_option.unwrap().value().attr("href");
                    if ico_link_option.is_some() {
                        let ico_link = ico_link_option.unwrap().trim();

                        let ico_interest_option = ico_card.select(&ico_interest_selector).next();
                        if ico_interest_option.is_some() {
                            let raw_ico_interest = ico_interest_option.unwrap().inner_html();
                            let ico_interest = raw_ico_interest.trim();

                            let ico_option = self.get_ico(&ico_link);
                            if ico_option.is_some() {
                                let ico = ico_option.unwrap();
                                let id = ico.id;
                                let name = ico.name.clone();
                                let interest = ico.interest.clone();
                                let status = ico.status;
                                if interest != ico_interest {
                                    self.update_ico_interest(id, ico_interest).await;
                                    sender.send(format!("ICO <a href=\"{}\">{}</a> changed interest to <b>{}</b> (old: {})", ico_link, name, ico_interest, interest)).unwrap();
                                }
                                if status != current_ico_status {
                                    let text_status = get_ico_text_status(status as usize);
                                    self.update_ico_status(id, current_ico_status).await;
                                    sender.send(format!("ICO <a href=\"{}\">{}</a> changed status to <b>{}</b> (old: {})", ico_link, name, text_current_status, text_status)).unwrap();
                                }
                            } else {
                                self.insert_ico(ico_name, ico_interest, current_ico_status, ico_link).await;
                                sender.send(format!("New ICO <a href=\"{}\">{}</a>\nInterest: <b>{}</b>\nStatus: <b>{}</b>", ico_link, ico_name, ico_interest, text_current_status)).unwrap();
                            }
                        }
                    }
                }
            }
            current_ico_status = current_ico_status + 1;
        }
    }

    async fn update_ico_list(&mut self) {
        let conn = self.mysql_connection.as_mut().unwrap();
        let selected_ico = conn.exec_map(
            "SELECT id, name, interest, status, link FROM ico",
            (),
            |(id, name, interest, status, link)| { ICO { id, name, interest, status, link } },
        ).await.unwrap();
        self.ico = selected_ico;
    }

    async fn insert_ico(&mut self, name: &str, interest: &str, status: u32, link: &str) {
        let conn = self.mysql_connection.as_mut().unwrap();
        conn.exec_drop("INSERT INTO ico (name, interest, status, link) values (:name, :interest, :status, :link)",
                       params! {
                            "name" => name,
                            "interest" => interest,
                            "status" => status,
                            "link" => link,
                        },
        ).await.unwrap();
    }

    async fn update_ico_interest(&mut self, id: u32, interest: &str) {
        let conn = self.mysql_connection.as_mut().unwrap();
        conn.exec_drop("UPDATE ico SET interest = :interest WHERE id = :id",
                       params! {
                            "id" => id,
                            "interest" => interest,
                        },
        ).await.unwrap();
    }

    async fn update_ico_status(&mut self, id: u32, status: u32) {
        let conn = self.mysql_connection.as_mut().unwrap();
        conn.exec_drop("UPDATE ico SET status = :status WHERE id = :id",
                       params! {
                            "id" => id,
                            "status" => status,
                        },
        ).await.unwrap();
    }

    fn get_ico(&self, link: &str) -> Option<&ICO> {
        let mut found_ico = None;
        for ico in self.ico.iter() {
            if ico.link == link {
                found_ico = Some(ico);
                break;
            }
        }
        found_ico
    }
}