use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

pub fn setup_logging() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env()?,
        )
        .init();
    Ok(())
}
