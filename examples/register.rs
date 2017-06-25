#[macro_use]
extern crate error_chain;

extern crate olifants;
extern crate futures;
extern crate tokio_core;

use futures::Future;
use olifants::Client;
use olifants::api::oauth;
use olifants::error::*;
use tokio_core::reactor::Core;

quick_main!(|| -> Result<()> {
    let mut core = Core::new().chain_err(|| "could not create Core")?;
    let client = Client::new(&core.handle(), "olifants").chain_err(
        || "could not create Client",
    )?;

    let instance_url = "https://mastodon.social";

    let app = oauth::App {
        client_name: "Example",
        redirect_uris: oauth::OOB_REDIRECT_URI,
        scopes: oauth::Scopes::new([oauth::Scope::Read]),
        website: "https://example.com",
    };

    let create = client.create_app(instance_url, &app);

    core.run(create.map(|resp| {
        println!("Created app successfully!");
        println!("id: {}", resp.id);
        println!("redirect_uri: {}", resp.redirect_uri);
        println!("client_id: {}", resp.client_id);
        println!("client_secret: {}", resp.client_secret);

        println!("");

        println!("Please visit the following URL to obtain an authorization code:");
        println!(
            "{}",
            oauth::authorization_url(instance_url, &resp.client_id, &resp.redirect_uri)
        );
    })).chain_err(|| "request failed")
});
