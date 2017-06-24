#[macro_use]
extern crate error_chain;

extern crate futures;
extern crate olifants;
extern crate tokio_core;

use futures::Future;
use olifants::Client;
use olifants::error::*;
use tokio_core::reactor::Core;

quick_main!(|| -> Result<()> {
    let mut core = Core::new().chain_err(|| "could not create Core")?;
    let client = Client::new(&core.handle(), "olifants").chain_err(
        || "could not create Client",
    )?;

    let client_id = "";
    let client_secret = "";
    let code = "";

    let token = client.get_token("https://mastodon.social", client_id, client_secret, code);

    core.run(token.map(|result| {
        println!("{:?}", result);
        ()
    })).chain_err(|| "request failed")
});
