use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodingGameError {
    #[error("Serialize error : {0}")]
    SerializeError(#[from] serde_json::Error),
    #[error("Reqwest error : {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("IO error : {0}")]
    IOError(#[from] std::io::Error),
    #[error("Tera error : {0}")]
    TeraError(#[from] tera::Error),
    #[error("Puzzle folder already exists : {0}")]
    PuzzleAlreadyExists(String),
    #[error("Asset access error")]
    AssetError(String),
    #[error("Utf8 conversion error")]
    Utf8Error(#[from] std::str::Utf8Error),
}
