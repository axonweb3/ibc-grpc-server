use std::str::Utf8Error;

use derive_more::Display;
use ibc::core::ics24_host::error::ValidationError;

#[derive(Debug, Display)]
pub enum ServerError {
    ValidateIdentifier(ValidationError),
    FromUtf8(Utf8Error),
}

impl From<ServerError> for String {
    fn from(err: ServerError) -> Self {
        err.to_string()
    }
}
