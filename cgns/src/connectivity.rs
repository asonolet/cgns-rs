use crate::error::CgnsResult;

/// An opaque handle to a 1-to-1 zone interface.
pub struct OneToOne {
    pub(crate) file_fn: i32,
    pub(crate) base_index: i32,
    pub(crate) zone_index: i32,
    pub(crate) index: i32,
}

impl OneToOne {
    pub const fn index(&self) -> i32 {
        self.index
    }

    /// Read 1-to-1 interface metadata.
    pub fn read(&self) -> CgnsResult<OneToOneInfo> {
        let _guard = cgns_sys::lock_cgns()?;
        let mut name_buf = vec![0u8; 64];
        let mut donor_buf = vec![0u8; 64];
        let mut range = vec![0i64; 6];
        let mut donor_range = vec![0i64; 6];
        let mut transform = vec![0i32; 3];
        cgns_sys::read_1to1(
            self.file_fn,
            self.base_index,
            self.zone_index,
            self.index,
            &mut name_buf,
            &mut donor_buf,
            &mut range,
            &mut donor_range,
            &mut transform,
        )?;
        let name_end = name_buf
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(name_buf.len());
        let name = std::str::from_utf8(&name_buf[..name_end])
            .map_err(|e| crate::error::CgnsError::Invalid(e.to_string()))?;
        let donor_end = donor_buf
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(donor_buf.len());
        let donor_name = std::str::from_utf8(&donor_buf[..donor_end])
            .map_err(|e| crate::error::CgnsError::Invalid(e.to_string()))?;
        Ok(OneToOneInfo {
            name: name.to_string(),
            donor_name: donor_name.to_string(),
            range,
            donor_range,
            transform,
        })
    }
}

/// Metadata describing a 1-to-1 zone interface.
#[derive(Debug, Clone)]
pub struct OneToOneInfo {
    pub name: String,
    pub donor_name: String,
    pub range: Vec<i64>,
    pub donor_range: Vec<i64>,
    pub transform: Vec<i32>,
}
