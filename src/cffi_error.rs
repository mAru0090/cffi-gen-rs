use thiserror::Error;

#[derive(Debug, Error)]
pub enum CFFIGeneralError {
    #[error("The value is null.")]
    NullAssertError,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
