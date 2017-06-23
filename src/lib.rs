#![feature(conservative_impl_trait)]
#![recursion_limit = "1024"]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate futures;

extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;
extern crate url;

pub mod error;
pub mod api;
pub mod timeline;

use error::*;
use futures::{Future, Stream};
use futures::future;
use hyper_tls::HttpsConnector;
use std::borrow::Cow;
use tokio_core::reactor::Handle;

pub struct Client {
    http: hyper::client::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>,
    user_agent: hyper::header::UserAgent,
}

impl Client {
    pub fn new<U>(handle: &Handle, user_agent: U) -> Result<Self>
    where
        U: Into<Cow<'static, str>>,
    {
        // TODO: Don't hardcode 4
        let connector = HttpsConnector::new(4, &handle).chain_err(
            || ErrorKind::Initialization,
        )?;

        let http = hyper::Client::configure().connector(connector).build(
            &handle,
        );

        Ok(Client {
            http,
            user_agent: hyper::header::UserAgent::new(user_agent),
        })
    }

    fn request<'a, F>(
        &'a self,
        uri: Result<hyper::Uri>,
        to_request: F,
    ) -> impl Future<Item = hyper::Response, Error = Error> + 'a
    where
        F: FnOnce(hyper::Uri) -> hyper::Request + 'a,
    {
        future::result(uri).and_then(move |u| {
            let mut request = to_request(u);
            request.headers_mut().set(self.user_agent.clone());

            self.http.request(request).then(|r| {
                r.chain_err(|| ErrorKind::Http)
            })
        })
    }

    pub fn create_app<'a>(
        &'a self,
        instance_url: &str,
        app: &api::oauth::App,
    ) -> impl Future<Item = api::oauth::CreateAppResponse, Error = Error> + 'a {
        let request_url = format!("{}/api/v1/apps", instance_url).parse().chain_err(
            || {
                ErrorKind::Uri(instance_url.to_string())
            },
        );

        let body = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("client_name", app.client_name)
            .append_pair("redirect_uris", app.redirect_uris)
            .append_pair("scopes", app.scopes)
            .append_pair("website", app.website)
            .finish();

        self.request(request_url, |url| {
            let mut req = hyper::Request::new(hyper::Method::Post, url);
            req.set_body(body);
            req
        }).and_then(|res| {
                res.body().concat2().then(
                    |r| r.chain_err(|| ErrorKind::Http),
                )
            })
            .and_then(|bytes| {
                let json_str = String::from_utf8_lossy(&bytes);
                serde_json::from_str(&json_str).chain_err(|| {
                    ErrorKind::Deserialize(json_str.to_string())
                })
            })
    }

    pub fn get_token<'a>(
        &'a self,
        instance_url: &str,
        client_id: &str,
        client_secret: &str,
        code: &str,
    ) -> impl Future<Item = api::oauth::TokenResponse, Error = Error> + 'a {
        let base_url = format!("{}/oauth/token", instance_url);

        let request_url = url::Url::parse(&base_url)
            .chain_err(|| ErrorKind::Uri(base_url.to_string()))
            .and_then(|mut url| {
                url.query_pairs_mut()
                    .append_pair("client_id", client_id)
                    .append_pair("client_secret", client_secret)
                    .append_pair("code", code);

                url.as_str().parse::<hyper::Uri>().chain_err(|| {
                    ErrorKind::Uri(url.into_string())
                })
            });

        self.request(request_url, move |url| {
            hyper::Request::new(hyper::Method::Post, url)
        }).and_then(|res| {
                res.body().concat2().then(
                    |r| r.chain_err(|| ErrorKind::Http),
                )
            })
            .and_then(|bytes| {
                let json_str = String::from_utf8_lossy(&bytes);
                serde_json::from_str(&json_str).chain_err(|| {
                    ErrorKind::Deserialize(json_str.to_string())
                })
            })
    }

    pub fn timeline<'a, S>(
        &'a self,
        instance_url: &'a str,
        access_token: S,
        endpoint: timeline::Endpoint,
    ) -> impl Stream<Item = timeline::Event, Error = Error> + 'a
    where
        S: Into<String> + 'a,
    {
        let request_url = format!("{}{}", instance_url, endpoint.as_path());

        let parsed_url = request_url.parse().chain_err(|| {
            ErrorKind::Uri(request_url.to_string())
        });

        let chunks = self.request(parsed_url, move |url| {
            let mut req = hyper::Request::new(hyper::Method::Get, url);
            req.headers_mut().set(hyper::header::Authorization(
                hyper::header::Bearer { token: access_token.into() },
            ));

            req
        }).map(|res| {
                res.body().then(
                    |result| result.chain_err(|| ErrorKind::Http),
                )
            })
            .flatten_stream();

        timeline::Timeline::from_lines(timeline::Lines::new(chunks))
    }
}
