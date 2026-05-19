use std::fmt;

#[derive(Debug, Clone)]
pub enum CgnsError {
    CgnsCode(i32),
    Invalid(String),
    NotFound(String),
    Io(String),
}

impl fmt::Display for CgnsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CgnsCode(code) => {
                let msg = cgns_sys::error_message();
                write!(f, "CGNS error (code {}): {}", code, msg)
            }
            Self::Invalid(msg) => write!(f, "invalid: {}", msg),
            Self::NotFound(msg) => write!(f, "not found: {}", msg),
            Self::Io(msg) => write!(f, "I/O: {}", msg),
        }
    }
}

impl std::error::Error for CgnsError {}

impl From<String> for CgnsError {
    fn from(_msg: String) -> Self {
        Self::CgnsCode(-1)
    }
}

pub type CgnsResult<T> = Result<T, CgnsError>;

pub fn from_sys_result<T>(r: Result<T, String>) -> CgnsResult<T> {
    r.map_err(|_msg| CgnsError::CgnsCode(-1))
}

pub fn check_sys_status(status: i32) -> CgnsResult<()> {
    if status == cgns_sys::CG_OK as i32 {
        Ok(())
    } else {
        Err(CgnsError::CgnsCode(status))
    }
}
