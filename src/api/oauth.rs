#[derive(Debug, Serialize)]
pub struct Application<'a> {
    pub client_name: &'a str,
    pub redirect_uris: &'a str,
    pub scopes: &'a str,
    pub website: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct RegistrationResponse {
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
