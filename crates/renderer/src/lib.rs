use thiserror::Error;

pub mod passes;

#[derive(Debug, Error)]
pub enum RendererError {
    #[error("other reason: {0}")]
    Other(&'static str),
    #[error(transparent)]
    RHI(#[from] rhi::RHIError),
    #[error(transparent)]
    AnyOther(#[from] anyhow::Error),
}
