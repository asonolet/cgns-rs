use std::fmt;

#[derive(Debug, Clone)]
pub enum CgnsError {
    CgnsCode(i32, String),
    Invalid(String),
    NotFound(String),
    Io(String),
}

impl fmt::Display for CgnsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CgnsCode(code, msg) => write!(f, "CGNS error (code {}): {}", code, msg),
            Self::Invalid(msg) => write!(f, "invalid: {}", msg),
            Self::NotFound(msg) => write!(f, "not found: {}", msg),
            Self::Io(msg) => write!(f, "I/O: {}", msg),
        }
    }
}

impl std::error::Error for CgnsError {}

impl From<String> for CgnsError {
    fn from(msg: String) -> Self {
        Self::CgnsCode(-1, msg)
    }
}

pub type CgnsResult<T> = Result<T, CgnsError>;

pub fn from_sys_result<T>(r: Result<T, String>) -> CgnsResult<T> {
    r.map_err(|msg| CgnsError::CgnsCode(-1, msg))
}

pub fn check_sys_status(status: i32) -> CgnsResult<()> {
    if status == cgns_sys::CG_OK as i32 {
        Ok(())
    } else {
        Err(CgnsError::CgnsCode(status, cgns_sys::error_message()))
    }
}
