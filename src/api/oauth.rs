#[derive(Debug, Serialize)]
pub struct App<'a> {
    pub client_name: &'a str,
    pub redirect_uris: &'a str,
    pub scopes: &'a str,
    pub website: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct CreateAppResponse {
    pub id: u32,
    pub redirect_uri: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub scope: Option<String>,
}

pub const OOB_REDIRECT_URI: &'static str = "urn:ietf:wg:oauth:2.0:oob";

pub fn authorization_url(instance_url: &str, client_id: &str, redirect_uri: &str) -> String {
    format!(
        "{}/oauth/authorize\
        ?client_id={}\
        &response_type=code\
        &redirect_uri={}",
        instance_url,
        client_id,
        redirect_uri
    )
}
