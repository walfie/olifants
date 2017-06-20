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
extern crate serde_urlencoded;
extern crate tokio_core;

pub mod error;
pub mod api;
pub mod timeline;

use error::*;
use futures::{Future, Stream};
use futures::future;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Handle;

pub struct Client {
    http: hyper::client::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>,
}

const REDIRECT_URI: &'static str = "urn:ietf:wg:oauth:2.0:oob";

pub fn authorize_url(instance_url: &str, client_id: &str) -> String {
    format!(
        "{}/oauth/authorize\
        ?client_id={}\
        &response_type=code\
        &redirect_uri={}",
        instance_url,
        client_id,
        REDIRECT_URI
    )
}

impl Client {
    pub fn new(handle: &Handle) -> Result<Self> {
        // TODO: Don't hardcode 4
        let connector = HttpsConnector::new(4, &handle).chain_err(|| {
            ErrorKind::ClientInitialization
        })?;

        let http = hyper::Client::configure().connector(connector).build(
            &handle,
        );

        Ok(Client { http })
    }

    pub fn register<'a>(
        &'a self,
        instance_url: &'a str,
        app: &'a api::oauth::Application,
    ) -> impl Future<Item = api::oauth::RegistrationResponse, Error = Error> {
        let app_params = serde_urlencoded::to_string(app).chain_err(|| ErrorKind::Encode);

        let request_url = format!("{}/api/v1/apps", instance_url).parse().chain_err(
            || {
                ErrorKind::InvalidUrl
            },
        );

        future::result(request_url)
            .join(future::result(app_params))
            .and_then(move |(url, params)| {
                let mut req = hyper::Request::new(hyper::Method::Post, url);
                req.set_body(params);

                self.http.request(req).then(
                    |r| r.chain_err(|| ErrorKind::Http),
                )
            })
            .and_then(|res| {
                res.body().concat2().then(
                    |r| r.chain_err(|| ErrorKind::Http),
                )
            })
            .and_then(|bytes| {
                let json_str = String::from_utf8_lossy(&bytes);
                serde_json::from_str(&json_str).chain_err(|| {
                    ErrorKind::JsonDecode(json_str.to_string())
                })
            })
    }

    pub fn get_token<'a>(
        &'a self,
        instance_url: &'a str,
        client_id: &'a str,
        client_secret: &'a str,
        code: &'a str,
    ) -> impl Future<Item = api::oauth::TokenResponse, Error = Error> {
        let request_url = format!(
            "{}/oauth/token\
            ?client_id={}\
            &client_secret={}\
            &code={}\
            &grant_type=authorization_code\
            &redirect_uri={}",
            instance_url,
            client_id,
            client_secret,
            code,
            REDIRECT_URI
        ).parse()
            .chain_err(|| ErrorKind::InvalidUrl);

        future::result(request_url)
            .and_then(move |url| {
                let req = hyper::Request::new(hyper::Method::Post, url);

                self.http.request(req).then(
                    |r| r.chain_err(|| ErrorKind::Http),
                )
            })
            .and_then(|res| {
                res.body().concat2().then(
                    |r| r.chain_err(|| ErrorKind::Http),
                )
            })
            .and_then(|bytes| {
                let json_str = String::from_utf8_lossy(&bytes);
                serde_json::from_str(&json_str).chain_err(|| {
                    ErrorKind::JsonDecode(json_str.to_string())
                })
            })
    }

    pub fn timeline<'a>(
        &'a self,
        instance_url: &'a str,
        access_token: &'a str,
        endpoint: timeline::Endpoint,
    ) -> impl Stream<Item = timeline::Event, Error = Error> {
        let request_url = format!("{}{}", instance_url, endpoint.as_path())
            .parse()
            .chain_err(|| ErrorKind::InvalidUrl);

        let chunks = future::result(request_url)
            .and_then(move |url| {
                let mut req = hyper::Request::new(hyper::Method::Get, url);
                req.headers_mut().set(hyper::header::Authorization(
                    hyper::header::Bearer { token: access_token.to_string() },
                ));

                self.http.request(req).then(|result| {
                    result.chain_err(|| ErrorKind::Http)
                })
            })
            .map(|res| {
                res.body().then(
                    |result| result.chain_err(|| ErrorKind::Http),
                )
            })
            .flatten_stream();

        timeline::Timeline::from_lines(timeline::Lines::new(chunks))
    }
}
