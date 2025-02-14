use color_eyre::eyre::Context;
use once_cell::sync::Lazy;
use time::macros::format_description;
use time::UtcOffset;
use tokio::runtime::Handle;
use tracing::metadata::LevelFilter;
use tracing_error::ErrorLayer;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

pub static LOCAL_OFFSET: Lazy<UtcOffset> = Lazy::new(|| {
    Handle::try_current().expect_err(
        "LOCAL_OFFSET should NOT be retrieved from within a possible multi-threaded environment for the first time",
    );

    UtcOffset::current_local_offset().expect("failed to acquire local offset")
});

pub fn setup() -> Result<(), color_eyre::Report> {
    let offset = *LOCAL_OFFSET;
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    let timer = OffsetTime::new(offset, format_description!("[hour]:[minute]:[second]"));

    let fmt_layer = tracing_subscriber::fmt::layer().with_timer(timer);

    let fmt_layer_filtered = fmt_layer.with_filter(env_filter);

    tracing_subscriber::registry()
        // add the console layer to the subscriber
        // .with(console_subscriber::spawn())
        .with(ErrorLayer::default())
        .with(fmt_layer_filtered)
        // set the registry as the default subscriber
        .init();

    color_eyre::install().with_context(|| "failed to install color_eyre")?;

    Ok(())
}
