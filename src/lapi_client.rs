use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use color_eyre::eyre::Context;
use reqwest::{header, Client};
use time::OffsetDateTime;

use crate::types::DecisionsResponse;

pub struct LapiClient {
    api_key: String,
    client: Client,
    host: reqwest::Url,
}

impl LapiClient {
    pub fn try_from_api_key(host: reqwest::Url, api_key: String) -> color_eyre::Result<Self> {
        let mut builder = Client::builder();

        let mut headers = header::HeaderMap::new();
        let mut api_key_header =
            header::HeaderValue::from_str(api_key.as_str()).with_context(|| "invalid api key")?;
        api_key_header.set_sensitive(true);
        headers.insert("x-api-key", api_key_header);

        let builder = builder.default_headers(headers);
        let builder = builder.user_agent(env!("CARGO_CRATE_NAME"));

        Ok(Self {
            api_key,
            host,
            client: builder.build()?,
        })
    }

    async fn get_decisions(&self) -> color_eyre::Result<DecisionsResponse> {
        let url = self.host.join("/v1/decisions/stream")?;
        let res = self.client.get(url).send().await?.error_for_status()?;

        let decisions = res
            .json::<DecisionsResponse>()
            .await
            .with_context(|| "failed to decode decisions stream response")?;

        Ok(decisions)
    }

    pub fn stream_decisions(
        &self,
        wait_at_least: Duration,
    ) -> impl futures::Stream<Item = color_eyre::Result<DecisionsResponse>> + use<'_> {
        async_stream::try_stream! {
            let mut last_fetched = None;
            loop {
                if let Some(last_fetched) = last_fetched {
                    let duration_since = Instant::now() - last_fetched;
                    if duration_since < wait_at_least {
                        tokio::time::sleep(wait_at_least - duration_since).await;
                    }
                }
                last_fetched = Some(Instant::now());
                let decisions = self.get_decisions().await?;
                yield decisions;
            }

        }
    }
}
