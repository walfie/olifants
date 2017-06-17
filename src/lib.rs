#![feature(conservative_impl_trait)]
#![recursion_limit = "1024"]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate error_chain;

extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;
extern crate serde_urlencoded;
extern crate tokio_core;

mod error;

use error::*;
use futures::{Future, Stream};
use futures::future;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Handle;


#[derive(Debug, Serialize)]
pub struct Application<'a> {
    pub client_name: &'a str,
    pub redirect_uris: &'a str,
    pub scopes: &'a str,
    pub website: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct RegistrationResponse {
    id: u32,
    redirect_uri: String,
    client_id: String,
    client_secret: String,
}

pub struct Client {
    http: hyper::client::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>,
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
        app: &'a Application,
    ) -> impl Future<Item = RegistrationResponse, Error = Error> {
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
                let json_str = std::str::from_utf8(&bytes).chain_err(|| ErrorKind::Api)?;
                serde_json::from_str(json_str).chain_err(|| {
                    ErrorKind::JsonDecode(json_str.to_string())
                })
            })
    }
}
