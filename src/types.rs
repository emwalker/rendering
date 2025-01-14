use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unknown error: {0}")]
    General(String),

    #[error("io: {0}")]
    IO(#[from] std::io::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("tree construction: {0}")]
    TreeConstruction(String),

    #[error("utf8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub type AttributeMap = HashMap<String, String>;
