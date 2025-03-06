use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use color_eyre::eyre::Context;
use reqwest::{header, Client};
use serde::{ser::SerializeStruct, Serialize};
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
        mut options: DecisionsStreamOptions,
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

                if options.startup {
                    options.startup = false;
                }
            }

        }
    }
}

#[derive(serde::Serialize)]
#[serde(untagged)]
pub enum ScenarioQueryOptions {
    Containing(String),
    NotContaining(String),
}

#[derive(Default, Serialize)]
pub struct DecisionsStreamOptions {
    /// If true, means that the remediation component is starting and a full list must be provided
    pub startup: bool,
    pub scopes: Vec<String>,
    pub origins: Vec<String>,

    #[serde(flatten)]
    #[serde(serialize_with = "serialize_scenarios")]
    pub scenarios: Vec<ScenarioQueryOptions>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_if_scenarios_are_serialized_correctly() {
        assert_eq!(
            serde_json::to_value(&DecisionsStreamOptions {
                scenarios: vec![
                    ScenarioQueryOptions::Containing("hello".to_owned()),
                    ScenarioQueryOptions::NotContaining("world".to_owned())
                ],
                ..Default::default()
            })
            .unwrap(),
            serde_json::json!({
                "startup": false,
                "origins": [],
                "scopes": [],
                "scenarios_containing": ["hello"],
                "scenarios_not_containing": ["world"],

            })
        );
    }
}

fn serialize_scenarios<S: serde::Serializer>(
    scenarios: &Vec<ScenarioQueryOptions>,
    ser: S,
) -> Result<S::Ok, S::Error> {
    let mut struct_ser = ser.serialize_struct("ScenarioQueryOptions", 2)?;

    let (containing, not_containing): (Vec<_>, Vec<_>) = scenarios
        .iter()
        .partition(|v| matches!(v, ScenarioQueryOptions::Containing(_)));

    struct_ser.serialize_field("scenarios_containing", &containing)?;
    struct_ser.serialize_field("scenarios_not_containing", &not_containing)?;

    struct_ser.end()
}

pub struct DecisionsStreamOptionsBuilder(DecisionsStreamOptions);

impl DecisionsStreamOptionsBuilder {
    pub fn new() -> Self {
        Self(DecisionsStreamOptions::default())
    }

    pub fn from_options(opts: DecisionsStreamOptions) -> Self {
        Self(opts)
    }

    pub fn startup(self, startup: bool) -> Self {
        let Self(mut opts) = self;
        opts.startup = startup;
        Self(opts)
    }

    pub fn scope(self, scope: String) -> Self {
        let Self(mut opts) = self;
        opts.scopes.push(scope);
        Self(opts)
    }

    pub fn origin(self, origin: String) -> Self {
        let Self(mut opts) = self;
        opts.origins.push(origin);
        Self(opts)
    }

    pub fn scenario(self, scenario: ScenarioQueryOptions) -> Self {
        let Self(mut opts) = self;
        opts.scenarios.push(scenario);
        Self(opts)
    }

    pub fn scenario_containing(self, scenario: String) -> Self {
        let Self(mut opts) = self;
        opts.scenarios
            .push(ScenarioQueryOptions::Containing(scenario));
        Self(opts)
    }

    pub fn scenario_not_containing(self, scenario: String) -> Self {
        let Self(mut opts) = self;
        opts.scenarios
            .push(ScenarioQueryOptions::NotContaining(scenario));
        Self(opts)
    }

    pub fn build(self) -> DecisionsStreamOptions {
        self.0
    }
}
