use crate::data::{DataType, GridConnectivityType, GridLocation, PointSetType, ZoneType};
use crate::error::{CgnsError, CgnsResult};

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

/// An opaque handle to a general (non-1-to-1) zone interface connection.
pub struct GeneralConnectivity {
    pub(crate) file_fn: i32,
    pub(crate) base_index: i32,
    pub(crate) zone_index: i32,
    pub(crate) index: i32,
}

impl GeneralConnectivity {
    pub const fn index(&self) -> i32 {
        self.index
    }

    /// Read connection metadata.
    pub fn info(&self) -> CgnsResult<GeneralConnectivityInfo> {
        let mut name_buf = vec![0u8; 64];
        let mut donor_buf = vec![0u8; 64];
        let mut location: u32 = 0;
        let mut connect_type: u32 = 0;
        let mut ptset_type: u32 = 0;
        let mut npnts: i64 = 0;
        let mut donor_zonetype: u32 = 0;
        let mut donor_ptset_type: u32 = 0;
        let mut donor_datatype: u32 = 0;
        let mut ndata_donor: i64 = 0;
        cgns_sys::conn_info(
            self.file_fn,
            self.base_index,
            self.zone_index,
            self.index,
            &mut name_buf,
            &mut location,
            &mut connect_type,
            &mut ptset_type,
            &mut npnts,
            &mut donor_buf,
            &mut donor_zonetype,
            &mut donor_ptset_type,
            &mut donor_datatype,
            &mut ndata_donor,
        )
        .map_err(CgnsError::Invalid)?;
        let name = read_c_string(&name_buf);
        let donor_name = read_c_string(&donor_buf);
        Ok(GeneralConnectivityInfo {
            name,
            location: GridLocation::from_raw(location)
                .ok_or_else(|| CgnsError::Invalid("invalid grid location".into()))?,
            connection_type: GridConnectivityType::from_raw(connect_type)
                .ok_or_else(|| CgnsError::Invalid("invalid connectivity type".into()))?,
            point_set_type: PointSetType::from_raw(ptset_type)
                .ok_or_else(|| CgnsError::Invalid("invalid point set type".into()))?,
            num_points: npnts,
            donor_name,
            donor_zone_type: ZoneType::from_raw(donor_zonetype)
                .ok_or_else(|| CgnsError::Invalid("invalid zone type".into()))?,
            donor_point_set_type: PointSetType::from_raw(donor_ptset_type)
                .ok_or_else(|| CgnsError::Invalid("invalid donor point set type".into()))?,
            donor_data_type: DataType::from_raw(donor_datatype)
                .ok_or_else(|| CgnsError::Invalid("invalid donor data type".into()))?,
            num_donor_data: ndata_donor,
        })
    }

    /// Read connection point and donor data.
    ///
    /// Call [`info()`](Self::info) first to get buffer sizes via
    /// [`num_points`](GeneralConnectivityInfo::num_points) and
    /// [`num_donor_data`](GeneralConnectivityInfo::num_donor_data).
    /// The `index_dim` is the zone's index dimension.
    pub fn read_data(&self, index_dim: i32) -> CgnsResult<GeneralConnectivityData> {
        let info = self.info()?;
        let npts_vals = info.num_points as usize * index_dim as usize;
        let ndata_vals = info.num_donor_data as usize * index_dim as usize;
        let mut points = vec![0i64; npts_vals];
        let mut donor_data = vec![0i64; ndata_vals];
        cgns_sys::conn_read(
            self.file_fn,
            self.base_index,
            self.zone_index,
            self.index,
            &mut points,
            &mut donor_data,
        )
        .map_err(CgnsError::Invalid)?;
        Ok(GeneralConnectivityData {
            info,
            points,
            donor_data,
        })
    }
}

fn read_c_string(buf: &[u8]) -> String {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    std::str::from_utf8(&buf[..end])
        .map(|s| s.to_string())
        .unwrap_or_default()
}

/// Metadata describing a general (non-1-to-1) zone interface connection.
#[derive(Debug, Clone)]
pub struct GeneralConnectivityInfo {
    pub name: String,
    pub location: GridLocation,
    pub connection_type: GridConnectivityType,
    pub point_set_type: PointSetType,
    pub num_points: i64,
    pub donor_name: String,
    pub donor_zone_type: ZoneType,
    pub donor_point_set_type: PointSetType,
    pub donor_data_type: DataType,
    pub num_donor_data: i64,
}

/// Full read-back of a general connection, including point and donor data.
#[derive(Debug, Clone)]
pub struct GeneralConnectivityData {
    pub info: GeneralConnectivityInfo,
    pub points: Vec<i64>,
    pub donor_data: Vec<i64>,
}
