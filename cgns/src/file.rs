use crate::base::Base;
use crate::error::CgnsResult;

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
        let n = cgns_sys::nbases(self.fn_)?;
        for i in 1..=n {
            let mut buf = vec![0u8; 64];
            let mut cell_dim: i32 = 0;
            let mut phys_dim: i32 = 0;
            cgns_sys::base_read(self.fn_, i, &mut buf, &mut cell_dim, &mut phys_dim)?;
            let found = crate::util::read_c_string(&buf)?;
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
        cgns_sys::nbases(self.fn_).map_err(crate::error::CgnsError::from)
    }

    pub fn bases(&self) -> CgnsResult<Vec<Base>> {
        let n = cgns_sys::nbases(self.fn_)?;
        let mut bases = Vec::with_capacity(n as usize);
        for i in 1..=n {
            let mut buf = vec![0u8; 64];
            let mut cell_dim: i32 = 0;
            let mut phys_dim: i32 = 0;
            cgns_sys::base_read(self.fn_, i, &mut buf, &mut cell_dim, &mut phys_dim)?;
            let name = crate::util::read_c_string(&buf)?;
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
