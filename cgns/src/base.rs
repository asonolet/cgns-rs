use crate::error::{check_sys_status, CgnsResult};
use crate::zone::Zone;

pub struct Base {
    pub(crate) file_fn: i32,
    pub(crate) index: i32,
}

impl Base {
    pub fn index(&self) -> i32 {
        self.index
    }

    pub fn name(&self) -> CgnsResult<String> {
        let _guard = cgns_sys::lock_cgns();
        let mut buf = vec![0u8; 64];
        let mut cell_dim: i32 = 0;
        let mut phys_dim: i32 = 0;
        let status = unsafe {
            cgns_sys::cg_base_read(
                self.file_fn,
                self.index,
                buf.as_mut_ptr() as *mut i8,
                &mut cell_dim,
                &mut phys_dim,
            )
        };
        check_sys_status(status)?;
        let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        Ok(std::str::from_utf8(&buf[..end])
            .map_err(|e| crate::error::CgnsError::Invalid(e.to_string()))?
            .to_string())
    }

    pub fn create_zone_structured(&self, name: &str, size: &[i64]) -> CgnsResult<Zone> {
        let index = cgns_sys::zone_write_structured(self.file_fn, self.index, name, size)?;
        Ok(Zone { file_fn: self.file_fn, base_index: self.index, index })
    }

    pub fn zone_count(&self) -> CgnsResult<i32> {
        let _guard = cgns_sys::lock_cgns();
        let mut n: i32 = 0;
        let status = unsafe { cgns_sys::cg_nzones(self.file_fn, self.index, &mut n) };
        check_sys_status(status)?;
        Ok(n)
    }

    pub fn zones(&self) -> CgnsResult<Vec<Zone>> {
        let n = self.zone_count()?;
        let mut zones = Vec::with_capacity(n as usize);
        for i in 1..=n {
            zones.push(Zone { file_fn: self.file_fn, base_index: self.index, index: i });
        }
        Ok(zones)
    }
}
