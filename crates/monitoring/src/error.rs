use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Metric error: {0}")]
    MetricError(String),

    #[error("Alert error: {0}")]
    AlertError(String),

    #[error("Health check failed: {0}")]
    HealthCheckError(String),

    #[error("Exporter error: {0}")]
    ExporterError(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
