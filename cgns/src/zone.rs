use crate::bc::Bc;
use crate::connectivity::{GeneralConnectivity, OneToOne};
use crate::data::{
    BcType, DataType, ElementType, GridConnectivityType, GridLocation, PointSetType, ZoneType,
};
use crate::error::{check_sys_status, from_sys_result, CgnsResult};
use crate::section::Section;

/// An opaque handle to a zone (mesh block) within a CGNS base.
///
/// A zone is either **structured** (ijk-ordered vertices) or **unstructured**
/// (vertices + element connectivity).  Create zones via
/// [`Base::create_zone_structured`] or [`Base::create_zone_unstructured`].
///
/// [`Base::create_zone_structured`]: crate::base::Base::create_zone_structured
/// [`Base::create_zone_unstructured`]: crate::base::Base::create_zone_unstructured
pub struct Zone {
    pub(crate) file_fn: i32,
    pub(crate) base_index: i32,
    pub(crate) index: i32,
}

impl Zone {
    pub const fn index(&self) -> i32 {
        self.index
    }

    /// Return the zone size array: `[num_vertices, num_elements, ...]`.
    /// For structured zones this returns the full 9-entry size array.
    pub fn size(&self) -> CgnsResult<Vec<i64>> {
        let _guard = cgns_sys::lock_cgns();
        // Query index_dim internally (avoid calling self.index_dim() which acquires the mutex)
        let mut index_dim: i32 = 0;
        let status = unsafe {
            cgns_sys::cg_index_dim(self.file_fn, self.base_index, self.index, &mut index_dim)
        };
        check_sys_status(status)?;
        let nsize_entries = index_dim as usize * 3;
        let mut size = vec![0i64; nsize_entries];
        let mut buf = vec![0u8; 64];
        let status = unsafe {
            cgns_sys::cg_zone_read(
                self.file_fn,
                self.base_index,
                self.index,
                buf.as_mut_ptr() as *mut i8,
                size.as_mut_ptr(),
            )
        };
        check_sys_status(status)?;
        Ok(size)
    }

    /// Return the index dimension of this zone.
    /// For structured zones this is 1, 2, or 3. For unstructured zones it is 1.
    pub fn index_dim(&self) -> CgnsResult<i32> {
        cgns_sys::index_dim(self.file_fn, self.base_index, self.index)
            .map_err(crate::error::CgnsError::Invalid)
    }

    pub fn zone_type(&self) -> CgnsResult<ZoneType> {
        let raw = cgns_sys::zone_type(self.file_fn, self.base_index, self.index)
            .map_err(crate::error::CgnsError::Invalid)?;
        ZoneType::from_raw(raw)
            .ok_or_else(|| crate::error::CgnsError::Invalid("unknown zone type".into()))
    }

    pub fn coord_count(&self) -> CgnsResult<i32> {
        cgns_sys::ncoords(self.file_fn, self.base_index, self.index)
            .map_err(crate::error::CgnsError::Invalid)
    }

    pub fn coord_names(&self) -> CgnsResult<Vec<String>> {
        let n = self.coord_count()?;
        let mut names = Vec::with_capacity(n as usize);
        for i in 1..=n {
            let mut buf = vec![0u8; 64];
            let mut data_type: u32 = 0;
            cgns_sys::coord_info(
                self.file_fn,
                self.base_index,
                self.index,
                i,
                &mut data_type,
                &mut buf,
            )
            .map_err(crate::error::CgnsError::Invalid)?;
            let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            names.push(
                std::str::from_utf8(&buf[..end])
                    .map_err(|e| crate::error::CgnsError::Invalid(e.to_string()))?
                    .to_string(),
            );
        }
        Ok(names)
    }

    /// Write 64-bit float coordinate data (structured zones).
    ///
    /// `data` is a flat slice in **Fortran (column-major) order**:
    /// `i` varies fastest, then `j`, then `k`.  The length must equal the
    /// total vertex count (`ni * nj * nk`).
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

