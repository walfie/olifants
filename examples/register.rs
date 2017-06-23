extern crate olifants;

extern crate futures;
extern crate tokio_core;

use futures::Future;
use olifants::Client;
use olifants::api::oauth;
use tokio_core::reactor::Core;

fn main() {
    let mut core = Core::new().expect("could not create Core");
    let client = Client::new(&core.handle(), "olifants").expect("could not create client");

    let app = oauth::App {
        client_name: "Example",
        redirect_uris: oauth::OOB_REDIRECT_URI,
        scopes: &[oauth::Scope::Read],
        website: "https://example.com",
    };

    let create = client.create_app("https://mastodon.social", &app);

    core.run(create.then(|result| {
        println!("{:?}", result);
        result
    })).unwrap();
}
