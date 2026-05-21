use cgns::data::{ElementType, GridLocation};
use cgns::CgnsFile;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().unwrap().to_path_buf()
}

fn test_path(name: &str) -> PathBuf {
    let base = workspace_root().join("target").join("cgns_test_files");
    std::fs::create_dir_all(&base).ok();
    base.join(format!("cgns_rs_{}.cgns", name))
}

// ---------------------------------------------------------------------------
// 3-D structured grid: full write-then-read cycle
// ---------------------------------------------------------------------------
#[test]
fn test_structured_3d_write_read() {
    let path = test_path("structured_3d");
    let _ = std::fs::remove_file(&path);

    let ni: i64 = 4;
    let nj: i64 = 3;
    let nk: i64 = 5;
    let nverts = (ni * nj * nk) as usize;

    let mut xs = vec![0.0f64; nverts];
    let mut ys = vec![0.0f64; nverts];
    let mut zs = vec![0.0f64; nverts];
    let mut idx = 0;
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
    let field_data: Vec<f64> = xs
        .iter()
        .zip(&ys)
        .zip(&zs)
        .map(|((&x, &y), &z)| x + y + z)
        .collect();

    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create file");
        let base = file.create_base("TestBase", 3, 3).expect("create base");
        let zone = base
            .create_zone_structured("TestZone", &[ni, nj, nk, ni - 1, nj - 1, nk - 1, 0, 0, 0])
            .expect("create zone");
        zone.write_coord_f64("CoordinateX", &xs).expect("write X");
        zone.write_coord_f64("CoordinateY", &ys).expect("write Y");
        zone.write_coord_f64("CoordinateZ", &zs).expect("write Z");
        let sol = zone
            .write_solution("TestSolution", GridLocation::Vertex)
            .expect("write solution");
        sol.write_field_f64("Density", &field_data)
            .expect("write field");
    }

    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open file");
        assert_eq!(file.base_count().expect("base count"), 1);
        let base = file.base("TestBase").expect("find base");
        assert_eq!(base.name().expect("base name"), "TestBase");
        assert_eq!(base.zone_count().expect("zone count"), 1);

        let zones = base.zones().expect("list zones");
        let zone = &zones[0];
        assert_eq!(zone.coord_count().expect("coord count"), 3);
        assert_eq!(
            zone.coord_names().expect("coord names"),
            vec!["CoordinateX", "CoordinateY", "CoordinateZ"]
        );

        let rmin = [1i64, 1, 1];
        let rmax = [ni, nj, nk];
        let mut xs_read = vec![0.0f64; nverts];
        let mut ys_read = vec![0.0f64; nverts];
        let mut zs_read = vec![0.0f64; nverts];
        zone.read_coord_f64("CoordinateX", &rmin, &rmax, &mut xs_read)
            .expect("read X");
        zone.read_coord_f64("CoordinateY", &rmin, &rmax, &mut ys_read)
            .expect("read Y");
        zone.read_coord_f64("CoordinateZ", &rmin, &rmax, &mut zs_read)
            .expect("read Z");
        assert_eq!(xs_read, xs);
        assert_eq!(ys_read, ys);
        assert_eq!(zs_read, zs);

        let sol = zone.solution("TestSolution").expect("find solution");
        assert_eq!(sol.field_count().expect("field count"), 1);
        let mut field_read = vec![0.0f64; nverts];
        sol.read_field_f64("Density", &rmin, &rmax, &mut field_read)
            .expect("read field");
        assert_eq!(field_read, field_data);
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// 2-D structured grid
// ---------------------------------------------------------------------------
#[test]
fn test_2d_grid() {
    let path = test_path("grid_2d");
    let _ = std::fs::remove_file(&path);

    let ni: i64 = 5;
    let nj: i64 = 4;
    let npts = (ni * nj) as usize;

    let xs: Vec<f64> = (0..npts).map(|i| i as f64).collect();
    let ys: Vec<f64> = (0..npts).map(|i| (i / ni as usize) as f64).collect();

    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
        let base = file.create_base("Base", 2, 2).expect("create base");
        let zone = base
            .create_zone_structured("Zone", &[ni, nj, ni - 1, nj - 1])
            .expect("create zone");
        zone.write_coord_f64("X", &xs).expect("write X");
        zone.write_coord_f64("Y", &ys).expect("write Y");
    }

    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
        let base = file.base("Base").expect("find base");
        let zone = &base.zones().expect("zones")[0];

        let mut xs_read = vec![0.0; npts];
        let mut ys_read = vec![0.0; npts];
        zone.read_coord_f64("X", &[1, 1], &[ni, nj], &mut xs_read)
            .expect("read X");
        zone.read_coord_f64("Y", &[1, 1], &[ni, nj], &mut ys_read)
            .expect("read Y");
        assert_eq!(xs_read, xs);
        assert_eq!(ys_read, ys);
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Multiple bases in one file
// ---------------------------------------------------------------------------
#[test]
fn test_multiple_bases() {
    let path = test_path("multiple_bases");
    let _ = std::fs::remove_file(&path);

    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
        file.create_base("BaseA", 2, 2).expect("base A");
        file.create_base("BaseB", 3, 3).expect("base B");
        file.create_base("BaseC", 2, 3).expect("base C");
    }

    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
        assert_eq!(file.base_count().expect("count"), 3);
        let bases = file.bases().expect("bases");
        assert_eq!(bases.len(), 3);
        assert_eq!(bases[0].name().expect("name"), "BaseA");
        assert_eq!(bases[1].name().expect("name"), "BaseB");
        assert_eq!(bases[2].name().expect("name"), "BaseC");
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Open in modify mode (append data to existing file)
// ---------------------------------------------------------------------------
#[test]
fn test_modify_mode() {
    let path = test_path("modify_mode");
    let _ = std::fs::remove_file(&path);

    // Create file with one base
    let data = vec![0.0f64; 8];
    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
        let base = file.create_base("Base", 3, 3).expect("create base");
        let zone = base
            .create_zone_structured("Zone", &[2, 2, 2, 1, 1, 1, 0, 0, 0])
            .expect("create zone");
        zone.write_coord_f64("X", &data).expect("write X");
    }

    // Reopen in modify mode and add another base
    {
        let file = CgnsFile::modify(&path.to_string_lossy()).expect("modify");
        assert_eq!(file.base_count().expect("base count"), 1);
        let _base2 = file.create_base("Base2", 2, 2).expect("create base2");
    }

    // Verify both bases exist
    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
        assert_eq!(file.base_count().expect("base count"), 2);
        assert!(file.base("Base").is_ok());
        assert!(file.base("Base2").is_ok());
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Multiple zones in one base
// ---------------------------------------------------------------------------
#[test]
fn test_multiple_zones() {
    let path = test_path("multiple_zones");
    let _ = std::fs::remove_file(&path);

    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
        let base = file.create_base("Base", 2, 2).expect("create base");
        base.create_zone_structured("Zone1", &[3, 4, 2, 3])
            .expect("zone 1");
        base.create_zone_structured("Zone2", &[5, 6, 4, 5])
            .expect("zone 2");
    }

    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
        let base = file.base("Base").expect("find base");
        assert_eq!(base.zone_count().expect("count"), 2);
        let zones = base.zones().expect("zones");
        assert_eq!(zones.len(), 2);
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Multiple solutions and fields per zone
// ---------------------------------------------------------------------------
#[test]
fn test_multiple_solutions_and_fields() {
    let path = test_path("multi_sol_field");
    let _ = std::fs::remove_file(&path);

    let nverts = 12usize;
    let data = vec![1.0f64; nverts];
    let data2 = vec![2.0f64; nverts];

    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
        let base = file.create_base("Base", 2, 2).expect("create base");
        let zone = base
            .create_zone_structured("Zone", &[3, 4, 2, 3])
            .expect("create zone");

        let sol1 = zone
            .write_solution("Sol1", GridLocation::Vertex)
            .expect("sol 1");
        sol1.write_field_f64("FieldA", &data).expect("field A");
        sol1.write_field_f64("FieldB", &data2).expect("field B");

        let sol2 = zone
            .write_solution("Sol2", GridLocation::CellCenter)
            .expect("sol 2");
        sol2.write_field_f64("FieldC", &data).expect("field C");
    }

    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
        let zone = file
            .base("Base")
            .expect("base")
            .zones()
            .expect("zones")
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(zone.solution_count().expect("sol count"), 2);

        let sol1 = zone.solution("Sol1").expect("find Sol1");
        assert_eq!(sol1.field_count().expect("field count"), 2);

        let sol2 = zone.solution("Sol2").expect("find Sol2");
        assert_eq!(sol2.field_count().expect("field count"), 1);
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Partial read (sub-range of coordinates)
// ---------------------------------------------------------------------------
#[test]
fn test_partial_read() {
    let path = test_path("partial_read");
    let _ = std::fs::remove_file(&path);

    let ni: i64 = 10;
    let nj: i64 = 10;
    let nk: i64 = 10;
    let nverts = (ni * nj * nk) as usize;
    let xs: Vec<f64> = (0..nverts).map(|i| i as f64).collect();

    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
        let base = file.create_base("Base", 3, 3).expect("create base");
        let zone = base
            .create_zone_structured("Zone", &[ni, nj, nk, ni - 1, nj - 1, nk - 1, 0, 0, 0])
            .expect("create zone");
        zone.write_coord_f64("X", &xs).expect("write X");
    }

    // Read a 2x2x2 corner sub-range
    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
        let zone = file
            .base("Base")
            .expect("base")
            .zones()
            .expect("zones")
            .into_iter()
            .next()
            .unwrap();
        let mut corner = vec![0.0f64; 8];
        zone.read_coord_f64("X", &[1, 1, 1], &[2, 2, 2], &mut corner)
            .expect("read corner");
        // Expected: indices (1,1,1)=0, (2,1,1)=1, (1,2,1)=10, (2,2,1)=11,
        //           (1,1,2)=100, (2,1,2)=101, (1,2,2)=110, (2,2,2)=111
        assert_eq!(corner[0], 0.0);
        assert_eq!(corner[1], 1.0);
        assert_eq!(corner[2], 10.0);
        assert_eq!(corner[3], 11.0);
        assert_eq!(corner[4], 100.0);
        assert_eq!(corner[5], 101.0);
        assert_eq!(corner[6], 110.0);
        assert_eq!(corner[7], 111.0);
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Explicit close() before Drop
// ---------------------------------------------------------------------------
#[test]
fn test_explicit_close() {
    let path = test_path("explicit_close");
    let _ = std::fs::remove_file(&path);

    let mut file = CgnsFile::create(&path.to_string_lossy()).expect("create");
    file.create_base("Base", 2, 2).expect("create base");
    file.close().expect("close");
    // Double close should be a no-op
    file.close().expect("close again");

    // Reopen and verify
    let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
    assert_eq!(file.base_count().expect("count"), 1);

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Open non-existent file
// ---------------------------------------------------------------------------
#[test]
fn test_open_nonexistent() {
    let result = CgnsFile::open("/nonexistent/path/test.cgns");
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Base not found
// ---------------------------------------------------------------------------
#[test]
fn test_base_not_found() {
    let path = test_path("base_not_found");
    let _ = std::fs::remove_file(&path);

    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
        file.create_base("RealBase", 2, 2).expect("create base");
    }

    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
        let result = file.base("ImaginaryBase");
        assert!(result.is_err());
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Empty file has no bases
// ---------------------------------------------------------------------------
#[test]
fn test_empty_file() {
    let path = test_path("empty_file");
    let _ = std::fs::remove_file(&path);

    {
        let _file = CgnsFile::create(&path.to_string_lossy()).expect("create");
        // no bases written
    }

    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
        assert_eq!(file.base_count().expect("count"), 0);
        let bases = file.bases().expect("bases");
        assert!(bases.is_empty());
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Minimal 2x2x2 zone
// ---------------------------------------------------------------------------
#[test]
fn test_minimal_zone() {
    let path = test_path("minimal_zone");
    let _ = std::fs::remove_file(&path);

    let data = vec![0.0f64; 8];
    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
        let base = file.create_base("Base", 3, 3).expect("create base");
        let zone = base
            .create_zone_structured("Tiny", &[2, 2, 2, 1, 1, 1, 0, 0, 0])
            .expect("create zone");
        zone.write_coord_f64("X", &data).expect("write X");
    }

    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
        let zone = file
            .base("Base")
            .expect("base")
            .zones()
            .expect("zones")
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(zone.coord_count().expect("coord count"), 1);
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// GridLocation round-trip via solution query
// ---------------------------------------------------------------------------
#[test]
fn test_grid_location_roundtrip() {
    let path = test_path("grid_location");
    let _ = std::fs::remove_file(&path);

    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
        let base = file.create_base("Base", 2, 2).expect("create base");
        let zone = base
            .create_zone_structured("Zone", &[3, 4, 2, 3])
            .expect("create zone");
        zone.write_solution("VertexSol", GridLocation::Vertex)
            .expect("vertex");
        zone.write_solution("CellSol", GridLocation::CellCenter)
            .expect("cell");
    }

    // We can't read back the location with the current API,
    // but we can verify the solutions exist
    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
        let zone = file
            .base("Base")
            .expect("base")
            .zones()
            .expect("zones")
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(zone.solution_count().expect("count"), 2);
        zone.solution("VertexSol").expect("find VertexSol");
        zone.solution("CellSol").expect("find CellSol");
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Single TRI_3 element (unstructured)
// ---------------------------------------------------------------------------
#[test]
fn test_unstructured_tri3() {
    let path = test_path("unstructured_tri3");
    let _ = std::fs::remove_file(&path);

    // Three triangles
    let conn: Vec<i64> = vec![1, 2, 3, 2, 4, 3, 1, 3, 4];

    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
        let base = file.create_base("Base", 3, 3).expect("create base");
        let zone = base
            .create_zone_unstructured("Zone", 4, 3)
            .expect("create zone");
        zone.write_section("TriSection", ElementType::Tri3, 1, 3, 0, &conn)
            .expect("write section");
    }

    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
        let zone = file
            .base("Base")
            .expect("base")
            .zones()
            .expect("zones")
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(zone.section_count().expect("section count"), 1);

        let sec = zone.section("TriSection").expect("find section");
        let info = sec.info().expect("section info");
        assert_eq!(info.name, "TriSection");
        assert_eq!(info.element_type, ElementType::Tri3);
        assert_eq!(info.start, 1);
        assert_eq!(info.end, 3);
        assert_eq!(sec.element_count().expect("element count"), 3);

        let read_conn = sec.read_connectivity().expect("read connectivity");
        assert_eq!(read_conn, conn);

        let arr = sec.read_connectivity_ndarray().expect("ndarray");
        assert_eq!(arr.shape(), &[3, 3]);
        assert_eq!(arr[(0, 0)], 1);
        assert_eq!(arr[(2, 2)], 4);
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Single TETRA_4 element (unstructured)
// ---------------------------------------------------------------------------
#[test]
fn test_unstructured_tetra4() {
    let path = test_path("unstructured_tetra4");
    let _ = std::fs::remove_file(&path);

    // Two tets
    let conn: Vec<i64> = vec![1, 2, 3, 4, 2, 5, 4, 3];

    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
        let base = file.create_base("Base", 3, 3).expect("create base");
        let zone = base
            .create_zone_unstructured("Zone", 5, 2)
            .expect("create zone");
        zone.write_section("TetSection", ElementType::Tetra4, 1, 2, 0, &conn)
            .expect("write section");
    }

    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
        let zone = file
            .base("Base")
            .expect("base")
            .zones()
            .expect("zones")
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(zone.section_count().expect("section count"), 1);

        let sec = zone.section("TetSection").expect("find section");
        let info = sec.info().expect("section info");
        assert_eq!(info.name, "TetSection");
        assert_eq!(info.element_type, ElementType::Tetra4);
        assert_eq!(sec.element_count().expect("element count"), 2);

        let read_conn = sec.read_connectivity().expect("read connectivity");
        assert_eq!(read_conn, conn);

        let arr = sec.read_connectivity_ndarray().expect("ndarray");
        assert_eq!(arr.shape(), &[2, 4]);
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Single HEXA_8 element (unstructured)
// ---------------------------------------------------------------------------
#[test]
fn test_unstructured_hexa8() {
    let path = test_path("unstructured_hexa8");
    let _ = std::fs::remove_file(&path);

    // One hex
    let conn: Vec<i64> = (1..=8).collect();

    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
        let base = file.create_base("Base", 3, 3).expect("create base");
        let zone = base
            .create_zone_unstructured("Zone", 8, 1)
            .expect("create zone");
        zone.write_section("HexSection", ElementType::Hexa8, 1, 1, 0, &conn)
            .expect("write section");
    }

    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
        let zone = file
            .base("Base")
            .expect("base")
            .zones()
            .expect("zones")
            .into_iter()
            .next()
            .unwrap();
        let sec = zone.section("HexSection").expect("find section");
        let read_conn = sec.read_connectivity().expect("read connectivity");
        assert_eq!(read_conn, conn);

        let arr = sec.read_connectivity_ndarray().expect("ndarray");
        assert_eq!(arr.shape(), &[1, 8]);
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Multiple sections in one zone
// ---------------------------------------------------------------------------
#[test]
fn test_unstructured_multiple_sections() {
    let path = test_path("multi_section");
    let _ = std::fs::remove_file(&path);

    let tri_conn: Vec<i64> = vec![1, 2, 3];
    let tet_conn: Vec<i64> = vec![1, 2, 3, 4];

    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
        let base = file.create_base("Base", 3, 3).expect("create base");
        let zone = base
            .create_zone_unstructured("Zone", 4, 2)
            .expect("create zone");
        zone.write_section("Tris", ElementType::Tri3, 1, 1, 0, &tri_conn)
            .expect("write tris");
        zone.write_section("Tets", ElementType::Tetra4, 2, 2, 0, &tet_conn)
            .expect("write tets");
    }

    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
        let zone = file
            .base("Base")
            .expect("base")
            .zones()
            .expect("zones")
            .into_iter()
            .next()
            .unwrap();
        let secs = zone.sections().expect("sections");
        assert_eq!(secs.len(), 2);

        let tri = zone.section("Tris").expect("find Tris");
        assert_eq!(tri.read_connectivity().expect("conn"), tri_conn);

        let tet = zone.section("Tets").expect("find Tets");
        assert_eq!(tet.read_connectivity().expect("conn"), tet_conn);
    }

    std::fs::remove_file(&path).ok();
}
