//! Basic CGNS read/write example.
//!
//! This example creates a CGNS file containing a simple 2-D structured grid,
//! writes grid coordinates and a flow solution field, then reads everything
//! back and prints it.
//!
//! Run with:
//! ```bash
//! cargo run --example basic_cgns
//! ```

use std::ffi::CString;

fn main() {
    let filename = "example_basic.cgns";

    // ======================================================================
    // 1. CREATE a CGNS file and write data
    // ======================================================================
    println!("─── Writing ─────────────────────────────────────────────");

    let fn_ = open_file(filename, CG_MODE_WRITE);

    // Write a base (2-D, 2 physical dimensions)
    let base_id = write_base(fn_, "Base", 2, 2);

    // Write a structured zone: 3 vertices in i × 4 vertices in j => 12 vertices
    let ni: i64 = 3;
    let nj: i64 = 4;
    // Zone size for structured 2-D: [ni, nj, cell_i, cell_j, 0, 0]
    let zone_size = [ni, nj, ni - 1, nj - 1, 0, 0];
    let zone_id = write_zone(fn_, base_id, "Zone", &zone_size);

    // Generate coordinate arrays: x = i, y = j (a rectangular lattice)
    let npts = (ni * nj) as usize;
    let mut xs = vec![0.0f64; npts];
    let mut ys = vec![0.0f64; npts];
    let mut idx = 0usize;
    for j in 0..nj {
        for i in 0..ni {
            xs[idx] = i as f64;
            ys[idx] = j as f64;
            idx += 1;
        }
    }

    write_coords(fn_, base_id, zone_id, "CoordinateX", &xs);
    write_coords(fn_, base_id, zone_id, "CoordinateY", &ys);

    // Write a solution at vertices with a scalar field "Pressure"
    let pressure: Vec<f64> = xs.iter().zip(&ys).map(|(&x, &y)| x * 0.5 + y * 0.3).collect();
    let sol_id = write_solution(fn_, base_id, zone_id, "Solution");
    write_field(fn_, base_id, zone_id, sol_id, "Pressure", &pressure);

    close_file(fn_);

    // ======================================================================
    // 2. RE-OPEN and read back
    // ======================================================================
    println!("─── Reading ──────────────────────────────────────────────");

    let fn_ = open_file(filename, CG_MODE_READ);

    let (_nbases, _nzones, ncoords, _nsols) = query_structure(fn_);

    // Read coordinates
    let rmin = [1i64, 1];
    let rmax = [ni, nj];
    let mut xs_read = vec![0.0f64; npts];
    let mut ys_read = vec![0.0f64; npts];
    read_coords(fn_, 1, 1, "CoordinateX", &rmin, &rmax, &mut xs_read);
    read_coords(fn_, 1, 1, "CoordinateY", &rmin, &rmax, &mut ys_read);

    println!(
        "Read {} coordinates for each of {} arrays",
        xs_read.len(),
        ncoords
    );

    // Read field
    let mut pressure_read = vec![0.0f64; npts];
    read_field(fn_, 1, 1, 1, "Pressure", &rmin, &rmax, &mut pressure_read);

    // Verify
    let coords_ok = xs_read == xs && ys_read == ys;
    let field_ok = pressure_read == pressure;
    if coords_ok && field_ok {
        println!("✅ Data verification: all values match");
    } else {
        eprintln!("❌ Data verification FAILED");
    }

    // Print some values
    println!("\nFirst 4 points (i={}..{}, j=0):", 0, ni - 1);
    for i in 0..ni as usize {
        println!(
            "  Point {}: x={}, y={}, Pressure={}",
            i, xs_read[i], ys_read[i], pressure_read[i]
        );
    }

    close_file(fn_);
    std::fs::remove_file(filename).ok();
    println!("\nDone (cleaned up {})", filename);
}

// ---------------------------------------------------------------------------
// Safe helper wrappers (using cgns_sys)
// ---------------------------------------------------------------------------

use cgns_sys::*;

fn open_file(filename: &str, mode: u32) -> i32 {
    let c_name = CString::new(filename).unwrap();
    let mut fn_: i32 = 0;
    if mode == CG_MODE_WRITE {
        unsafe { cg_set_file_type(CG_FILE_HDF5 as i32); }
    }
    let status = unsafe { cg_open(c_name.as_ptr(), mode as i32, &mut fn_) };
    assert_eq!(status, CG_OK as i32, "cg_open failed: {}", error_message());
    fn_
}

fn close_file(fn_: i32) {
    let status = unsafe { cg_close(fn_) };
    assert_eq!(status, CG_OK as i32, "cg_close failed: {}", error_message());
}

fn write_base(fn_: i32, name: &str, cell_dim: i32, phys_dim: i32) -> i32 {
    let c_name = CString::new(name).unwrap();
    let mut base: i32 = 0;
    let status = unsafe { cg_base_write(fn_, c_name.as_ptr(), cell_dim, phys_dim, &mut base) };
    assert_eq!(status, CG_OK as i32, "cg_base_write failed: {}", error_message());
    println!("  Base '{}' (id={})", name, base);
    base
}

