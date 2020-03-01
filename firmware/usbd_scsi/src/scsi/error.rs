use packing::Error as PackingError_SEXY_NEW;
use packed_struct::PackingError;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    UnhandledOpCode,
    /// The identified opcode requires more data than was sent
    InsufficientDataForCommand,
    PackingError(PackingError),
    PackingError2(PackingError_SEXY_NEW),
}

impl From<PackingError> for Error {
    fn from(e: PackingError) -> Error {
        Error::PackingError(e)
    }
}

impl From<PackingError_SEXY_NEW> for Error {
    fn from(e: PackingError_SEXY_NEW) -> Error {
        Error::PackingError2(e)
    }
}