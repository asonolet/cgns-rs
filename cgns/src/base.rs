use crate::data::{
    AngleUnits, DataClass, LengthUnits, MassUnits, SimulationType, TemperatureUnits, TimeUnits,
    UnitsSystem,
};
use crate::error::{check_sys_status, from_sys_result, CgnsError, CgnsResult};
use crate::zone::Zone;

pub struct Base {
    pub(crate) file_fn: i32,
    pub(crate) index: i32,
    pub(crate) name: String,
}

impl Base {
    pub const fn index(&self) -> i32 {
        self.index
    }

    pub fn name(&self) -> CgnsResult<String> {
        Ok(self.name.clone())
    }

    /// Create a structured (ijk-ordered) zone.
    ///
    /// `size` is the CGNS zone-size array with 3 or 9 entries:
    /// - 3 entries: `[ni_v, nj_v, nk_v]` — vertex dimensions.
    /// - 9 entries: `[ni_v, nj_v, nk_v, ni_c, nj_c, nk_c, 0, 0, 0]` —
    ///   vertex dimensions followed by cell dimensions.
    ///
    /// Data is in **Fortran (column-major) order**: `i` varies fastest.
    pub fn create_zone_structured(&self, name: &str, size: &[i64]) -> CgnsResult<Zone> {
        let index = cgns_sys::zone_write_structured(self.file_fn, self.index, name, size)?;
        Ok(Zone {
            file_fn: self.file_fn,
            base_index: self.index,
            index,
        })
    }

    /// Create an unstructured zone.
    ///
    /// `num_vertices` is the number of mesh vertices (nodes) and
    /// `num_elements` is the total number of cells/elements across all
    /// sections.  The CGNS zone-size array is `[num_vertices, num_elements, 0]`.
    ///
    /// Add element connectivity with [`Zone::write_section`] after creation.
    pub fn create_zone_unstructured(
        &self,
        name: &str,
        num_vertices: i64,
        num_elements: i64,
    ) -> CgnsResult<Zone> {
        let size = [num_vertices, num_elements, 0];
        let index = cgns_sys::zone_write_unstructured(self.file_fn, self.index, name, &size)?;
        Ok(Zone {
            file_fn: self.file_fn,
            base_index: self.index,
            index,
        })
    }

    pub fn zone_count(&self) -> CgnsResult<i32> {
        cgns_sys::nzones(self.file_fn, self.index).map_err(CgnsError::from)
    }

    pub fn zones(&self) -> CgnsResult<Vec<Zone>> {
        let n = self.zone_count()?;
        let mut zones = Vec::with_capacity(n as usize);
        for i in 1..=n {
            zones.push(Zone {
                file_fn: self.file_fn,
                base_index: self.index,
                index: i,
            });
        }
        Ok(zones)
    }

    /// Return the cell dimension of this base (2 for 2D, 3 for 3D).
    pub fn cell_dim(&self) -> CgnsResult<i32> {
        cgns_sys::cell_dim(self.file_fn, self.index).map_err(CgnsError::from)
    }

    /// Return the physical dimension of this base.
    pub fn phys_dim(&self) -> CgnsResult<i32> {
        let mut cell_dim: i32 = 0;
        let mut phys_dim: i32 = 0;
        let mut buf = vec![0u8; 64];
        cgns_sys::base_read(
            self.file_fn,
            self.index,
            &mut buf,
            &mut cell_dim,
            &mut phys_dim,
        )
        .map_err(CgnsError::from)?;
        Ok(phys_dim)
    }

    /// Write base units (Mass, Length, Time, Temperature, Angle).
    pub fn write_units(&self, units: &UnitsSystem) -> CgnsResult<()> {
        let _guard = cgns_sys::lock_cgns()?;
        let c_path = std::ffi::CString::new(format!("/{}", self.name))
            .map_err(|e| crate::error::CgnsError::Invalid(e.to_string()))?;
        unsafe {
            check_sys_status(cgns_sys::cg_gopath(self.file_fn, c_path.as_ptr()))?;
            check_sys_status(cgns_sys::cg_units_write(
                units.mass.to_raw(),
                units.length.to_raw(),
                units.time.to_raw(),
                units.temperature.to_raw(),
                units.angle.to_raw(),
            ))
        }
    }

