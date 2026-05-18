/// Represents an open CGNS file.
pub struct CgnsFile {
    // TODO: store CGNS file handle
}

impl CgnsFile {
    /// Open an existing CGNS file for reading.
    pub fn open(_path: &str) -> super::error::CgnsResult<Self> {
        todo!()
    }

    /// Create a new CGNS file.
    pub fn create(_path: &str) -> super::error::CgnsResult<Self> {
        todo!()
    }

    /// Close the file.
    pub fn close(self) -> super::error::CgnsResult<()> {
        todo!()
    }
}
