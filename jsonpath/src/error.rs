use std::num::ParseIntError;

use pest::error::Error as PestError;

use crate::parser::Rule;

use failure::Fail;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "The next item of pairs does not exist")]
    PairsNextItemError,

    #[fail(display = "{}", _0)]
    ParseIntError(#[fail(cause)] ParseIntError),

    #[fail(display = "{}", _0)]
    PestError(#[fail(cause)] PestError<Rule>),
}

macro_rules! impl_from {
    ($e:ident $(<$t:ty>)*) => {
        impl From<$e $(<$t>)*> for Error {
            fn from(e: $e $(<$t>)*) -> Self {
                Error::$e(e)
            }
        }
    };
}

impl_from!(ParseIntError);
impl_from!(PestError<Rule>);
