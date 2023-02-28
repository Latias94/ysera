use thiserror::Error;

pub mod passes;
pub mod shader;

#[derive(Debug, Error)]
pub enum RendererError {
    #[error("other reason: {0}")]
    Other(&'static str),
    #[error(transparent)]
    RHI(#[from] rhi::RHIError),
    #[error("Spirq error: {0}")]
    Spirq(#[from] spirq::Error),
    #[error(transparent)]
    AnyOther(#[from] anyhow::Error),
}
