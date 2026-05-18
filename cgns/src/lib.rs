//! High-level, safe Rust bindings for CGNS (CFD General Notation System).
//!
//! This crate wraps the raw FFI declarations from `cgns-sys` in a safe,
//! idiomatic Rust API.  The design follows CGNS's tree-node data model:
//!
//! - [`CgnsFile`] — represents an open CGNS/HDF5 file
//! - [`IndexPath`] — path-based navigation to CGNS nodes
//! - Data types are mapped to Rust enums to prevent invalid states
//! - Errors are returned as `Result<T, CgnsError>`
//!
//! # Status
//!
//! This is a **skeleton** — module declarations are in place but the
//! implementations will be filled in during a later phase.

/// Re-export the low-level FFI bindings for advanced use.
pub use cgns_sys;

/// CGNS-specific error type wrapping CGNS error codes.
pub mod error;

/// Open / create / close CGNS files.
pub mod file;

/// CGNS node tree types and identifiers.
pub mod node;

/// Data types and conversions (Rust ↔ CGNS type system).
pub mod data;

/// Utility functions and re-exports.
pub mod util;
