#[macro_use]
extern crate error_chain;

extern crate olifants;
extern crate futures;
extern crate tokio_core;

use futures::Stream;
use olifants::{Client, timeline};
use olifants::error::*;
use tokio_core::reactor::Core;

// Get from environment variable, or from stdin if variable is absent
fn get_env(name: &str) -> Result<String> {
    use std::env::VarError::*;
    use std::io::{Write, stdin, stdout};

    match std::env::var(name) {
        Ok(value) => Ok(value),
        Err(NotPresent) => {
            print!("{}: ", name);
            stdout().flush().chain_err(|| "failed to flush")?;

            let mut buffer = String::new();
            stdin()
                .read_line(&mut buffer)
                .map(|_| {
                    buffer.pop();
                    buffer
                })
                .chain_err(|| "stdin failed")
        }
        other => other.chain_err(|| format!("invalid {}", name)),
    }
}

quick_main!(|| -> Result<()> {
    let mut core = Core::new().chain_err(|| "could not create Core")?;
    let client = Client::new(&core.handle(), "olifants").chain_err(
        || "could not create Client",
    )?;

    let instance_url = get_env("INSTANCE_URL")?;
    let access_token = get_env("ACCESS_TOKEN")?;

    let timeline = client.timeline(&instance_url, access_token, timeline::Endpoint::Federated);

    core.run(timeline.for_each(|s| {
        println!("{:#?}", s);
        Ok(())
    })).chain_err(|| "received error from timeline")
});
