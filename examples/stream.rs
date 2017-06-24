#[macro_use]
extern crate error_chain;

extern crate olifants;
extern crate futures;
extern crate tokio_core;

use futures::Stream;
use olifants::{Client, timeline};
use olifants::error::*;
use tokio_core::reactor::Core;

quick_main!(|| -> Result<()> {
    let mut core = Core::new().chain_err(|| "could not create Core")?;
    let client = Client::new(&core.handle(), "olifants").chain_err(
        || "could not create Client",
    )?;

    let access_token = "";

    let timeline = client.timeline(
        "https://mastodon.social",
        access_token,
        timeline::Endpoint::Federated,
    );

    core.run(timeline.for_each(|s| {
        println!("{:?}", s);
        Ok(())
    })).chain_err(|| "received error from timeline")
});
