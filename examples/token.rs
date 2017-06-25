#[macro_use]
extern crate error_chain;

extern crate futures;
extern crate olifants;
extern crate tokio_core;

use futures::Future;
use olifants::Client;
use olifants::api::oauth;
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
    let client_id = get_env("CLIENT_ID")?;
    let client_secret = get_env("CLIENT_SECRET")?;
    let auth_code = {
        if std::env::var("AUTH_CODE").is_err() {
            println!(
                "\nPlease visit the following URL to obtain an authorization code:\n{}\n",
                oauth::authorization_url(&instance_url, &client_id, oauth::OOB_REDIRECT_URI)
            );
        }
        get_env("AUTH_CODE")?
    };

    println!("\nRequesting access token...\n");

    let token = client
        .get_token(
            &instance_url,
            oauth::OOB_REDIRECT_URI,
            &client_id,
            &client_secret,
            &auth_code,
        )
        .map(|token_resp| {
            println!("{:#?}", token_resp);
        });

    core.run(token).chain_err(|| "request failed")
});
