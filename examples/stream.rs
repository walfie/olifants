extern crate olifants;

extern crate futures;
extern crate tokio_core;

use futures::Stream;
use olifants::{Client, timeline};
use tokio_core::reactor::Core;

fn main() {
    let mut core = Core::new().expect("could not create Core");
    let client = Client::new(&core.handle()).expect("could not create client");

    let access_token = "";

    let timeline = client.timeline(
        "https://mastodon.social",
        access_token,
        timeline::Endpoint::Federated,
    );

    core.run(timeline.for_each(|s| {
        println!("{:?}", s);
        Ok(())
    })).unwrap();
}
