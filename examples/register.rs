extern crate olifants;

extern crate futures;
extern crate tokio_core;

use futures::Future;
use olifants::{Client, api};
use tokio_core::reactor::Core;

fn main() {
    let mut core = Core::new().expect("could not create Core");
    let client = Client::new(&core.handle(), "olifants").expect("could not create client");

    let app = api::oauth::App {
        client_name: "Example",
        redirect_uris: api::oauth::OOB_REDIRECT_URI,
        scopes: "read",
        website: "https://example.com",
    };

    let create = client.create_app("https://mastodon.social", &app);

    core.run(create.then(|result| {
        println!("{:?}", result);
        result
    })).unwrap();
}
