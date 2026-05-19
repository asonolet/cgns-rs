use crate::data::ZoneType;
use crate::error::{check_sys_status, CgnsResult};

pub struct Zone {
    pub(crate) file_fn: i32,
    pub(crate) base_index: i32,
    pub(crate) index: i32,
}

impl Zone {
    pub fn index(&self) -> i32 {
        self.index
    }

    pub fn zone_type(&self) -> CgnsResult<ZoneType> {
        let _guard = cgns_sys::lock_cgns();
        let mut raw: u32 = 0;
        let status = unsafe { cgns_sys::cg_zone_type(self.file_fn, self.base_index, self.index, &mut raw) };
        check_sys_status(status)?;
        ZoneType::from_raw(raw).ok_or_else(|| crate::error::CgnsError::Invalid("unknown zone type".into()))
    }

    pub fn coord_count(&self) -> CgnsResult<i32> {
        let _guard = cgns_sys::lock_cgns();
        let mut n: i32 = 0;
        let status = unsafe { cgns_sys::cg_ncoords(self.file_fn, self.base_index, self.index, &mut n) };
        check_sys_status(status)?;
        Ok(n)
    }

    pub fn coord_names(&self) -> CgnsResult<Vec<String>> {
        let n = self.coord_count()?;
        let mut names = Vec::with_capacity(n as usize);
        let _guard = cgns_sys::lock_cgns();
        for i in 1..=n {
            let mut buf = vec![0u8; 64];
            let mut data_type: u32 = 0;
            let status = unsafe {
                cgns_sys::cg_coord_info(
                    self.file_fn,
                    self.base_index,
                    self.index,
                    i,
                    &mut data_type,
                    buf.as_mut_ptr() as *mut i8,
                )
            };
            check_sys_status(status)?;
            let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            names.push(
                std::str::from_utf8(&buf[..end])
                    .map_err(|e| crate::error::CgnsError::Invalid(e.to_string()))?
                    .to_string(),
            );
        }
        Ok(names)
    }

    pub fn write_coord_f64(&self, name: &str, data: &[f64]) -> CgnsResult<()> {
        cgns_sys::coord_write(
            self.file_fn,
            self.base_index,
            self.index,
            cgns_sys::DataType_t_RealDouble,
            name,
            data,
        )?;
        Ok(())
    }

    pub fn read_coord_f64(&self, name: &str, rmin: &[i64], rmax: &[i64], data: &mut [f64]) -> CgnsResult<()> {
        cgns_sys::coord_read(
            self.file_fn,
            self.base_index,
            self.index,
            cgns_sys::DataType_t_RealDouble,
            name,
            rmin,
            rmax,
            data,
        )?;
        Ok(())
    }

    pub fn solution_count(&self) -> CgnsResult<i32> {
        let _guard = cgns_sys::lock_cgns();
        let mut n: i32 = 0;
        let status = unsafe { cgns_sys::cg_nsols(self.file_fn, self.base_index, self.index, &mut n) };
        check_sys_status(status)?;
        Ok(n)
    }

    pub fn write_solution(&self, name: &str, location: crate::data::GridLocation) -> CgnsResult<Solution> {
        let index = cgns_sys::sol_write(self.file_fn, self.base_index, self.index, name, location.to_raw())?;
        Ok(Solution { file_fn: self.file_fn, base_index: self.base_index, zone_index: self.index, index })
    }

    pub fn solution(&self, name: &str) -> CgnsResult<Solution> {
        let n = self.solution_count()?;
        let _guard = cgns_sys::lock_cgns();
        for i in 1..=n {
            let mut buf = vec![0u8; 64];
            let mut location: u32 = 0;
            let status = unsafe {
                cgns_sys::cg_sol_info(
                    self.file_fn,
                    self.base_index,
                    self.index,
                    i,
                    buf.as_mut_ptr() as *mut i8,
                    &mut location,
                )
            };
            check_sys_status(status)?;
            let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            let found = std::str::from_utf8(&buf[..end])
                .map_err(|e| crate::error::CgnsError::Invalid(e.to_string()))?;
            if found == name {
                return Ok(Solution { file_fn: self.file_fn, base_index: self.base_index, zone_index: self.index, index: i });
            }
        }
        Err(crate::error::CgnsError::NotFound(format!("solution '{}' not found", name)))
    }

    pub fn solutions(&self) -> CgnsResult<Vec<Solution>> {
        let n = self.solution_count()?;
        let mut sols = Vec::with_capacity(n as usize);
        for i in 1..=n {
            sols.push(Solution { file_fn: self.file_fn, base_index: self.base_index, zone_index: self.index, index: i });
        }
        Ok(sols)
    }
}

pub struct Solution {
    pub(crate) file_fn: i32,
    pub(crate) base_index: i32,
    pub(crate) zone_index: i32,
    pub(crate) index: i32,
}

impl Solution {
    pub fn index(&self) -> i32 {
        self.index
    }

    pub fn field_count(&self) -> CgnsResult<i32> {
        let _guard = cgns_sys::lock_cgns();
        let mut n: i32 = 0;
        let status = unsafe {
            cgns_sys::cg_nfields(self.file_fn, self.base_index, self.zone_index, self.index, &mut n)
        };
        check_sys_status(status)?;
        Ok(n)
    }

    pub fn write_field_f64(&self, name: &str, data: &[f64]) -> CgnsResult<()> {
        cgns_sys::field_write(
            self.file_fn,
            self.base_index,
            self.zone_index,
            self.index,
            cgns_sys::DataType_t_RealDouble,
            name,
            data,
        )?;
        Ok(())
    }

    pub fn read_field_f64(
        &self,
        name: &str,
        rmin: &[i64],
        rmax: &[i64],
        data: &mut [f64],
    ) -> CgnsResult<()> {
        cgns_sys::field_read(
            self.file_fn,
            self.base_index,
            self.zone_index,
            self.index,
            cgns_sys::DataType_t_RealDouble,
            name,
            rmin,
            rmax,
            data,
        )?;
        Ok(())
    }
}
