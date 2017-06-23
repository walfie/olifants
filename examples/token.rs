extern crate olifants;

extern crate futures;
extern crate tokio_core;

use futures::Future;
use olifants::Client;
use tokio_core::reactor::Core;

fn main() {
    let mut core = Core::new().expect("could not create Core");
    let client = Client::new(&core.handle(), "olifants").expect("could not create client");

    let client_id = "";
    let client_secret = "";
    let code = "";

    let token = client.get_token("https://mastodon.social", client_id, client_secret, code);

    core.run(token.then(|result| {
        println!("{:?}", result);
        result
    })).unwrap();
}
