/// `ErrorCode` represents the error conditions that can be reached during
/// a regular TFTP operation.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ErrorCode {
    /// Not defined, see error message (if any).
    NotDefined = 0,

    /// File not found.
    FileNotFound = 1,

    /// Access violation.
    AccessViolation = 2,

    /// Disk full or allocation exceeded.
    DiskFull = 3,

    /// Illegal TFTP operation.
    IllegalOperation = 4,

    /// Unknown transfer ID.
    UnknownTid = 5,

    /// File already exists.
    FileAlreadyExists = 6,

    /// No such user.
    NoSuchUser = 7,
}

impl From<ErrorCode> for String {
    fn from(code: ErrorCode) -> String {
        match code {
            ErrorCode::NotDefined => "Not defined".to_string(),
            ErrorCode::FileNotFound => "File not found".to_string(),
            ErrorCode::AccessViolation => "Access violation".to_string(),
            ErrorCode::DiskFull => "Disk full or allocation exceeded".to_string(),
            ErrorCode::IllegalOperation => "Illegal TFTP operation".to_string(),
            ErrorCode::UnknownTid => "Unknown transfer ID".to_string(),
            ErrorCode::FileAlreadyExists => "File already exists".to_string(),
            ErrorCode::NoSuchUser => "No such user".to_string(),
        }
    }
}

/// An `Error` packet is a courtesy packet that is sent prior to terminating
/// the TFTP connection due to an unrecoverable error.
#[derive(Debug)]
pub struct Error {
    /// An integer code that describes the error.
    pub code: ErrorCode,

    /// A human readable description of the error.
    pub message: String,
}
