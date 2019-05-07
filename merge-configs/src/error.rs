use std::net::AddrParseError;

use config::ConfigError;
use serde_json::Error as JsonError;

pub use failure::{format_err, Fail, Error as FailureError};
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "The value is not a JSON map")]
    NotJsonMapError,

    #[fail(display = "The index/key is invalid for JSON array/object")]
    JsonInvalidIndexError,

    #[fail(display = "{}", _0)]
    AddrParseError(#[fail(cause)] AddrParseError),

    #[fail(display = "{}", _0)]
    ConfigError(#[fail(cause)] ConfigError),

    #[fail(display = "{}", _0)]
    JsonError(#[fail(cause)] JsonError),
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
impl_from!(ConfigError);
impl_from!(JsonError);
