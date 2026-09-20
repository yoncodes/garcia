#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailGiftRecord {
    pub reward: i32,
    pub reward_type: i32,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailRecord {
    pub email_id: i64,
    pub is_read: i32,
    pub taken: i32,
    pub sent_at: i32,
    pub sender: i32,
    pub title: String,
    pub content: String,
    pub gifts: Vec<MailGiftRecord>,
    pub expires_at: i32,
    pub sys_mail_id: i32,
    pub parameter: String,
}
