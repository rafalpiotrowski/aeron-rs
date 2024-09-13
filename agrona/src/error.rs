use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgronaError {
    #[error("Argument out of bounds error: {0}")]
    ArgumentOutOfBounds(String),
}
