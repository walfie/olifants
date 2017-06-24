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
use futures::{Future, IntoFuture, Stream, future};
use hyper::header::UserAgent;
use hyper_tls::HttpsConnector;
use std::borrow::Cow;
use tokio_core::reactor::Handle;

pub struct Client<H = HttpsConnector<hyper::client::HttpConnector>> {
    http: hyper::client::Client<H>,
    user_agent: UserAgent,
}

impl Client {
    pub fn new<U>(handle: &Handle, user_agent: U) -> Result<Self>
    where
        U: Into<Cow<'static, str>>,
    {
        let connector = HttpsConnector::new(4, &handle).chain_err(
            || ErrorKind::Initialization,
        )?;

        let http = hyper::Client::configure().connector(connector).build(
            &handle,
        );

        Ok(Client {
            http,
            user_agent: UserAgent::new(user_agent),
        })
    }

    pub fn from_hyper_client<H, U>(hyper: hyper::Client<H>, user_agent: U) -> Client<H>
    where
        U: Into<Cow<'static, str>>,
    {
        Client {
            http: hyper,
            user_agent: UserAgent::new(user_agent),
        }
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
        let response = uri.map(move |valid_uri| {
            let mut req = hyper::Request::new(method, valid_uri);
            req.headers_mut().set(self.user_agent.clone());
            req = modify_request(req);

            self.http.request(req).then(
                |r| r.chain_err(|| ErrorKind::Http),
            )
        }).into_future()
            .flatten();

        // If we receive a non-2XX error code, extract the body
        // into a string and return the response as an error
        response.and_then(|mut resp| if resp.status().is_success() {
            future::Either::A(future::ok(resp))
        } else {
            let version = resp.version();
            let status = resp.status();

            let mut headers = hyper::Headers::new();
            ::std::mem::swap(&mut headers, resp.headers_mut());

            let as_error = resp.body()
                .concat2()
                .then(|r| r.chain_err(|| ErrorKind::Http))
                .and_then(move |bytes| {
                    bail!(ErrorKind::StatusCode(
                        status,
                        version,
                        headers,
                        String::from_utf8_lossy(&bytes).into(),
                    ));
                });

            future::Either::B(as_error)
        })
    }

    fn request_json<T, F>(
        &self,
        uri: Result<hyper::Uri>,
        method: hyper::Method,
        modify_request: F,
    ) -> impl Future<Item = T, Error = Error>
    where
        F: FnOnce(hyper::Request) -> hyper::Request,
        T: serde::de::DeserializeOwned,
    {
        self.request(uri, method, modify_request).and_then(|res| {
            res.body()
                .concat2()
                .then(|r| r.chain_err(|| ErrorKind::Http))
                .and_then(|bytes| {
                    serde_json::from_slice(&bytes).chain_err(|| {
                        let invalid_json = String::from_utf8_lossy(&bytes);
                        ErrorKind::Deserialize(invalid_json.into())
                    })
                })
        })
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

        self.request_json(request_url, hyper::Method::Post, |mut req| {
            req.set_body(body);
            req
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

        self.request_json(request_url, hyper::Method::Post, |req| req)
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
