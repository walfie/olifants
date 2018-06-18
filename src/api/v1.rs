use chrono;

pub type DateTime = chrono::DateTime<chrono::Utc>;
pub type StatusId = String;
pub type AccountId = String;
pub type MentionId = String;
pub type AttachmentId = String;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Status {
    pub id: StatusId,
    pub uri: String,
    pub url: String,
    pub account: Account,
    pub in_reply_to_id: Option<StatusId>,
    pub in_reply_to_account_id: Option<AccountId>,
    pub reblog: Option<Box<Status>>,
    pub content: String,
    pub created_at: DateTime,
    pub reblogs_count: i32,
    pub favourites_count: i32,
    pub reblogged: Option<bool>,
    pub favourited: Option<bool>,
    pub sensitive: Option<bool>,
    pub spoiler_text: String,
    pub visibility: String, // TODO: Enum values -- direct, private, unlisted, public
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Account {
    pub id: AccountId,
    pub username: String,
    pub acct: String,
    pub display_name: String,
    pub locked: bool,
    pub created_at: DateTime,
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
    pub media_type: String, // TODO: Enum values -- image, video, gifv
    pub url: String,
    pub remote_url: Option<String>,
    pub preview_url: String,
    pub text_url: Option<String>, // TODO: Add meta (dimensions, etc)
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Notification {
    pub id: i64,
    #[serde(rename = "type")]
    pub notification_type: String, // TODO: Enum values -- mention, reblog, favourite, follow
    pub created_at: DateTime,
    pub account: Account,
    pub status: Option<Status>,
}
