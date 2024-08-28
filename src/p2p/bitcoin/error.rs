pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error: {0}")]
    IO(#[from] std::io::Error),
    #[error("utf8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("array error: {0}")]
    TryFromSlice(#[from] std::array::TryFromSliceError),
    #[error("command error: {0}")]
    Command(String),
    #[error("not enough bytes to decode: {0}")]
    NotEnoughBytes(&'static str),
}
