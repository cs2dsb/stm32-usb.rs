use packed_struct::PackingError;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    SignatureError,
    UnhandledOpCode,
    /// The identified opcode requires more data than was sent
    InsufficientDataForCommand,
    PackingError(PackingError),
}

impl From<PackingError> for Error {
    fn from(e: PackingError) -> Error {
        Error::PackingError(e)
    }
}