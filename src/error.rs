use std::fmt;
use std::io::Error;

/// Errors which occured when parsing the file
#[derive(Debug)]
pub enum TiledError {
    /// An error occured when decompressing using the
    /// [flate2](https://github.com/alexcrichton/flate2-rs) crate.
    DecompressingError(Error),
    ParsingError(serde_json::error::Error),
    Base64DecodingError(base64::DecodeError),
    Other(String),
}

impl fmt::Display for TiledError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            TiledError::DecompressingError(ref e) => write!(fmt, "{}", e),
            TiledError::ParsingError(ref e) => write!(fmt, "{}", e),
            TiledError::Base64DecodingError(ref e) => write!(fmt, "{}", e),
            TiledError::Other(ref s) => write!(fmt, "{}", s),
        }
    }
}

// This is a skeleton implementation, which should probably be extended in the future.
impl std::error::Error for TiledError {
    fn description(&self) -> &str {
        match *self {
            TiledError::DecompressingError(ref e) => e.description(),
            TiledError::ParsingError(ref e) => e.description(),
            TiledError::Base64DecodingError(ref e) => e.description(),
            TiledError::Other(ref s) => s.as_ref(),
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            TiledError::ParsingError(ref e) => Some(e as &dyn std::error::Error),
            TiledError::DecompressingError(ref e) => Some(e as &dyn std::error::Error),
            TiledError::Base64DecodingError(ref e) => Some(e as &dyn std::error::Error),
            _ => None,
        }
    }
}
