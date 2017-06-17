extern crate olifants;

extern crate futures;
extern crate tokio_core;

use futures::Future;
use olifants::{Application, Client};
use tokio_core::reactor::Core;

fn main() {
    let mut core = Core::new().expect("could not create Core");
    let client = Client::new(&core.handle()).expect("could not create client");

    let app = Application {
        client_name: "Example",
        redirect_uris: "urn:ietf:wg:oauth:2.0:oob",
        scopes: "read",
        website: "https://example.com",
    };

    let register = client.register("https://mastodon.social", &app);

    core.run(register.then(|result| {
        println!("{:?}", result);
        result
    })).unwrap();
}
