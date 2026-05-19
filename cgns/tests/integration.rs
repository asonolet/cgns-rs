use cgns::data::GridLocation;
use cgns::CgnsFile;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().unwrap().to_path_buf()
}

fn test_path(name: &str) -> PathBuf {
    let base = workspace_root().join("target").join("cgns_test_files");
    std::fs::create_dir_all(&base).ok();
    base.join(format!("cgns_rs_test_{}.cgns", name))
}

// ---------------------------------------------------------------------------
// Test: create a simple structured 3-D grid, write coordinates and a solution
//       field, then read everything back.
// ---------------------------------------------------------------------------

#[test]
fn test_structured_3d_write_read() {
    let path = test_path("high_level_structured_3d");
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
    let field_data: Vec<f64> = xs.iter().zip(&ys).zip(&zs).map(|((&x, &y), &z)| x + y + z).collect();

    // ---- WRITE ----
    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create file");
        let base = file.create_base("TestBase", 3, 3).expect("create base");
        let zone_size = [ni, nj, nk, ni - 1, nj - 1, nk - 1, 0, 0, 0];
        let zone = base.create_zone_structured("TestZone", &zone_size).expect("create zone");
        zone.write_coord_f64("CoordinateX", &xs).expect("write X");
        zone.write_coord_f64("CoordinateY", &ys).expect("write Y");
        zone.write_coord_f64("CoordinateZ", &zs).expect("write Z");
        let sol = zone.write_solution("TestSolution", GridLocation::Vertex).expect("write solution");
        sol.write_field_f64("Density", &field_data).expect("write field");
        // CgnsFile dropped here → auto-close
    }

    // ---- READ ----
    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open file");
        assert_eq!(file.base_count().expect("base count"), 1);

        let base = file.base("TestBase").expect("find base");
        assert_eq!(base.name().expect("base name"), "TestBase");
        assert_eq!(base.zone_count().expect("zone count"), 1);

        let zones = base.zones().expect("list zones");
        assert_eq!(zones.len(), 1);
        let zone = &zones[0];
        assert_eq!(zone.coord_count().expect("coord count"), 3);

        let coord_names = zone.coord_names().expect("coord names");
        assert_eq!(coord_names, vec!["CoordinateX", "CoordinateY", "CoordinateZ"]);

        let rmin = [1i64, 1, 1];
        let rmax = [ni, nj, nk];
        let mut xs_read = vec![0.0f64; nverts];
        let mut ys_read = vec![0.0f64; nverts];
        let mut zs_read = vec![0.0f64; nverts];
        zone.read_coord_f64("CoordinateX", &rmin, &rmax, &mut xs_read).expect("read X");
        zone.read_coord_f64("CoordinateY", &rmin, &rmax, &mut ys_read).expect("read Y");
        zone.read_coord_f64("CoordinateZ", &rmin, &rmax, &mut zs_read).expect("read Z");
        assert_eq!(xs_read, xs);
        assert_eq!(ys_read, ys);
        assert_eq!(zs_read, zs);

        let sol = zone.solution("TestSolution").expect("find solution");
        assert_eq!(sol.field_count().expect("field count"), 1);

        let mut field_read = vec![0.0f64; nverts];
        sol.read_field_f64("Density", &rmin, &rmax, &mut field_read).expect("read field");
        assert_eq!(field_read, field_data);
        // CgnsFile dropped here → auto-close
    }

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_create_base_and_query() {
    let path = test_path("high_level_base_query");
    let _ = std::fs::remove_file(&path);

    // Write phase
    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
        file.create_base("BaseA", 2, 2).expect("create base A");
        file.create_base("BaseB", 3, 3).expect("create base B");
        // file closed on drop
    }

    // Read phase — CGNS requires reopening for query
    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
        let bases = file.bases().expect("list bases");
        assert_eq!(bases.len(), 2);
        assert_eq!(bases[0].name().expect("name"), "BaseA");
        assert_eq!(bases[1].name().expect("name"), "BaseB");

        let found = file.base("BaseB").expect("find BaseB");
        assert_eq!(found.index(), 2);
    }

    std::fs::remove_file(&path).ok();
}

#[test]
fn test_solution_not_found() {
    let path = test_path("high_level_sol_not_found");
    let _ = std::fs::remove_file(&path);

    let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
    let base = file.create_base("Base", 2, 2).expect("create base");
    let zone = base.create_zone_structured("Zone", &[3, 4, 2, 3]).expect("create zone");
    let result = zone.solution("NonExistent");
    assert!(result.is_err(), "should fail for missing solution");

    std::fs::remove_file(&path).ok();
}
