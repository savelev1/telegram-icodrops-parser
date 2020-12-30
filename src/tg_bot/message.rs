pub struct Message {
    pub chat_id: i64,
    pub text: String,
    pub parse_mode: String,
    pub disable_notification: bool,
    pub disable_web_page_preview: bool,
}

impl Message {
    pub fn new(chat_id: i64, text: String) -> Message {
        Message {
            chat_id,
            text,
            parse_mode: "HTML".to_owned(),
            disable_notification: false,
            disable_web_page_preview: false,
        }
    }

    pub fn set_disable_notification(&mut self, disable_notification: bool) -> &mut Message {
        self.disable_notification = disable_notification;
        self
    }

    pub fn set_disable_web_page_preview(&mut self, disable_web_page_preview: bool) -> &mut Message {
        self.disable_web_page_preview = disable_web_page_preview;
        self
    }
}