fn write_zone(fn_: i32, base: i32, name: &str, size: &[i64]) -> i32 {
    let c_name = CString::new(name).unwrap();
    let mut zone: i32 = 0;
    let status = unsafe {
        cg_zone_write(fn_, base, c_name.as_ptr(), size.as_ptr(), ZoneType_t_Structured, &mut zone)
    };
    assert_eq!(status, CG_OK as i32, "cg_zone_write failed: {}", error_message());
    println!("  Zone '{}' (id={}) dims={:?}", name, zone, &size[..size.len() / 3 * 2]);
    zone
}

fn write_coords(fn_: i32, base: i32, zone: i32, name: &str, data: &[f64]) {
    let c_name = CString::new(name).unwrap();
    let mut idx: i32 = 0;
    let status = unsafe {
        cg_coord_write(
            fn_,
            base,
            zone,
            DataType_t_RealDouble,
            c_name.as_ptr(),
            data.as_ptr() as *const std::ffi::c_void,
            &mut idx,
        )
    };
    assert_eq!(status, CG_OK as i32, "cg_coord_write({}) failed: {}", name, error_message());
    println!("  Coordinates '{}' written ({} points)", name, data.len());
}

fn write_solution(fn_: i32, base: i32, zone: i32, name: &str) -> i32 {
    let c_name = CString::new(name).unwrap();
    let mut sol: i32 = 0;
    let status = unsafe { cg_sol_write(fn_, base, zone, c_name.as_ptr(), GridLocation_t_Vertex, &mut sol) };
    assert_eq!(status, CG_OK as i32, "cg_sol_write failed: {}", error_message());
    println!("  Solution '{}' (id={})", name, sol);
    sol
}

fn write_field(fn_: i32, base: i32, zone: i32, sol: i32, name: &str, data: &[f64]) {
    let c_name = CString::new(name).unwrap();
    let mut field: i32 = 0;
    let status = unsafe {
        cg_field_write(
            fn_,
            base,
            zone,
            sol,
            DataType_t_RealDouble,
            c_name.as_ptr(),
            data.as_ptr() as *const std::ffi::c_void,
            &mut field,
        )
    };
    assert_eq!(status, CG_OK as i32, "cg_field_write({}) failed: {}", name, error_message());
    println!("  Field '{}' written ({} values)", name, data.len());
}

fn query_structure(fn_: i32) -> (i32, i32, i32, i32) {
    let mut nbases: i32 = 0;
    let mut nzones: i32 = 0;
    let mut ncoords: i32 = 0;
    let mut nsols: i32 = 0;

    let status = unsafe { cg_nbases(fn_, &mut nbases) };
    assert_eq!(status, CG_OK as i32, "cg_nbases failed");

    if nbases > 0 {
        let status = unsafe { cg_nzones(fn_, 1, &mut nzones) };
        assert_eq!(status, CG_OK as i32, "cg_nzones failed");

        if nzones > 0 {
            let status = unsafe { cg_ncoords(fn_, 1, 1, &mut ncoords) };
            assert_eq!(status, CG_OK as i32, "cg_ncoords failed");

            let status = unsafe { cg_nsols(fn_, 1, 1, &mut nsols) };
            assert_eq!(status, CG_OK as i32, "cg_nsols failed");
        }
    }

    println!(
        "File structure: {} bases, {} zones, {} coords, {} solutions",
        nbases, nzones, ncoords, nsols
    );
    (nbases, nzones, ncoords, nsols)
}

fn read_coords(fn_: i32, base: i32, zone: i32, name: &str, rmin: &[i64], rmax: &[i64], data: &mut [f64]) {
    let c_name = CString::new(name).unwrap();
    let status = unsafe {
        cg_coord_read(
            fn_,
            base,
            zone,
            c_name.as_ptr(),
            DataType_t_RealDouble,
            rmin.as_ptr(),
            rmax.as_ptr(),
            data.as_mut_ptr() as *mut std::ffi::c_void,
        )
    };
    assert_eq!(status, CG_OK as i32, "cg_coord_read({}) failed: {}", name, error_message());
}

fn read_field(
    fn_: i32,
    base: i32,
    zone: i32,
    sol: i32,
    name: &str,
    rmin: &[i64],
    rmax: &[i64],
    data: &mut [f64],
) {
    let c_name = CString::new(name).unwrap();
    let status = unsafe {
        cg_field_read(
            fn_,
            base,
            zone,
            sol,
            c_name.as_ptr(),
            DataType_t_RealDouble,
            rmin.as_ptr(),
            rmax.as_ptr(),
            data.as_mut_ptr() as *mut std::ffi::c_void,
        )
    };
    assert_eq!(status, CG_OK as i32, "cg_field_read({}) failed: {}", name, error_message());
}
