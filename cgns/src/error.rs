/// Error type for CGNS operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CgnsError {
    /// CGNS returned a non-zero error code.
    CgnsCode(i32),
    /// Invalid argument or state (before calling CGNS).
    Invalid(String),
    /// File or node not found.
    NotFound(String),
    /// I/O error.
    Io(String),
}

impl std::fmt::Display for CgnsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CgnsCode(code) => write!(f, "CGNS error code: {}", code),
            Self::Invalid(msg) => write!(f, "invalid: {}", msg),
            Self::NotFound(msg) => write!(f, "not found: {}", msg),
            Self::Io(msg) => write!(f, "I/O: {}", msg),
        }
    }
}

impl std::error::Error for CgnsError {}

pub type CgnsResult<T> = Result<T, CgnsError>;
