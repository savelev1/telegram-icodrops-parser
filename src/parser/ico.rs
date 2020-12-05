pub struct ICO {
    pub id: u32,
    pub name: String,
    pub interest: String,
    pub status: u32,
    pub link: String,
}

pub fn get_ico_text_status(status: usize) -> String {
    let statuses = ["Active", "Upcoming", "Ended"];
    statuses[status].to_owned()
}