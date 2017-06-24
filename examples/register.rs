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

    let app = oauth::App {
        client_name: "Example",
        redirect_uris: oauth::OOB_REDIRECT_URI,
        scopes: oauth::Scopes::new([oauth::Scope::Read]),
        website: "https://example.com",
    };

    let create = client.create_app("https://mastodon.social", &app);

    core.run(create.map(|result| {
        println!("{:?}", result);
    })).chain_err(|| "request failed")
});
