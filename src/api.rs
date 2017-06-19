#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Status {
    pub id: i64,
    pub created_at: String,
    pub in_reply_to_id: Option<i64>,
    pub in_reply_to_account_id: Option<i64>,
    pub sensitive: Option<bool>,
    pub spoiler_text: String,
    pub visibility: String,
    pub language: String,
    pub application: Option<Application>,
    pub account: Account,
    pub media_attachments: Vec<MediaAttachment>,
    pub mentions: Vec<Mention>,
    pub tags: Vec<Tag>,
    pub uri: String,
    pub content: String,
    pub url: String,
    pub reblogs_count: i32,
    pub favourites_count: i32,
    pub reblog: Option<Box<Status>>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Mention {
    pub id: i32,
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
    pub id: i64,
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
pub struct MediaAttachment {
    pub id: i64,
    pub remote_url: String,
    #[serde(rename = "type")]
    pub media_type: String,
    pub url: String,
    pub preview_url: String,
    pub text_url: Option<String>, // TODO: Add meta (dimensions, etc)
}
