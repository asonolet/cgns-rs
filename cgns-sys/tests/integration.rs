//! Integration tests for the CGNS raw FFI bindings.
//!
//! These tests exercise the full write-then-read cycle using the safe helpers
//! provided by `cgns-sys`.  Each test creates a temporary CGNS file, writes
//! CGNS data into it, reads it back, and verifies correctness.

use std::path::PathBuf;

/// Return a writable path for a test CGNS file.
fn test_file_path(name: &str) -> PathBuf {
    // Use a path relative to the current dir (cargo test runs from project root).
    let base = PathBuf::from("target").join("cgns_test_files");
    std::fs::create_dir_all(&base).ok();
    let p = base.join(format!("cgns_rs_test_{}.cgns", name));
    let _ = std::fs::remove_file(&p);
    p
}

// ---------------------------------------------------------------------------
// Test: create a simple structured 3-D grid, write coordinates and a solution
//       field, then read everything back.
// ---------------------------------------------------------------------------
#[test]
fn test_structured_3d_write_read() {
    let path = test_file_path("structured_3d");
    let path_str = path.to_string_lossy();

    // --- Grid dimensions ---
    // A small structured 3-D block: 4×3×5 vertices
    let ni: i64 = 4;
    let nj: i64 = 3;
    let nk: i64 = 5;
    let nvertices = (ni * nj * nk) as usize; // 60

    // Zone size array for structured 3-D: [vtx_i, vtx_j, vtx_k, cell_i, cell_j, cell_k, 0,0,0]
    let zone_size: [i64; 9] = [ni, nj, nk, ni - 1, nj - 1, nk - 1, 0, 0, 0];

    // Generate coordinate arrays: x = i, y = j, z = k (simple lattice)
    let mut xs = vec![0.0f64; nvertices];
    let mut ys = vec![0.0f64; nvertices];
    let mut zs = vec![0.0f64; nvertices];
    let mut idx = 0usize;
    for k in 0..nk {
        for j in 0..nj {
            for i in 0..ni {
                xs[idx] = i as f64;
                ys[idx] = j as f64;
                zs[idx] = k as f64;
                idx += 1;
            }
        }
    }

    // Solution field: a simple scalar function f = x + y + z
    let field_data: Vec<f64> = xs.iter().zip(ys.iter()).zip(zs.iter())
        .map(|((&x, &y), &z)| x + y + z)
        .collect();

    // ---- WRITE ----
    {
        // open_write handles cg_set_file_type internally
        let fn_ = cgns_sys::open_write(&path_str)
            .expect("failed to open file for writing");
        let base = cgns_sys::base_write(fn_, "TestBase", 3, 3)
            .expect("failed to write base");
        let zone = cgns_sys::zone_write_structured(fn_, base, "TestZone", &zone_size)
            .expect("failed to write zone");
        cgns_sys::coord_write(fn_, base, zone, cgns_sys::DataType_t_RealDouble, "CoordinateX", &xs)
            .expect("failed to write CoordinateX");
        cgns_sys::coord_write(fn_, base, zone, cgns_sys::DataType_t_RealDouble, "CoordinateY", &ys)
            .expect("failed to write CoordinateY");
        cgns_sys::coord_write(fn_, base, zone, cgns_sys::DataType_t_RealDouble, "CoordinateZ", &zs)
            .expect("failed to write CoordinateZ");
        let sol = cgns_sys::sol_write(fn_, base, zone, "TestSolution", cgns_sys::GridLocation_t_Vertex)
            .expect("failed to write solution");
        cgns_sys::field_write(fn_, base, zone, sol, cgns_sys::DataType_t_RealDouble, "Density", &field_data)
            .expect("failed to write Density");
        cgns_sys::close(fn_).expect("failed to close file");
    }

    // ---- READ ----
    {
        let fn_ = cgns_sys::open_read(&path_str)
            .expect("failed to open file for reading");

        // Check number of bases
        let mut nbases: i32 = 0;
        let status = unsafe { cgns_sys::cg_nbases(fn_, &mut nbases) };
        assert_eq!(status, cgns_sys::CG_OK as i32, "cg_nbases failed");
        assert_eq!(nbases, 1, "expected 1 base");

        // Check number of zones
        let mut nzones: i32 = 0;
        let status = unsafe { cgns_sys::cg_nzones(fn_, 1, &mut nzones) };
        assert_eq!(status, cgns_sys::CG_OK as i32, "cg_nzones failed");
        assert_eq!(nzones, 1, "expected 1 zone");

        // Check coordinates exist
        let mut ncoords: i32 = 0;
        let status = unsafe { cgns_sys::cg_ncoords(fn_, 1, 1, &mut ncoords) };
        assert_eq!(status, cgns_sys::CG_OK as i32, "cg_ncoords failed");
        assert_eq!(ncoords, 3, "expected 3 coordinate arrays");

        // Read back coordinates
        let rmin: [i64; 3] = [1, 1, 1];
        let rmax: [i64; 3] = [ni, nj, nk];
        let mut xs_read = vec![0.0f64; nvertices];
        let mut ys_read = vec![0.0f64; nvertices];
        let mut zs_read = vec![0.0f64; nvertices];

        cgns_sys::coord_read(fn_, 1, 1, cgns_sys::DataType_t_RealDouble, "CoordinateX", &rmin, &rmax, &mut xs_read)
            .expect("failed to read CoordinateX");
        cgns_sys::coord_read(fn_, 1, 1, cgns_sys::DataType_t_RealDouble, "CoordinateY", &rmin, &rmax, &mut ys_read)
            .expect("failed to read CoordinateY");
        cgns_sys::coord_read(fn_, 1, 1, cgns_sys::DataType_t_RealDouble, "CoordinateZ", &rmin, &rmax, &mut zs_read)
            .expect("failed to read CoordinateZ");

        assert_eq!(xs_read, xs, "CoordinateX mismatch");
        assert_eq!(ys_read, ys, "CoordinateY mismatch");
        assert_eq!(zs_read, zs, "CoordinateZ mismatch");

        // Check solution exists
        let mut nsols: i32 = 0;
        let status = unsafe { cgns_sys::cg_nsols(fn_, 1, 1, &mut nsols) };
        assert_eq!(status, cgns_sys::CG_OK as i32, "cg_nsols failed");
        assert_eq!(nsols, 1, "expected 1 solution");

        // Check fields
        let mut nfields: i32 = 0;
        let status = unsafe { cgns_sys::cg_nfields(fn_, 1, 1, 1, &mut nfields) };
        assert_eq!(status, cgns_sys::CG_OK as i32, "cg_nfields failed");
        assert_eq!(nfields, 1, "expected 1 field");

        // Read back field
        let mut field_read = vec![0.0f64; nvertices];
        cgns_sys::field_read(fn_, 1, 1, 1, cgns_sys::DataType_t_RealDouble, "Density", &rmin, &rmax, &mut field_read)
            .expect("failed to read Density");
        assert_eq!(field_read, field_data, "Density field mismatch");

        cgns_sys::close(fn_).expect("failed to close file");
    }

    // Cleanup
    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Test: error handling — opening a non-existent file should fail.
// ---------------------------------------------------------------------------
#[test]
fn test_open_nonexistent_file() {
    let result = cgns_sys::open_read("/nonexistent/path/test.cgns");
    assert!(result.is_err(), "opening a non-existent file should fail");
    let err = result.unwrap_err();
    assert!(!err.is_empty(), "error message should not be empty");
}

// ---------------------------------------------------------------------------
// Test: write with empty / invalid parameters.
// ---------------------------------------------------------------------------
#[test]
fn test_invalid_filename_nul_byte() {
    let result = cgns_sys::open_write("bad\0file");
    assert!(result.is_err(), "filename with null byte should fail");
}
