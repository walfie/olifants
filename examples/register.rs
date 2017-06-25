#[macro_use]
extern crate error_chain;

extern crate olifants;
extern crate futures;
extern crate tokio_core;

use futures::{Future, IntoFuture};
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
    let scopes = &get_env("CLIENT_SCOPES")?;

    let app = oauth::App {
        client_name: &get_env("CLIENT_NAME")?,
        redirect_uris: oauth::OOB_REDIRECT_URI,
        scopes: oauth::Scopes::from_str(&scopes),
        website: &get_env("CLIENT_WEBSITE")?,
    };

    println!("\nRegistering app...\n");
    let register = client
        .create_app(&instance_url, &app)
        .map(|resp| {
            println!("Created app successfully!");
            println!("{:#?}\n", resp);

            println!(
                "Please visit the following URL to obtain an authorization code:\n{}\n",
                oauth::authorization_url(&instance_url, &resp.client_id, &resp.redirect_uri)
            );

            resp
        })
        .and_then(|app| {
            get_env("AUTH_CODE")
                .map(move |code| (app, code))
                .into_future()
        })
        .and_then(|(app, code)| {
            println!("\nRequesting access token...");
            println!("(if this fails, you can run the `token` example to try again)\n");

            client
                .get_token(
                    &instance_url,
                    oauth::OOB_REDIRECT_URI,
                    &app.client_id,
                    &app.client_secret,
                    &code,
                )
                .then(|r| r.chain_err(|| "failed to get access token"))
                .map(|token_resp| {
                    println!("{:#?}", token_resp);
                })
        });

    core.run(register).chain_err(|| "request failed")
});
