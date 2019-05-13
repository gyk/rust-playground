use std::net::AddrParseError;

use redis_async::error::Error as RedisError;
use serde_json::Error as JsonError;


pub use failure::{format_err, Fail, Error as FailureError};
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Internal error: {}", _0)]
    InternalError(String),

    #[fail(display = "{}", _0)]
    AddrParseError(#[fail(cause)] AddrParseError),

    #[fail(display = "{}", _0)]
    JsonError(#[fail(cause)] JsonError),

    #[fail(display = "{}", _0)]
    RedisError(#[fail(cause)] RedisError),
}

macro_rules! impl_from {
    ($e:ident) => {
        impl From<$e> for Error {
            fn from(e: $e) -> Self {
                Error::$e(e)
            }
        }
    };
}

impl_from!(AddrParseError);
impl_from!(JsonError);
impl_from!(RedisError);
