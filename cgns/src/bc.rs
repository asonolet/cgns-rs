use crate::data::{BcType, PointSetType};
use crate::error::CgnsResult;

/// An opaque handle to a boundary condition on a zone.
pub struct Bc {
    pub(crate) file_fn: i32,
    pub(crate) base_index: i32,
    pub(crate) zone_index: i32,
    pub(crate) index: i32,
}

impl Bc {
    pub const fn index(&self) -> i32 {
        self.index
    }

    /// Read boundary condition metadata.
    pub fn info(&self) -> CgnsResult<BcInfo> {
        let _guard = cgns_sys::lock_cgns();
        let mut buf = vec![0u8; 64];
        let mut bocotype: u32 = 0;
        let mut ptset_type: u32 = 0;
        let mut npnts: i64 = 0;
        cgns_sys::boco_info(
            self.file_fn,
            self.base_index,
            self.zone_index,
            self.index,
            &mut buf,
            &mut bocotype,
            &mut ptset_type,
            &mut npnts,
        )?;
        let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        let name = std::str::from_utf8(&buf[..end])
            .map_err(|e| crate::error::CgnsError::Invalid(e.to_string()))?;
        Ok(BcInfo {
            name: name.to_string(),
            bc_type: BcType::from_raw(bocotype).unwrap_or(BcType::Null),
            point_set_type: PointSetType::from_raw(ptset_type).unwrap_or(PointSetType::PointList),
            num_points: npnts,
        })
    }

    /// Read the boundary condition point data.
    ///
    /// Returns the flat point indices array.
    pub fn read_points(&self) -> CgnsResult<Vec<i64>> {
        let info = self.info()?;
        let mut pnts = vec![0i64; info.num_points as usize];
        cgns_sys::boco_read(
            self.file_fn,
            self.base_index,
            self.zone_index,
            self.index,
            &mut pnts,
        )?;
        Ok(pnts)
    }
}

/// Metadata describing a boundary condition.
#[derive(Debug, Clone)]
pub struct BcInfo {
    pub name: String,
    pub bc_type: BcType,
    pub point_set_type: PointSetType,
    pub num_points: i64,
}
