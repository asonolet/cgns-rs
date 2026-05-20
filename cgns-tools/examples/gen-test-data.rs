//! Generate a comprehensive CGNS test file used by integration tests.
//!
//! Run: cargo run --example gen-test-data --manifest-path cgns-tools/Cargo.toml
//! Output goes to cgns-tools/tests/data/test_grid.cgns

use cgns::data::{ElementType, GridLocation};
use cgns::CgnsFile;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().unwrap().to_path_buf()
}

fn main() {
    let out_dir = workspace_root().join("cgns-tools").join("tests").join("data");
    std::fs::create_dir_all(&out_dir).unwrap();
    let path = out_dir.join("test_grid.cgns");

    let file = CgnsFile::create(&path.to_string_lossy()).unwrap();

    // --- Base 1: 2-D structured ---
    let base = file.create_base("BaseStructured2D", 2, 2).unwrap();
    let zone = base.create_zone_structured("Grid2D", &[5, 4, 4, 3]).unwrap();
    let xs: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let ys: Vec<f64> = (0..20).map(|i| (i as f64) * 2.0).collect();
    zone.write_coord_f64("CoordinateX", &xs).unwrap();
    zone.write_coord_f64("CoordinateY", &ys).unwrap();
    let sol = zone.write_solution("Solution", GridLocation::Vertex).unwrap();
    let field: Vec<f64> = xs.iter().zip(&ys).map(|(x, y)| x + y).collect();
    sol.write_field_f64("Pressure", &field).unwrap();

    // --- Base 2: 3-D structured ---
    let base2 = file.create_base("BaseStructured3D", 3, 3).unwrap();
    let zone2 = base2.create_zone_structured("Volume", &[4, 3, 5, 3, 2, 4, 0, 0, 0]).unwrap();
    let nverts = 60usize;
    let xs2: Vec<f64> = (0..nverts).map(|i| i as f64).collect();
    let ys2: Vec<f64> = (0..nverts).map(|i| (i % 3) as f64).collect();
    let zs2: Vec<f64> = (0..nverts).map(|i| (i / 12) as f64).collect();
    zone2.write_coord_f64("CoordinateX", &xs2).unwrap();
    zone2.write_coord_f64("CoordinateY", &ys2).unwrap();
    zone2.write_coord_f64("CoordinateZ", &zs2).unwrap();

    // --- Base 3: Unstructured with mixed elements ---
    let base3 = file.create_base("BaseUnstructured", 3, 3).unwrap();
    let zone3 = base3.create_zone_unstructured("Mesh", 8, 4).unwrap();

    // Tris: 2 triangles
    let tri_conn: Vec<i64> = vec![1, 2, 3, 2, 4, 3];
    zone3.write_section("TriSection", ElementType::Tri3, 1, 2, 0, &tri_conn).unwrap();
    // Hex: 1 hex
    let hex_conn: Vec<i64> = (1..=8).collect();
    zone3.write_section("HexSection", ElementType::Hexa8, 3, 3, 0, &hex_conn).unwrap();
    // Tet: 1 tet
    let tet_conn: Vec<i64> = vec![1, 2, 3, 4];
    zone3.write_section("TetSection", ElementType::Tetra4, 4, 4, 0, &tet_conn).unwrap();

    println!("Test data written to {}", path.display());
}
