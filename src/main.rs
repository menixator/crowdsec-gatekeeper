use std::fmt::write;

use color_eyre::eyre::{eyre, Context};
use futures::{pin_mut, StreamExt};
use lapi_client::DecisionsStreamOptions;

mod diagnostics;
mod lapi_client;
mod types;

fn main() -> Result<(), color_eyre::Report> {
    diagnostics::setup()?;
    start()
}

#[tokio::main]
async fn start() -> Result<(), color_eyre::Report> {
    let auth_key =
        std::env::var("CROWDSEC_API_KEY").with_context(|| "Failed to read CROWDSEC_API_KEY")?;

    let crowdsec_api_host =
        std::env::var("CROWDSEC_API_HOST").with_context(|| "Failed to read CROWDSEC_API_HOST")?;

    let client = lapi_client::LapiClient::try_from_api_key(
        crowdsec_api_host
            .parse()
            .with_context(|| "failed to parse CROWDSEC_API_HOST as a url")?,
        auth_key,
    )?;

    let stream = client.stream_decisions(
        DecisionsStreamOptions::default(),
        std::time::Duration::from_secs(5),
    );

    pin_mut!(stream);

    while let Some(value) = stream.next().await {
        let decisions = value?;
        println!("{:#?}", decisions);
    }

    Ok(())
}