    /// Read base units.
    pub fn read_units(&self) -> CgnsResult<UnitsSystem> {
        let _guard = cgns_sys::lock_cgns()?;
        let c_path = std::ffi::CString::new(format!("/{}", self.name))
            .map_err(|e| crate::error::CgnsError::Invalid(e.to_string()))?;
        unsafe {
            check_sys_status(cgns_sys::cg_gopath(self.file_fn, c_path.as_ptr()))?;
            let mut mass: u32 = 0;
            let mut length: u32 = 0;
            let mut time: u32 = 0;
            let mut temperature: u32 = 0;
            let mut angle: u32 = 0;
            check_sys_status(cgns_sys::cg_units_read(
                &mut mass,
                &mut length,
                &mut time,
                &mut temperature,
                &mut angle,
            ))?;
            Ok(UnitsSystem {
                mass: MassUnits::from_raw(mass).unwrap_or(MassUnits::Null),
                length: LengthUnits::from_raw(length).unwrap_or(LengthUnits::Null),
                time: TimeUnits::from_raw(time).unwrap_or(TimeUnits::Null),
                temperature: TemperatureUnits::from_raw(temperature)
                    .unwrap_or(TemperatureUnits::Null),
                angle: AngleUnits::from_raw(angle).unwrap_or(AngleUnits::Null),
            })
        }
    }

    /// Write the data class for this base.
    pub fn write_dataclass(&self, dc: DataClass) -> CgnsResult<()> {
        let _guard = cgns_sys::lock_cgns()?;
        let c_path = std::ffi::CString::new(format!("/{}", self.name))
            .map_err(|e| crate::error::CgnsError::Invalid(e.to_string()))?;
        unsafe {
            check_sys_status(cgns_sys::cg_gopath(self.file_fn, c_path.as_ptr()))?;
            check_sys_status(cgns_sys::cg_dataclass_write(dc.to_raw()))
        }
    }

    /// Read the data class from this base.
    pub fn read_dataclass(&self) -> CgnsResult<DataClass> {
        let _guard = cgns_sys::lock_cgns()?;
        let c_path = std::ffi::CString::new(format!("/{}", self.name))
            .map_err(|e| crate::error::CgnsError::Invalid(e.to_string()))?;
        unsafe {
            check_sys_status(cgns_sys::cg_gopath(self.file_fn, c_path.as_ptr()))?;
            let mut dc: u32 = 0;
            check_sys_status(cgns_sys::cg_dataclass_read(&mut dc))?;
            Ok(DataClass::from_raw(dc).unwrap_or(DataClass::Null))
        }
    }

    /// Write the simulation type for this base.
    pub fn write_simulation_type(&self, st: SimulationType) -> CgnsResult<()> {
        cgns_sys::simulation_type_write(self.file_fn, self.index, st.to_raw())?;
        Ok(())
    }

    /// Read the simulation type from this base.
    pub fn read_simulation_type(&self) -> CgnsResult<SimulationType> {
        let mut raw: u32 = 0;
        cgns_sys::simulation_type_read(self.file_fn, self.index, &mut raw)?;
        Ok(SimulationType::from_raw(raw).unwrap_or(SimulationType::Null))
    }

    /// Return the number of families in this base.
    pub fn family_count(&self) -> CgnsResult<i32> {
        from_sys_result(cgns_sys::nfamilies(self.file_fn, self.index))
    }

    /// Write a family.
    pub fn write_family(&self, name: &str) -> CgnsResult<i32> {
        from_sys_result(cgns_sys::family_write(self.file_fn, self.index, name))
    }

    /// Return all family names in this base.
    pub fn family_names(&self) -> CgnsResult<Vec<String>> {
        let n = self.family_count()?;
        let mut names = Vec::with_capacity(n as usize);
        for i in 1..=n {
            let mut buf = vec![0u8; 64];
            cgns_sys::family_read(self.file_fn, self.index, i, &mut buf)?;
            let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            names.push(
                std::str::from_utf8(&buf[..end])
                    .map_err(|e| crate::error::CgnsError::Invalid(e.to_string()))?
                    .to_string(),
            );
        }
        Ok(names)
    }
}
