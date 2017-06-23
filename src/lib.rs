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
use tokio_core::reactor::Handle;

pub struct Client {
    http: hyper::client::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>,
}

impl Client {
    pub fn new(handle: &Handle) -> Result<Self> {
        // TODO: Don't hardcode 4
        let connector = HttpsConnector::new(4, &handle).chain_err(
            || ErrorKind::Initialization,
        )?;

        let http = hyper::Client::configure().connector(connector).build(
            &handle,
        );

        Ok(Client { http })
    }

    pub fn create_app<'a>(
        &'a self,
        instance_url: &'a str,
        app: &'a api::oauth::Application,
    ) -> impl Future<Item = api::oauth::RegistrationResponse, Error = Error> {
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

        future::result(request_url)
            .and_then(move |url| {
                let mut req = hyper::Request::new(hyper::Method::Post, url);

                req.set_body(body);

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
                    ErrorKind::Deserialize(json_str.to_string())
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
        let base_url = format!("{}/oauth/token", instance_url);

        let request_url =
            url::Url::parse(&base_url).chain_err(|| ErrorKind::Uri(base_url.to_string()));

        let parsed_url = request_url.and_then(|mut url| {
            url.query_pairs_mut()
                .append_pair("client_id", client_id)
                .append_pair("client_secret", client_secret)
                .append_pair("code", code);

            url.as_str().parse::<hyper::Uri>().chain_err(|| {
                ErrorKind::Uri(url.into_string())
            })
        });

        future::result(parsed_url)
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
                    ErrorKind::Deserialize(json_str.to_string())
                })
            })
    }

    pub fn timeline<'a>(
        &'a self,
        instance_url: &'a str,
        access_token: &'a str,
        endpoint: timeline::Endpoint,
    ) -> impl Stream<Item = timeline::Event, Error = Error> {
        let request_url = format!("{}{}", instance_url, endpoint.as_path());

        let parsed_url = request_url.parse().chain_err(|| {
            ErrorKind::Uri(request_url.to_string())
        });

        let chunks = future::result(parsed_url)
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
