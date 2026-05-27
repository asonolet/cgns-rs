use crate::error::{check_sys_status, from_sys_result, CgnsResult};
use crate::zone::Zone;

pub struct Base {
    pub(crate) file_fn: i32,
    pub(crate) index: i32,
}

impl Base {
    pub const fn index(&self) -> i32 {
        self.index
    }

    pub fn name(&self) -> CgnsResult<String> {
        let _guard = cgns_sys::lock_cgns()?;
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
        let _guard = cgns_sys::lock_cgns()?;
        let mut n: i32 = 0;
        let status = unsafe { cgns_sys::cg_nzones(self.file_fn, self.index, &mut n) };
        check_sys_status(status)?;
        Ok(n)
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
        let _guard = cgns_sys::lock_cgns()?;
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
