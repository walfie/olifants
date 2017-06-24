use std::borrow::Cow;
use url;

#[derive(Clone, Debug, Serialize)]
pub struct App<'a> {
    pub client_name: &'a str,
    pub redirect_uris: &'a str,
    pub scopes: Scopes<'a>,
    pub website: &'a str,
}

impl<'a> App<'a> {
    pub fn as_form_urlencoded(&self) -> String {
        url::form_urlencoded::Serializer::new(String::new())
            .append_pair("client_name", self.client_name)
            .append_pair("redirect_uris", self.redirect_uris)
            .append_pair("scopes", &self.scopes.0)
            .append_pair("website", self.website)
            .finish()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Scopes<'a>(pub Cow<'a, str>);

impl<'a> Scopes<'a> {
    pub fn new<S>(scopes: S) -> Self
    where
        S: AsRef<[Scope]>,
    {
        Scopes(
            scopes
                .as_ref()
                .iter()
                .map(|s| s.as_param())
                .collect::<Vec<_>>()
                .join(" ")
                .into(),
        )
    }

    pub fn from_str(scopes: &'a str) -> Self {
        Scopes(scopes.into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Scope {
    Read,
    Write,
    Follow,
}

impl Scope {
    pub fn as_param(&self) -> &'static str {
        use self::Scope::*;

        match *self {
            Read => "read",
            Write => "write",
            Follow => "follow",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateAppResponse {
    pub id: u32,
    pub redirect_uri: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Deserialize, Serialize)]
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
    fn scopes_constructor() {
        let scopes1 = Scopes::new([Scope::Read, Scope::Write, Scope::Follow]);
        assert_eq!(scopes1.0, "read write follow");

        let scopes2 = Scopes::from_str("whatever");
        assert_eq!(scopes2.0, "whatever");
    }

    #[test]
    fn app_as_form_urlencoded() {
        let app = App {
            client_name: "Example",
            redirect_uris: "https://example.com/hello",
            scopes: Scopes::from_str("read write whatever"),
            website: "https://example.com",
        };

        assert_eq!(
            app.as_form_urlencoded(),
            "client_name=Example\
            &redirect_uris=https%3A%2F%2Fexample.com%2Fhello\
            &scopes=read+write+whatever\
            &website=https%3A%2F%2Fexample.com"
        );
    }
}
