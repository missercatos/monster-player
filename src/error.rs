use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] ureq::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Audio error: {0}")]
    Audio(String),

    #[error("API error: code={code}, msg={msg}")]
    Api { code: i32, msg: String },
}

pub type Result<T> = std::result::Result<T, Error>;
