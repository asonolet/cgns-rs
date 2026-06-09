//! Utility functions for CGNS convenience.

use crate::error::CgnsError;

/// Extract a null-terminated C string from a `&[u8]` buffer.
///
/// CGNS writes names into fixed-size byte buffers with a trailing `\0`.
/// This helper finds the terminator and converts the prefix to `&str`.
pub(crate) fn read_c_string(buf: &[u8]) -> Result<&str, CgnsError> {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    std::str::from_utf8(&buf[..end]).map_err(|e| CgnsError::Invalid(e.to_string()))
}
