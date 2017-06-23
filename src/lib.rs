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
use futures::{Future, IntoFuture, Stream};
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

    fn request<F>(
        &self,
        uri: Result<hyper::Uri>,
        method: hyper::Method,
        modify_request: F,
    ) -> impl Future<Item = hyper::Response, Error = Error>
    where
        F: FnOnce(hyper::Request) -> hyper::Request,
    {
        uri.map(move |valid_uri| {
            let mut req = hyper::Request::new(method, valid_uri);
            req.headers_mut().set(self.user_agent.clone());
            req = modify_request(req);

            self.http.request(req).then(
                |r| r.chain_err(|| ErrorKind::Http),
            )
        }).into_future()
            .flatten()
    }

    pub fn create_app(
        &self,
        instance_url: &str,
        app: &api::oauth::App,
    ) -> impl Future<Item = api::oauth::CreateAppResponse, Error = Error> {
        let request_url = format!("{}/api/v1/apps", instance_url).parse().chain_err(
            || {
                ErrorKind::Uri(instance_url.to_string())
            },
        );

        let body = app.as_form_urlencoded();

        self.request(request_url, hyper::Method::Post, |mut req| {
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

    pub fn get_token(
        &self,
        instance_url: &str,
        client_id: &str,
        client_secret: &str,
        code: &str,
    ) -> impl Future<Item = api::oauth::TokenResponse, Error = Error> {
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

        self.request(request_url, hyper::Method::Post, |req| req)
            .and_then(|res| {
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

    pub fn timeline<S>(
        &self,
        instance_url: &str,
        access_token: S,
        endpoint: timeline::Endpoint,
    ) -> impl Stream<Item = timeline::Event, Error = Error>
    where
        S: Into<String>,
    {
        let request_url = format!("{}{}", instance_url, endpoint.as_path());

        let parsed_url = request_url.parse().chain_err(|| {
            ErrorKind::Uri(request_url.to_string())
        });

        let chunks = self.request(parsed_url, hyper::Method::Get, |mut req| {
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
