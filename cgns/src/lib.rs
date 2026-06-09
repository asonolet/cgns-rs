//! Safe, idiomatic Rust bindings for CGNS (CFD General Notation System).
//!
//! CGNS is the ISO 10303-209 standard for storing and retrieving computational
//! fluid dynamics (CFD) mesh and solution data.  The underlying storage is
//! HDF5; this crate hides those details and provides a high-level API.
//!
//! # Concepts
//!
//! | CGNS term | Rust type | What it is |
//! |---|---|---|
//! | File | [`CgnsFile`] | A `.cgns` file on disk (HDF5) |
//! | Base | [`Base`] | A top-level data group (cell_dim, phys_dim) |
//! | Zone | [`Zone`] | A mesh zone — structured (ijk) or unstructured |
//! | Section | [`Section`] | A group of elements of one type (unstructured only) |
//! | Solution | [`Solution`] | A set of field variables at a grid location |
//!
//! # Indexing conventions
//!
//! CGNS uses **1-based** indices everywhere.  Node IDs in connectivity
//! arrays start at **1**, not 0.  Element numbers, section start/end, and
//! structured zone ranges are also 1-based.
//!
//! # Connectivity layout (unstructured)
//!
//! Element connectivity is stored as a **flat** array of `i64` values.
//! For an element type with `NPE` vertices per element:
//!
//! ```text
//! [e1_v1, e1_v2, ..., e1_vNPE, e2_v1, e2_v2, ..., e2_vNPE, ...]
//! ```
//!
//! Each vertex reference is a **1-based** node index into the zone's vertex
//! list.  The convenience method
//! [`Section::read_connectivity_ndarray`](section/struct.Section.html#method.read_connectivity_ndarray)
//! (requires feature `ndarray`) reshapes this into an `ndarray::Array2<i64>` with shape `(num_elements, npe)`.
//!
//! # Structured zone indexing
//!
//! Structured zone data uses **Fortran (column-major) order**: the first index
//! varies fastest.  A `[ni, nj, nk]` vertex block has `ni*nj*nk` values laid
//! out as `(i, j, k)` with `i` as the innermost (fastest-varying) dimension.
//!
//! # Thread safety
//!
//! The underlying CGNS C library uses global state and is **not thread-safe**.
//! This crate serialises all FFI calls through a global mutex
//! ([`cgns_sys::lock_cgns`]).  Safe wrappers acquire the lock automatically;
//! raw FFI callers must acquire it themselves.
//!
//! # Quick start — unstructured mesh
//!
//! ```no_run
//! use cgns::data::ElementType;
//! use cgns::CgnsFile;
//! # use std::path::PathBuf;
//! # let path = "/tmp/example.cgns";
//!
//! // --- Write a triangle mesh ---
//! {
//!     let file = CgnsFile::create(path).expect("create file");
//!     let base = file.create_base("Base", 3, 3).expect("create base");
//!     let zone = base.create_zone_unstructured("Zone", 3, 1).expect("create zone");
//!
//!     // Flat connectivity: one triangle with 1-based vertex IDs [1, 2, 3]
//!     let elements = vec![1i64, 2, 3];
//!     let _section = zone.write_section("TriSection", ElementType::Tri3, 1, 1, 0, &elements)
//!         .expect("write section");
//! }
//!
//! // --- Read it back ---
//! {
//!     let file = CgnsFile::open(path).expect("open file");
//!     let zone = file.base("Base").expect("find base")
//!         .zones().expect("list zones")
//!         .into_iter().next().unwrap();
//!
//!     let sec = zone.section("TriSection").expect("find section");
//!     let conn = sec.read_connectivity().expect("read connectivity");
//!     assert_eq!(conn, vec![1i64, 2, 3]);
//! }
//! # std::fs::remove_file(path).ok();
//! ```

pub use cgns_sys;

pub mod data;
pub mod error;
pub mod util;

mod base;
mod bc;
mod connectivity;
mod file;
mod section;
mod zone;

pub use base::Base;
pub use bc::Bc;
pub use connectivity::{
    GeneralConnectivity, GeneralConnectivityData, GeneralConnectivityInfo, OneToOne,
};
pub use file::CgnsFile;
pub use section::Section;
pub use zone::{Solution, Zone};
