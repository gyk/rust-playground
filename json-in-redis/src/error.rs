use std::net::AddrParseError;

use thiserror::Error;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("{0}")]
    AddrParseError(#[from] AddrParseError),

    #[error("{0}")]
    JsonError(#[from] serde_json::Error),

    #[error("{0}")]
    RedisError(#[from] redis_async::error::Error),
}
