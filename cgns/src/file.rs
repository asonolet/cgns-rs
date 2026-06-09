use crate::base::Base;
use crate::error::{check_sys_status, CgnsResult};

pub struct CgnsFile {
    pub(crate) fn_: i32,
    closed: bool,
}

impl CgnsFile {
    pub fn open(path: &str) -> CgnsResult<Self> {
        let fn_ = cgns_sys::open_read(path)?;
        Ok(Self { fn_, closed: false })
    }

    pub fn create(path: &str) -> CgnsResult<Self> {
        let fn_ = cgns_sys::open_write(path)?;
        Ok(Self { fn_, closed: false })
    }

    pub fn modify(path: &str) -> CgnsResult<Self> {
        let fn_ = cgns_sys::open_modify(path)?;
        Ok(Self { fn_, closed: false })
    }

    pub fn close(&mut self) -> CgnsResult<()> {
        if !self.closed {
            cgns_sys::close(self.fn_)?;
            self.closed = true;
        }
        Ok(())
    }

    pub fn create_base(&self, name: &str, cell_dim: i32, phys_dim: i32) -> CgnsResult<Base> {
        let index = cgns_sys::base_write(self.fn_, name, cell_dim, phys_dim)?;
        Ok(Base {
            file_fn: self.fn_,
            index,
            name: name.to_string(),
        })
    }

    pub fn base(&self, name: &str) -> CgnsResult<Base> {
        let n = self.base_count()?;
        for i in 1..=n {
            let _guard = cgns_sys::lock_cgns();
            let mut buf = vec![0u8; 64];
            let mut cell_dim: i32 = 0;
            let mut phys_dim: i32 = 0;
            let status = unsafe {
                cgns_sys::cg_base_read(
                    self.fn_,
                    i,
                    buf.as_mut_ptr() as *mut i8,
                    &mut cell_dim,
                    &mut phys_dim,
                )
            };
            check_sys_status(status)?;
            let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            let found = std::str::from_utf8(&buf[..end])
                .map_err(|e| crate::error::CgnsError::Invalid(e.to_string()))?;
            if found == name {
                return Ok(Base {
                    file_fn: self.fn_,
                    index: i,
                    name: name.to_string(),
                });
            }
        }
        Err(crate::error::CgnsError::NotFound(format!(
            "base '{}' not found",
            name
        )))
    }

    pub fn base_count(&self) -> CgnsResult<i32> {
        let _guard = cgns_sys::lock_cgns();
        let mut n: i32 = 0;
        let status = unsafe { cgns_sys::cg_nbases(self.fn_, &mut n) };
        check_sys_status(status)?;
        Ok(n)
    }

    pub fn bases(&self) -> CgnsResult<Vec<Base>> {
        let n = self.base_count()?;
        let mut bases = Vec::with_capacity(n as usize);
        for i in 1..=n {
            let _guard = cgns_sys::lock_cgns();
            let mut buf = vec![0u8; 64];
            let mut cell_dim: i32 = 0;
            let mut phys_dim: i32 = 0;
            let status = unsafe {
                cgns_sys::cg_base_read(
                    self.fn_,
                    i,
                    buf.as_mut_ptr() as *mut i8,
                    &mut cell_dim,
                    &mut phys_dim,
                )
            };
            check_sys_status(status)?;
            let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            let name = std::str::from_utf8(&buf[..end])
                .map_err(|e| crate::error::CgnsError::Invalid(e.to_string()))?;
            bases.push(Base {
                file_fn: self.fn_,
                index: i,
                name: name.to_string(),
            });
        }
        Ok(bases)
    }
}

impl Drop for CgnsFile {
    fn drop(&mut self) {
        if !self.closed {
            let _ = cgns_sys::close(self.fn_);
        }
    }
}
