pub type StatusId = i64;
pub type AccountId = i64;
pub type MentionId = i64;
pub type AttachmentId = i64;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Status {
    pub id: StatusId,
    pub uri: String,
    pub url: String,
    pub account: Account,
    pub in_reply_to_id: Option<StatusId>,
    pub in_reply_to_account_id: Option<AccountId>,
    pub reblog: Option<Box<Status>>,
    pub content: String,
    pub created_at: String,
    pub reblogs_count: i32,
    pub favourites_count: i32,
    pub reblogged: Option<bool>,
    pub favourited: Option<bool>,
    pub sensitive: Option<bool>,
    pub spoiler_text: String,
    pub visibility: String, // TODO: Enum
    pub media_attachments: Vec<Attachment>,
    pub mentions: Vec<Mention>,
    pub tags: Vec<Tag>,
    pub application: Option<Application>,
    pub language: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Mention {
    pub id: MentionId,
    pub url: String,
    pub username: String,
    pub acct: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Tag {
    pub name: String,
    pub url: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Application {
    pub name: String,
    pub website: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Account {
    pub id: AccountId,
    pub username: String,
    pub acct: String,
    pub display_name: String,
    pub locked: bool,
    pub created_at: String,
    pub followers_count: i32,
    pub following_count: i32,
    pub statuses_count: i32,
    pub note: String,
    pub url: String,
    pub avatar: String,
    pub avatar_static: String,
    pub header: String,
    pub header_static: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Attachment {
    pub id: AttachmentId,
    #[serde(rename = "type")]
    pub media_type: String,
    pub url: String,
    pub remote_url: Option<String>,
    pub preview_url: String,
    pub text_url: Option<String>, // TODO: Add meta (dimensions, etc)
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Notification {
    pub id: i64,
    #[serde(rename = "type")]
    pub notification_type: String,
    pub created_at: String,
    pub account: Account,
    pub status: Option<Status>,
}
