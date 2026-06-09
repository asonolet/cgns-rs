use crate::data::ElementType;
use crate::error::CgnsResult;

/// An opaque handle to a section (element group) in an unstructured zone.
///
/// A section collects elements of a single [`ElementType`] (or `Mixed` for
/// heterogeneous element sets).  Sections are obtained from a [`Zone`] via
/// [`Zone::section`], [`Zone::sections`], or [`Zone::write_section`].
///
/// [`Zone`]: crate::zone::Zone
/// [`Zone::section`]: crate::zone::Zone::section
/// [`Zone::sections`]: crate::zone::Zone::sections
/// [`Zone::write_section`]: crate::zone::Zone::write_section
///
/// # Connectivity layout
///
/// CGNS stores element connectivity as a **flat** `Vec<i64>`.  For a fixed-NPE
/// type like `Tri3` (NPE = 3):
///
/// ```text
/// [elem1_v1, elem1_v2, elem1_v3, elem2_v1, elem2_v2, elem2_v3, ...]
/// ```
///
/// All vertex indices are **1-based** within the zone.
///
/// Use [`read_connectivity`](Self::read_connectivity) for the raw flat array,
/// or [`read_connectivity_ndarray`](Self::read_connectivity_ndarray) to get an
/// `ndarray::Array2<i64>` of shape `(num_elements, npe)`.
pub struct Section {
    pub(crate) file_fn: i32,
    pub(crate) base_index: i32,
    pub(crate) zone_index: i32,
    pub(crate) index: i32,
}

impl Section {
    /// The 1-based section index within its zone.
    pub const fn index(&self) -> i32 {
        self.index
    }

    /// Read the section metadata (name, element type, element range, etc.).
    pub fn info(&self) -> CgnsResult<SectionInfo> {
        let mut buf = vec![0u8; 64];
        let mut elem_type: u32 = 0;
        let mut start: i64 = 0;
        let mut end: i64 = 0;
        let mut nbndry: i32 = 0;
        let mut parent_flag: i32 = 0;
        cgns_sys::section_read(
            self.file_fn,
            self.base_index,
            self.zone_index,
            self.index,
            &mut buf,
            &mut elem_type,
            &mut start,
            &mut end,
            &mut nbndry,
            &mut parent_flag,
        )
        .map_err(crate::error::CgnsError::Invalid)?;
        let name = crate::util::read_c_string(&buf)?;
        Ok(SectionInfo {
            name: name.to_string(),
            element_type: ElementType::from_raw(elem_type).unwrap_or(ElementType::Null),
            start,
            end,
            nbndry,
            parent_flag,
        })
    }

    /// Read the element connectivity as a flat `Vec<i64>`.
    ///
    /// The returned vector contains the raw CGNS connectivity data.  See the
    /// [struct-level docs](Section#connectivity-layout) for the layout.
    pub fn read_connectivity(&self) -> CgnsResult<Vec<i64>> {
        let size =
            cgns_sys::element_data_size(self.file_fn, self.base_index, self.zone_index, self.index)
                .map_err(crate::error::CgnsError::Invalid)?;
        let mut elements = vec![0i64; size as usize];
        cgns_sys::elements_read(
            self.file_fn,
            self.base_index,
            self.zone_index,
            self.index,
            &mut elements,
        )
        .map_err(crate::error::CgnsError::Invalid)?;
        Ok(elements)
    }

    /// Return the number of elements in this section.
    ///
    /// This is simply `end - start + 1` from the section metadata.
    pub fn element_count(&self) -> CgnsResult<i32> {
        let info = self.info()?;
        Ok((info.end - info.start + 1) as i32)
    }

    /// Read connectivity and reshape into an `ndarray::Array2<i64>`.
    ///
    /// Returns an array of shape `(num_elements, NPE)` where NPE is the
    /// number of vertices per element for this element type.
    ///
    /// # Errors
    ///
    /// Returns `Invalid` if the section uses `Mixed` elements (variable NPE)
    /// or if the element type reports zero vertices per element.
    pub fn read_connectivity_ndarray(&self) -> CgnsResult<ndarray::Array2<i64>> {
        let info = self.info()?;
        if info.element_type == ElementType::Mixed {
            return Err(crate::error::CgnsError::Invalid(
                "cannot reshape Mixed element connectivity with fixed npe".into(),
            ));
        }
        let npe = info.element_type.npe() as usize;
        if npe == 0 {
            return Err(crate::error::CgnsError::Invalid(
                "cannot reshape element type with zero vertices per element".into(),
            ));
        }
        let flat = self.read_connectivity()?;
        let nelem = flat.len() / npe;
        if flat.len() != nelem * npe {
            return Err(crate::error::CgnsError::Invalid(
                "connectivity size not divisible by npe".into(),
            ));
        }
        ndarray::Array2::from_shape_vec((nelem, npe), flat)
            .map_err(|e| crate::error::CgnsError::Invalid(e.to_string()))
    }
}

/// Metadata describing a single section (element group).
///
/// Returned by [`Section::info`].
#[derive(Debug, Clone)]
pub struct SectionInfo {
    /// The section name (e.g. `"TriSection"`).
    pub name: String,
    /// The element type of this section.
    ///
    /// When the section was written as `Mixed`, this will be
    /// [`ElementType::Mixed`].
    pub element_type: ElementType,
    /// 1-based index of the first element in this section (within the zone).
    pub start: i64,
    /// 1-based index of the last element in this section (within the zone).
    ///
    /// The total number of elements is `end - start + 1`.
    pub end: i64,
    /// Number of boundary elements in this section.
    pub nbndry: i32,
    /// Whether parent data is available (0 = none, 1 = available).
    pub parent_flag: i32,
}
