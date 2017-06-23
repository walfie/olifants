use std::borrow::Cow;
use url;

#[derive(Clone, Debug, Serialize)]
pub struct App<'a> {
    pub client_name: &'a str,
    pub redirect_uris: &'a str,
    pub scopes: &'a [Scope],
    pub website: &'a str,
}

impl<'a> App<'a> {
    pub fn as_form_urlencoded(&self) -> String {
        let scopes_str = self.scopes
            .iter()
            .map(|s| s.as_param())
            .collect::<Vec<_>>()
            .join(" ");

        url::form_urlencoded::Serializer::new(String::new())
            .append_pair("client_name", self.client_name)
            .append_pair("redirect_uris", self.redirect_uris)
            .append_pair("scopes", &scopes_str)
            .append_pair("website", self.website)
            .finish()
    }
}

#[derive(Clone, Debug, Serialize)]
pub enum Scope {
    Read,
    Write,
    Follow,
    Other(String),
}

impl Scope {
    pub fn as_param(&self) -> Cow<'static, str> {
        use self::Scope::*;

        match *self {
            Read => "read".into(),
            Write => "write".into(),
            Follow => "follow".into(),
            Other(ref s) => s.clone().into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
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

#[cfg(test)]
#[allow(unused_must_use)]
mod test {
    use super::*;

    #[test]
    fn app_as_form_urlencoded() {
        let app = App {
            client_name: "Example",
            redirect_uris: "https://example.com/hello",
            scopes: &[Scope::Read, Scope::Write, Scope::Other("unknown".into())],
            website: "https://example.com",
        };

        assert_eq!(
            app.as_form_urlencoded(),
            "client_name=Example\
            &redirect_uris=https%3A%2F%2Fexample.com%2Fhello\
            &scopes=read+write+unknown\
            &website=https%3A%2F%2Fexample.com"
        );
    }
}
