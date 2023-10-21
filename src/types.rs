use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("tree construction test parsing: {0}")]
    TreeConstruction(String),

    #[error("problem reading io: {0}")]
    IO(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