    /// Read a sub-range of 64-bit float coordinate data (structured zones).
    ///
    /// `rmin` and `rmax` are 1-based index ranges `[i_min, j_min, k_min]`
    /// and `[i_max, j_max, k_max]`.  The output `data` slice length must
    /// equal `(i_max - i_min + 1) * (j_max - j_min + 1) * (k_max - k_min + 1)`.
    ///
    /// Data is returned in **Fortran (column-major) order**.
    pub fn read_coord_f64(
        &self,
        name: &str,
        rmin: &[i64],
        rmax: &[i64],
        data: &mut [f64],
    ) -> CgnsResult<()> {
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

    fn read_ranges_size(rmin: &[i64], rmax: &[i64]) -> usize {
        rmin.iter()
            .zip(rmax)
            .map(|(&a, &b)| (b - a + 1) as usize)
            .product()
    }

    /// Read coordinate data and return an owned `Vec<f64>`.
    pub fn read_coord_f64_vec(
        &self,
        name: &str,
        rmin: &[i64],
        rmax: &[i64],
    ) -> CgnsResult<Vec<f64>> {
        let n = Self::read_ranges_size(rmin, rmax);
        let mut data = vec![0.0; n];
        self.read_coord_f64(name, rmin, rmax, &mut data)?;
        Ok(data)
    }

    /// Read coordinate data and return an owned `Vec<f32>`.
    pub fn read_coord_f32_vec(
        &self,
        name: &str,
        rmin: &[i64],
        rmax: &[i64],
    ) -> CgnsResult<Vec<f32>> {
        let n = Self::read_ranges_size(rmin, rmax);
        let mut data = vec![0.0f32; n];
        cgns_sys::coord_read_f32(
            self.file_fn,
            self.base_index,
            self.index,
            name,
            rmin,
            rmax,
            &mut data,
        )
        .map_err(crate::error::CgnsError::Invalid)?;
        Ok(data)
    }

    /// Read coordinate data and return an owned `Vec<i32>`.
    pub fn read_coord_i32_vec(
        &self,
        name: &str,
        rmin: &[i64],
        rmax: &[i64],
    ) -> CgnsResult<Vec<i32>> {
        let n = Self::read_ranges_size(rmin, rmax);
        let mut data = vec![0i32; n];
        cgns_sys::coord_read_i32(
            self.file_fn,
            self.base_index,
            self.index,
            name,
            rmin,
            rmax,
            &mut data,
        )
        .map_err(crate::error::CgnsError::Invalid)?;
        Ok(data)
    }

    /// Read coordinate data and return an owned `Vec<i64>`.
    pub fn read_coord_i64_vec(
        &self,
        name: &str,
        rmin: &[i64],
        rmax: &[i64],
    ) -> CgnsResult<Vec<i64>> {
        let n = Self::read_ranges_size(rmin, rmax);
        let mut data = vec![0i64; n];
        cgns_sys::coord_read_i64(
            self.file_fn,
            self.base_index,
            self.index,
            name,
            rmin,
            rmax,
            &mut data,
        )
        .map_err(crate::error::CgnsError::Invalid)?;
        Ok(data)
    }

    pub fn solution_count(&self) -> CgnsResult<i32> {
        cgns_sys::nsols(self.file_fn, self.base_index, self.index)
            .map_err(crate::error::CgnsError::Invalid)
    }

    pub fn write_solution(
        &self,
        name: &str,
        location: crate::data::GridLocation,
    ) -> CgnsResult<Solution> {
        let index = cgns_sys::sol_write(
            self.file_fn,
            self.base_index,
            self.index,
            name,
            location.to_raw(),
        )?;
        Ok(Solution {
            file_fn: self.file_fn,
            base_index: self.base_index,
            zone_index: self.index,
            index,
        })
    }

    pub fn solution(&self, name: &str) -> CgnsResult<Solution> {
        let n = self.solution_count()?;
        for i in 1..=n {
            let mut buf = vec![0u8; 64];
            let mut location: u32 = 0;
            cgns_sys::sol_info(
                self.file_fn,
                self.base_index,
                self.index,
                i,
                &mut buf,
                &mut location,
            )
            .map_err(crate::error::CgnsError::Invalid)?;
            let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            let found = std::str::from_utf8(&buf[..end])
                .map_err(|e| crate::error::CgnsError::Invalid(e.to_string()))?;
            if found == name {
                return Ok(Solution {
                    file_fn: self.file_fn,
                    base_index: self.base_index,
                    zone_index: self.index,
                    index: i,
                });
            }
        }
        Err(crate::error::CgnsError::NotFound(format!(
            "solution '{}' not found",
            name
        )))
    }

    pub fn solutions(&self) -> CgnsResult<Vec<Solution>> {
        let n = self.solution_count()?;
        let mut sols = Vec::with_capacity(n as usize);
        for i in 1..=n {
            sols.push(Solution {
                file_fn: self.file_fn,
                base_index: self.base_index,
                zone_index: self.index,
                index: i,
            });
        }
        Ok(sols)
    }

    /// Return the number of sections (element groups) in this zone.
    pub fn section_count(&self) -> CgnsResult<i32> {
        from_sys_result(cgns_sys::nsections(
            self.file_fn,
            self.base_index,
            self.index,
        ))
    }

    /// Return all sections in this zone.
    ///
    /// Sections are indexed 1-based from the CGNS file.
    pub fn sections(&self) -> CgnsResult<Vec<Section>> {
        let n = self.section_count()?;
        let mut secs = Vec::with_capacity(n as usize);
        for i in 1..=n {
            secs.push(Section {
                file_fn: self.file_fn,
                base_index: self.base_index,
                zone_index: self.index,
                index: i,
            });
        }
        Ok(secs)
    }

    /// Find a section by name.
    ///
    /// Returns `NotFound` if no section with that name exists.
    pub fn section(&self, name: &str) -> CgnsResult<Section> {
        let n = self.section_count()?;
        for i in 1..=n {
            let mut buf = vec![0u8; 64];
            let mut elem_type: u32 = 0;
            let mut start: i64 = 0;
            let mut end: i64 = 0;
            let mut nbndry: i32 = 0;
            let mut parent_flag: i32 = 0;
            cgns_sys::section_read(
                self.file_fn,
                self.base_index,
                self.index,
                i,
                &mut buf,
                &mut elem_type,
                &mut start,
                &mut end,
                &mut nbndry,
                &mut parent_flag,
            )
            .map_err(crate::error::CgnsError::Invalid)?;
            let name_end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            let found = std::str::from_utf8(&buf[..name_end])
                .map_err(|e| crate::error::CgnsError::Invalid(e.to_string()))?;
            if found == name {
                return Ok(Section {
                    file_fn: self.file_fn,
                    base_index: self.base_index,
                    zone_index: self.index,
                    index: i,
                });
            }
        }
        Err(crate::error::CgnsError::NotFound(format!(
            "section '{}' not found",
            name
        )))
    }

    /// Write an element section to this unstructured zone.
    ///
    /// `start` and `end` are the 1-based element numbers within the zone.
    /// For a single section containing all elements, use `start = 1` and
    /// `end = num_elements`.
    ///
    /// `elements` is the flat connectivity array (see
    /// [`Section`](crate::section::Section) for the layout).  Node indices
    /// are **1-based** vertex IDs.
    ///
    /// Returns the newly written [`Section`] handle.
    pub fn write_section(
        &self,
        name: &str,
        elem_type: ElementType,
        start: i64,
        end: i64,
        nbndry: i32,
        elements: &[i64],
    ) -> CgnsResult<Section> {
        let idx = from_sys_result(cgns_sys::section_write(
            self.file_fn,
            self.base_index,
            self.index,
            name,
            elem_type.to_raw(),
            start,
            end,
            nbndry,
            elements,
        ))?;
        Ok(Section {
            file_fn: self.file_fn,
            base_index: self.base_index,
            zone_index: self.index,
            index: idx,
        })
    }

    /// Return the number of boundary conditions on this zone.
    pub fn bc_count(&self) -> CgnsResult<i32> {
        from_sys_result(cgns_sys::nbocos(self.file_fn, self.base_index, self.index))
    }

    /// Write a boundary condition.
    ///
    /// `npnts` is the number of point-set entries:
    /// - For `PointRange`: number of ranges (typically `1`)
    /// - For `PointList`: number of points
    ///
    /// `points` are the flat point/range indices (1-based).
    pub fn write_bc(
        &self,
        name: &str,
        bc_type: BcType,
        point_set: PointSetType,
        npnts: i64,
        points: &[i64],
    ) -> CgnsResult<Bc> {
        let idx = from_sys_result(cgns_sys::boco_write(
            self.file_fn,
            self.base_index,
            self.index,
            name,
            bc_type.to_raw(),
            point_set.to_raw(),
            npnts,
            points,
        ))?;
        Ok(Bc {
            file_fn: self.file_fn,
            base_index: self.base_index,
            zone_index: self.index,
            index: idx,
        })
    }

    /// Return all boundary conditions on this zone.
    pub fn bcs(&self) -> CgnsResult<Vec<Bc>> {
        let n = self.bc_count()?;
        let mut bcs = Vec::with_capacity(n as usize);
        for i in 1..=n {
            bcs.push(Bc {
                file_fn: self.file_fn,
                base_index: self.base_index,
                zone_index: self.index,
                index: i,
            });
        }
        Ok(bcs)
    }

    /// Find a boundary condition by name.
    pub fn bc(&self, name: &str) -> CgnsResult<Bc> {
        let all = self.bcs()?;
        for bc in &all {
            let info = bc.info()?;
            if info.name == name {
                return Ok(Bc {
                    file_fn: self.file_fn,
                    base_index: self.base_index,
                    zone_index: self.index,
                    index: bc.index,
                });
            }
        }
        Err(crate::error::CgnsError::NotFound(format!(
            "BC '{}' not found",
            name
        )))
    }

    /// Return the number of 1-to-1 zone interfaces.
    pub fn n1to1(&self) -> CgnsResult<i32> {
        from_sys_result(cgns_sys::n1to1(self.file_fn, self.base_index, self.index))
    }

    /// Write a 1-to-1 zone interface.
    pub fn write_1to1(
        &self,
        name: &str,
        donor_name: &str,
        range: &[i64],
        donor_range: &[i64],
        transform: &[i32],
    ) -> CgnsResult<OneToOne> {
        let idx = from_sys_result(cgns_sys::write_1to1(
            self.file_fn,
            self.base_index,
            self.index,
            name,
            donor_name,
            range,
            donor_range,
            transform,
        ))?;
        Ok(OneToOne {
            file_fn: self.file_fn,
            base_index: self.base_index,
            zone_index: self.index,
            index: idx,
        })
    }

    /// Return all 1-to-1 zone interfaces.
    pub fn one_to_ones(&self) -> CgnsResult<Vec<OneToOne>> {
        let n = self.n1to1()?;
        let mut conns = Vec::with_capacity(n as usize);
        for i in 1..=n {
            conns.push(OneToOne {
                file_fn: self.file_fn,
                base_index: self.base_index,
                zone_index: self.index,
                index: i,
            });
        }
        Ok(conns)
    }

    /// Return the number of general (non-1-to-1) zone interface connections.
    pub fn nconns(&self) -> CgnsResult<i32> {
        cgns_sys::nconns(self.file_fn, self.base_index, self.index)
            .map_err(crate::error::CgnsError::Invalid)
    }

    /// Find a general connection by name.
    pub fn conn(&self, name: &str) -> CgnsResult<GeneralConnectivity> {
        let n = self.nconns()?;
        for i in 1..=n {
            let mut buf = vec![0u8; 64];
            let mut location: u32 = 0;
            let mut connect_type: u32 = 0;
            let mut ptset_type: u32 = 0;
            let mut npnts: i64 = 0;
            let mut donor_buf = vec![0u8; 64];
            let mut donor_zonetype: u32 = 0;
            let mut donor_ptset_type: u32 = 0;
            let mut donor_datatype: u32 = 0;
            let mut ndata_donor: i64 = 0;
            cgns_sys::conn_info(
                self.file_fn,
                self.base_index,
                self.index,
                i,
                &mut buf,
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
            .map_err(crate::error::CgnsError::Invalid)?;
            let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            let found = std::str::from_utf8(&buf[..end])
                .map_err(|e| crate::error::CgnsError::Invalid(e.to_string()))?;
            if found == name {
                return Ok(GeneralConnectivity {
                    file_fn: self.file_fn,
                    base_index: self.base_index,
                    zone_index: self.index,
                    index: i,
                });
            }
        }
        Err(crate::error::CgnsError::NotFound(format!(
            "connection '{}' not found",
            name
        )))
    }

    /// Return all general (non-1-to-1) zone interface connections.
    pub fn general_connectivities(&self) -> CgnsResult<Vec<GeneralConnectivity>> {
        let n = self.nconns()?;
        let mut conns = Vec::with_capacity(n as usize);
        for i in 1..=n {
            conns.push(GeneralConnectivity {
                file_fn: self.file_fn,
                base_index: self.base_index,
                zone_index: self.index,
                index: i,
            });
        }
        Ok(conns)
    }

    /// Write a general (non-1-to-1) zone interface connection.
    ///
    /// `npnts` is the **number of points** in the point set (e.g. `2` for a
    /// `PointRange` in 3-D).  `ndata_donor` is the **number of donor points**
    /// (the CGNS library internally multiplies by the index dimension).
    #[allow(clippy::too_many_arguments)]
    pub fn write_conn(
        &self,
        name: &str,
        location: GridLocation,
        connection_type: GridConnectivityType,
        point_set_type: PointSetType,
        npnts: i64,
        points: &[i64],
        donor_name: &str,
        donor_zone_type: ZoneType,
        donor_point_set_type: PointSetType,
        ndata_donor: i64,
        donor_data: &[i64],
    ) -> CgnsResult<GeneralConnectivity> {
        let idx = cgns_sys::conn_write(
            self.file_fn,
            self.base_index,
            self.index,
            name,
            location.to_raw(),
            connection_type.to_raw(),
            point_set_type.to_raw(),
            npnts,
            points,
            donor_name,
            donor_zone_type.to_raw(),
            donor_point_set_type.to_raw(),
            cgns_sys::DataType_t_LongInteger,
            ndata_donor,
            donor_data,
        )
        .map_err(crate::error::CgnsError::Invalid)?;
        Ok(GeneralConnectivity {
            file_fn: self.file_fn,
            base_index: self.base_index,
            zone_index: self.index,
            index: idx,
        })
    }

    /// Write grid coordinates with arbitrary data type.
    pub fn write_coord_f32(&self, name: &str, data: &[f32]) -> CgnsResult<()> {
        cgns_sys::coord_write_f32(self.file_fn, self.base_index, self.index, name, data)
            .map_err(crate::error::CgnsError::Invalid)?;
        Ok(())
    }

    /// Write a flow solution field (32-bit float).
    pub fn write_field_f32(&self, sol: &Solution, name: &str, data: &[f32]) -> CgnsResult<()> {
        cgns_sys::field_write_f32(
            self.file_fn,
            self.base_index,
            self.index,
            sol.index,
            name,
            data,
        )
        .map_err(crate::error::CgnsError::Invalid)?;
        Ok(())
    }

    /// Write a flow solution field (32-bit integer).
    pub fn write_field_i32(&self, sol: &Solution, name: &str, data: &[i32]) -> CgnsResult<()> {
        cgns_sys::field_write_i32(
            self.file_fn,
            self.base_index,
            self.index,
            sol.index,
            name,
            data,
        )
        .map_err(crate::error::CgnsError::Invalid)?;
        Ok(())
    }

    /// Write a flow solution field (64-bit integer).
    pub fn write_field_i64(&self, sol: &Solution, name: &str, data: &[i64]) -> CgnsResult<()> {
        cgns_sys::field_write_i64(
            self.file_fn,
            self.base_index,
            self.index,
            sol.index,
            name,
            data,
        )
        .map_err(crate::error::CgnsError::Invalid)?;
        Ok(())
    }
}

/// An opaque handle to a solution node in a zone.
///
/// A solution groups field variables (e.g. pressure, density, velocity)
/// that share a [`GridLocation`](crate::data::GridLocation).
///
/// Obtain a solution via [`Zone::write_solution`] or [`Zone::solution`].
pub struct Solution {
    pub(crate) file_fn: i32,
    pub(crate) base_index: i32,
    pub(crate) zone_index: i32,
    pub(crate) index: i32,
}

/// Metadata describing a solution node.
#[derive(Debug, Clone)]
pub struct SolutionInfo {
    /// The solution name (e.g. "FlowSolution").
    pub name: String,
    /// Where the solution data is stored (Vertex, CellCenter, etc.).
    pub location: GridLocation,
}

/// Metadata describing a field variable.
#[derive(Debug, Clone)]
pub struct FieldInfo {
    /// The field name (e.g. "Pressure", "Density").
    pub name: String,
    /// The data type of the field.
    pub data_type: DataType,
}

impl Solution {
    pub const fn index(&self) -> i32 {
        self.index
    }

    /// Read solution metadata (name and grid location).
    pub fn info(&self) -> CgnsResult<SolutionInfo> {
        let mut buf = vec![0u8; 64];
        let mut location: u32 = 0;
        cgns_sys::sol_info(
            self.file_fn,
            self.base_index,
            self.zone_index,
            self.index,
            &mut buf,
            &mut location,
        )
        .map_err(crate::error::CgnsError::Invalid)?;
        let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        let name = std::str::from_utf8(&buf[..end])
            .map_err(|e| crate::error::CgnsError::Invalid(e.to_string()))?;
        Ok(SolutionInfo {
            name: name.to_string(),
            location: GridLocation::from_raw(location).unwrap_or(GridLocation::Null),
        })
    }

    /// Return the names of all field variables in this solution.
    pub fn field_names(&self) -> CgnsResult<Vec<String>> {
        let n = self.field_count()?;
        let mut names = Vec::with_capacity(n as usize);
        for i in 1..=n {
            let mut buf = vec![0u8; 64];
            let mut data_type: u32 = 0;
            cgns_sys::field_info(
                self.file_fn,
                self.base_index,
                self.zone_index,
                self.index,
                i,
                &mut data_type,
                &mut buf,
            )
            .map_err(crate::error::CgnsError::Invalid)?;
            let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            names.push(
                std::str::from_utf8(&buf[..end])
                    .map_err(|e| crate::error::CgnsError::Invalid(e.to_string()))?
                    .to_string(),
            );
        }
        Ok(names)
    }

    /// Return info for each field (name + data type).
    pub fn field_info_list(&self) -> CgnsResult<Vec<FieldInfo>> {
        let n = self.field_count()?;
        let mut infos = Vec::with_capacity(n as usize);
        for i in 1..=n {
            let mut buf = vec![0u8; 64];
            let mut data_type: u32 = 0;
            cgns_sys::field_info(
                self.file_fn,
                self.base_index,
                self.zone_index,
                self.index,
                i,
                &mut data_type,
                &mut buf,
            )
            .map_err(crate::error::CgnsError::Invalid)?;
            let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            let name = std::str::from_utf8(&buf[..end])
                .map_err(|e| crate::error::CgnsError::Invalid(e.to_string()))?;
            infos.push(FieldInfo {
                name: name.to_string(),
                data_type: DataType::from_raw(data_type).unwrap_or(DataType::R8),
            });
        }
        Ok(infos)
    }

    pub fn field_count(&self) -> CgnsResult<i32> {
        cgns_sys::nfields(self.file_fn, self.base_index, self.zone_index, self.index)
            .map_err(crate::error::CgnsError::Invalid)
    }

    /// Write a 64-bit float field variable.
    ///
    /// `data` is flat in the same ordering as the zone's coordinates
    /// (Fortran order for structured zones, compact array for unstructured).
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

    /// Read a sub-range of a 64-bit float field variable (structured zones).
    ///
    /// `rmin` / `rmax` specify the 1-based index range.  See
    /// [`Zone::read_coord_f64`] for details.
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

    fn read_field_range_size(rmin: &[i64], rmax: &[i64]) -> usize {
        rmin.iter()
            .zip(rmax)
            .map(|(&a, &b)| (b - a + 1) as usize)
            .product()
    }

    /// Read a field and return an owned `Vec<f64>`.
    pub fn read_field_f64_vec(
        &self,
        name: &str,
        rmin: &[i64],
        rmax: &[i64],
    ) -> CgnsResult<Vec<f64>> {
        let n = Self::read_field_range_size(rmin, rmax);
        let mut data = vec![0.0; n];
        self.read_field_f64(name, rmin, rmax, &mut data)?;
        Ok(data)
    }

    /// Read a field and return an owned `Vec<f32>`.
    pub fn read_field_f32_vec(
        &self,
        name: &str,
        rmin: &[i64],
        rmax: &[i64],
    ) -> CgnsResult<Vec<f32>> {
        let n = Self::read_field_range_size(rmin, rmax);
        let mut data = vec![0.0f32; n];
        cgns_sys::field_read_f32(
            self.file_fn,
            self.base_index,
            self.zone_index,
            self.index,
            name,
            rmin,
            rmax,
            &mut data,
        )
        .map_err(crate::error::CgnsError::Invalid)?;
        Ok(data)
    }

    /// Read a field and return an owned `Vec<i32>`.
    pub fn read_field_i32_vec(
        &self,
        name: &str,
        rmin: &[i64],
        rmax: &[i64],
    ) -> CgnsResult<Vec<i32>> {
        let n = Self::read_field_range_size(rmin, rmax);
        let mut data = vec![0i32; n];
        cgns_sys::field_read_i32(
            self.file_fn,
            self.base_index,
            self.zone_index,
            self.index,
            name,
            rmin,
            rmax,
            &mut data,
        )
        .map_err(crate::error::CgnsError::Invalid)?;
        Ok(data)
    }

    /// Read a field and return an owned `Vec<i64>`.
    pub fn read_field_i64_vec(
        &self,
        name: &str,
        rmin: &[i64],
        rmax: &[i64],
    ) -> CgnsResult<Vec<i64>> {
        let n = Self::read_field_range_size(rmin, rmax);
        let mut data = vec![0i64; n];
        cgns_sys::field_read_i64(
            self.file_fn,
            self.base_index,
            self.zone_index,
            self.index,
            name,
            rmin,
            rmax,
            &mut data,
        )
        .map_err(crate::error::CgnsError::Invalid)?;
        Ok(data)
    }
}
