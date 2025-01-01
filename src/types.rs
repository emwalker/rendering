use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io: {0}")]
    IO(#[from] std::io::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("tree construction: {0}")]
    TreeConstruction(String),

    #[cfg(feature = "tl")]
    #[error("tl parse error: {0}")]
    TlParseError(#[from] tl::ParseError),
}

pub type Result<T> = std::result::Result<T, Error>;

pub type AttributeMap = HashMap<String, String>;
