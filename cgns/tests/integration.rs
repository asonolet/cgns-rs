use cgns::data::{BcType, ElementType, GridLocation, PointSetType, ZoneType};
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
// Comprehensive 3-D structured grid: coords, fields, BCs, 1-to-1, families
// Port of write_test.c structured section
// ---------------------------------------------------------------------------
#[test]
fn test_comprehensive_structured_3d() {
    let path = test_path("comprehensive_structured_3d");
    let _ = std::fs::remove_file(&path);

    let ni: i64 = 5;
    let nj: i64 = 4;
    let nk: i64 = 3;
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
    let xs_f32: Vec<f32> = xs.iter().map(|&x| x as f32).collect();
    let field_f64: Vec<f64> = xs
        .iter()
        .zip(&ys)
        .zip(&zs)
        .map(|((&x, &y), &z)| x + y + z)
        .collect();
    let field_f32: Vec<f32> = field_f64.iter().map(|&v| v as f32).collect();
    let field_i32: Vec<i32> = (0..nverts as i32).collect();
    let field_i64: Vec<i64> = (0..nverts as i64).collect();

    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create file");
        let base = file.create_base("Base", 3, 3).expect("create base");

        base.write_family("Wing").expect("family Wing");
        base.write_family("Fuselage").expect("family Fuselage");

        let zone_a = base
            .create_zone_structured("ZoneA", &[ni, nj, nk, ni - 1, nj - 1, nk - 1, 0, 0, 0])
            .expect("create zone A");

        zone_a.write_coord_f64("CoordinateX", &xs).expect("write X");
        zone_a.write_coord_f64("CoordinateY", &ys).expect("write Y");
        zone_a.write_coord_f64("CoordinateZ", &zs).expect("write Z");
        zone_a
            .write_coord_f32("CoordinateX_f32", &xs_f32)
            .expect("write X f32");

        let sol_v = zone_a
            .write_solution("VertexSolution", GridLocation::Vertex)
            .expect("write vertex solution");
        sol_v
            .write_field_f64("Density", &field_f64)
            .expect("write Density");
        zone_a
            .write_field_f32(&sol_v, "VelX", &field_f32)
            .expect("write VelX");
        zone_a
            .write_field_i32(&sol_v, "NodeID", &field_i32)
            .expect("write NodeID");
        zone_a
            .write_field_i64(&sol_v, "GlobalID", &field_i64)
            .expect("write GlobalID");

        let sol_c = zone_a
            .write_solution("CellSolution", GridLocation::CellCenter)
            .expect("write cell solution");
        let ncells = ((ni - 1) * (nj - 1) * (nk - 1)) as usize;
        let cell_data = vec![2.0f64; ncells];
        sol_c
            .write_field_f64("Pressure", &cell_data)
            .expect("write Pressure");

        let imin_range = [1i64, 1, 1, 1, nj, nk];
        zone_a
            .write_bc(
                "IMinWall",
                BcType::Wall,
                PointSetType::PointRange,
                2,
                &imin_range,
            )
            .expect("write BC IMinWall");
        let imax_range = [ni, 1, 1, ni, nj, nk];
        zone_a
            .write_bc(
                "IMaxOutflow",
                BcType::Outflow,
                PointSetType::PointRange,
                2,
                &imax_range,
            )
            .expect("write BC IMaxOutflow");
        let jmin_sym = [1i64, 1, 1, ni, 1, nk];
        zone_a
            .write_bc(
                "JMinSymmetry",
                BcType::SymmetryPlane,
                PointSetType::PointRange,
                2,
                &jmin_sym,
            )
            .expect("write BC JMinSymmetry");

        let zone_b = base
            .create_zone_structured("ZoneB", &[ni, nj, nk, ni - 1, nj - 1, nk - 1, 0, 0, 0])
            .expect("create zone B");
        zone_b
            .write_coord_f64("CoordinateX", &xs)
            .expect("write zone B X");
        zone_b
            .write_coord_f64("CoordinateY", &ys)
            .expect("write zone B Y");
        zone_b
            .write_coord_f64("CoordinateZ", &zs)
            .expect("write zone B Z");

        let range = [ni, 1i64, 1, ni, nj, nk];
        let donor_range = [1i64, 1, 1, 1, nj, nk];
        let transform = [1i32, 2, 3];
        zone_a
            .write_1to1("AToB", "ZoneB", &range, &donor_range, &transform)
            .expect("write 1-to-1");
    }

    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open file");
        let base = file.base("Base").expect("find base");

        assert_eq!(base.family_count().expect("family count"), 2);
        let fam_names = base.family_names().expect("family names");
        assert!(fam_names.contains(&"Wing".to_string()));
        assert!(fam_names.contains(&"Fuselage".to_string()));

        assert_eq!(base.zone_count().expect("zone count"), 2);
        let zones = base.zones().expect("zones");
        let zone_a = &zones[0];
        let _zone_b = &zones[1];

        assert_eq!(zone_a.zone_type().expect("zone type"), ZoneType::Structured);
        assert_eq!(zone_a.coord_count().expect("coord count"), 4);
        let coord_names = zone_a.coord_names().expect("coord names");
        assert!(coord_names.contains(&"CoordinateX".to_string()));
        assert!(coord_names.contains(&"CoordinateX_f32".to_string()));

        let rmin = [1i64, 1, 1];
        let rmax = [ni, nj, nk];
        let mut xs_read = vec![0.0f64; nverts];
        zone_a
            .read_coord_f64("CoordinateX", &rmin, &rmax, &mut xs_read)
            .expect("read coord X");
        assert_eq!(xs_read, xs);

        assert_eq!(zone_a.solution_count().expect("sol count"), 2);
        let sol_v = zone_a
            .solution("VertexSolution")
            .expect("find VertexSolution");
        assert_eq!(sol_v.field_count().expect("field count"), 4);

        let mut density = vec![0.0f64; nverts];
        sol_v
            .read_field_f64("Density", &rmin, &rmax, &mut density)
            .expect("read Density");
        assert_eq!(density, field_f64);

        assert_eq!(zone_a.bc_count().expect("BC count"), 3);

        let bc_imin = zone_a.bc("IMinWall").expect("find BC IMinWall");
        let bc_imin_info = bc_imin.info().expect("BC info");
        assert_eq!(bc_imin_info.bc_type, BcType::Wall);
        assert_eq!(bc_imin_info.point_set_type, PointSetType::PointRange);
        assert_eq!(bc_imin_info.num_points, 2);

        let bc_imin_pts = bc_imin.read_points().expect("BC points");
        assert_eq!(bc_imin_pts, [1i64, 1, 1, 1, nj, nk]);

        let bc_imax = zone_a.bc("IMaxOutflow").expect("find BC IMaxOutflow");
        let bc_imax_info = bc_imax.info().expect("BC info");
        assert_eq!(bc_imax_info.bc_type, BcType::Outflow);

        assert_eq!(zone_a.n1to1().expect("1-to-1 count"), 1);
        let oto = &zone_a.one_to_ones().expect("1-to-1 list")[0];
        let oto_info = oto.read().expect("1-to-1 read");
        assert_eq!(oto_info.name, "AToB");
        assert_eq!(oto_info.donor_name, "ZoneB");
        assert_eq!(oto_info.range, vec![ni, 1, 1, ni, nj, nk]);
        assert_eq!(oto_info.donor_range, vec![1, 1, 1, 1, nj, nk]);
        assert_eq!(oto_info.transform, vec![1, 2, 3]);
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Comprehensive unstructured grid: multiple element types, sections, fields
// Port of write_test.c / elemtest.c
// ---------------------------------------------------------------------------
#[test]
fn test_comprehensive_unstructured() {
    let path = test_path("comprehensive_unstructured");
    let _ = std::fs::remove_file(&path);

    // 8 vertices for a hexahedron
    let nverts: i64 = 8;
    let xs_f64: Vec<f64> = vec![0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
    let ys_f64: Vec<f64> = vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
    let zs_f64: Vec<f64> = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];

    let tri_conn: Vec<i64> = vec![1, 2, 3, 1, 3, 4, 5, 7, 6, 5, 8, 7];
    let quad_conn: Vec<i64> = vec![1, 5, 8, 4, 2, 6, 7, 3];

    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create file");
        let base = file.create_base("Base", 3, 3).expect("create base");
        let zone = base
            .create_zone_unstructured("Zone", nverts, 6)
            .expect("create zone");

        zone.write_coord_f64("CoordinateX", &xs_f64)
            .expect("write X");
        zone.write_coord_f64("CoordinateY", &ys_f64)
            .expect("write Y");
        zone.write_coord_f64("CoordinateZ", &zs_f64)
            .expect("write Z");

        zone.write_section("Tris", ElementType::Tri3, 1, 4, 4, &tri_conn)
            .expect("write Tris");
        zone.write_section("Quads", ElementType::Quad4, 5, 6, 2, &quad_conn)
            .expect("write Quads");

        let sol = zone
            .write_solution("VertexSolution", GridLocation::Vertex)
            .expect("write solution");
        let field_data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        sol.write_field_f64("Pressure", &field_data)
            .expect("write field");

        let plist = [1i64, 2, 3, 4, 5, 6, 7, 8];
        zone.write_bc("AllWalls", BcType::Wall, PointSetType::PointList, 8, &plist)
            .expect("write BC");
    }

    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open file");
        let base = file.base("Base").expect("find base");
        let zone = &base.zones().expect("zones")[0];

        assert_eq!(zone.zone_type().expect("zone type"), ZoneType::Unstructured);
        assert_eq!(zone.coord_count().expect("coord count"), 3);
        assert_eq!(zone.section_count().expect("section count"), 2);

        let tri_sec = zone.section("Tris").expect("find Tris");
        let tri_info = tri_sec.info().expect("Tris info");
        assert_eq!(tri_info.element_type, ElementType::Tri3);
        assert_eq!(tri_info.start, 1);
        assert_eq!(tri_info.end, 4);
        assert_eq!(tri_info.nbndry, 4);
        assert_eq!(tri_sec.element_count().expect("element count"), 4);
        assert_eq!(tri_sec.read_connectivity().expect("tri conn"), tri_conn);

        let and = tri_sec.read_connectivity_ndarray().expect("ndarray");
        assert_eq!(and.shape(), &[4, 3]);

        let quad_sec = zone.section("Quads").expect("find Quads");
        let quad_info = quad_sec.info().expect("Quads info");
        assert_eq!(quad_info.element_type, ElementType::Quad4);
        assert_eq!(quad_info.nbndry, 2);
        assert_eq!(quad_sec.element_count().expect("element count"), 2);
        assert_eq!(quad_sec.read_connectivity().expect("quad conn"), quad_conn);

        assert_eq!(zone.solution_count().expect("solution count"), 1);
        let sol = zone.solution("VertexSolution").expect("find solution");
        assert_eq!(sol.field_count().expect("field count"), 1);

        assert_eq!(zone.bc_count().expect("BC count"), 1);
        let bc = zone.bc("AllWalls").expect("find BC");
        let bc_info = bc.info().expect("BC info");
        assert_eq!(bc_info.bc_type, BcType::Wall);
        assert_eq!(bc_info.point_set_type, PointSetType::PointList);
        assert_eq!(bc_info.num_points, 8);
        let bc_pts = bc.read_points().expect("BC points");
        assert_eq!(bc_pts, [1i64, 2, 3, 4, 5, 6, 7, 8]);
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// All fixed-NPE element types round-trip (port of elemtest.c)
// ---------------------------------------------------------------------------
#[test]
fn test_all_element_types() {
    let path = test_path("all_element_types");
    let _ = std::fs::remove_file(&path);

    let nverts: i64 = 42;

    let bar2: Vec<i64> = vec![1, 2];
    let bar3: Vec<i64> = vec![1, 2, 12];
    let tri3: Vec<i64> = vec![4, 3, 9];
    let tri6: Vec<i64> = vec![4, 3, 9, 14, 31, 32];
    let quad4: Vec<i64> = vec![1, 2, 3, 4];
    let quad8: Vec<i64> = vec![1, 2, 3, 4, 12, 13, 14, 15];
    let quad9: Vec<i64> = vec![1, 2, 3, 4, 12, 13, 14, 15, 24];
    let tetra4: Vec<i64> = vec![8, 7, 10, 11];
    let tetra10: Vec<i64> = vec![8, 7, 10, 11, 22, 35, 36, 41, 40, 42];
    let pyra5: Vec<i64> = vec![5, 6, 7, 8, 11];
    let pyra14: Vec<i64> = vec![5, 6, 7, 8, 11, 20, 21, 22, 23, 38, 39, 40, 41, 29];
    let penta6: Vec<i64> = vec![4, 3, 9, 8, 7, 10];
    let penta15: Vec<i64> = vec![4, 3, 9, 8, 7, 10, 14, 31, 32, 19, 18, 35, 22, 34, 35];
    let penta18: Vec<i64> = vec![
        4, 3, 9, 8, 7, 10, 14, 31, 32, 19, 18, 35, 22, 34, 35, 27, 36, 37,
    ];
    let hexa8: Vec<i64> = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let hexa20: Vec<i64> = vec![
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    ];
    let hexa27: Vec<i64> = (1..=27).collect();

    type ElemCase = (&'static str, ElementType, Vec<i64>);
    let cases: &[ElemCase] = &[
        ("Bar2", ElementType::Bar2, bar2),
        ("Bar3", ElementType::Bar3, bar3),
        ("Tri3", ElementType::Tri3, tri3),
        ("Tri6", ElementType::Tri6, tri6),
        ("Quad4", ElementType::Quad4, quad4),
        ("Quad8", ElementType::Quad8, quad8),
        ("Quad9", ElementType::Quad9, quad9),
        ("Tetra4", ElementType::Tetra4, tetra4),
        ("Tetra10", ElementType::Tetra10, tetra10),
        ("Pyra5", ElementType::Pyra5, pyra5),
        ("Pyra14", ElementType::Pyra14, pyra14),
        ("Penta6", ElementType::Penta6, penta6),
        ("Penta15", ElementType::Penta15, penta15),
        ("Penta18", ElementType::Penta18, penta18),
        ("Hexa8", ElementType::Hexa8, hexa8),
        ("Hexa20", ElementType::Hexa20, hexa20),
        ("Hexa27", ElementType::Hexa27, hexa27),
    ];

    let mut total_elems: i64 = 0;
    for (_, etype, conn) in cases {
        total_elems += 1;
        let npe = etype.npe() as usize;
        assert_eq!(conn.len(), npe, "wrong npe for {:?}", etype);
    }

    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create file");
        let base = file.create_base("Base", 3, 3).expect("create base");
        let zone = base
            .create_zone_unstructured("Zone", nverts, total_elems)
            .expect("create zone");

        // Write one element per section
        for (i, (name, etype, conn)) in cases.iter().enumerate() {
            let elem_num = (i + 1) as i64;
            zone.write_section(name, *etype, elem_num, elem_num, 0, conn)
                .unwrap_or_else(|_| panic!("write section {}", name));
        }
    }

    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open file");
        let zone = file
            .base("Base")
            .expect("find base")
            .zones()
            .expect("zones")
            .into_iter()
            .next()
            .unwrap();

        assert_eq!(
            zone.section_count().expect("section count"),
            cases.len() as i32
        );

        for (name, etype, expected_conn) in cases {
            let sec = zone
                .section(name)
                .unwrap_or_else(|_| panic!("find section {}", name));
            let info = sec.info().unwrap_or_else(|_| panic!("info for {}", name));
            assert_eq!(
                info.element_type, *etype,
                "element type mismatch for {}",
                name
            );
            assert_eq!(sec.element_count().expect("element count"), 1);

            let read_conn = sec
                .read_connectivity()
                .unwrap_or_else(|_| panic!("read conn for {}", name));
            assert_eq!(
                read_conn, *expected_conn,
                "connectivity mismatch for {}",
                name
            );

            let and = sec
                .read_connectivity_ndarray()
                .unwrap_or_else(|_| panic!("ndarray for {}", name));
            assert_eq!(
                and.shape(),
                &[1, etype.npe() as usize],
                "shape mismatch for {}",
                name
            );
        }
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Multiple BC types and point set types
// ---------------------------------------------------------------------------
#[test]
fn test_bc_types() {
    let path = test_path("bc_types");
    let _ = std::fs::remove_file(&path);

    let ni: i64 = 6;
    let nj: i64 = 5;
    let nk: i64 = 4;
    let nverts = (ni * nj * nk) as usize;
    let data = vec![0.0f64; nverts];

    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create file");
        let base = file.create_base("Base", 3, 3).expect("create base");
        let zone = base
            .create_zone_structured("Zone", &[ni, nj, nk, ni - 1, nj - 1, nk - 1, 0, 0, 0])
            .expect("create zone");
        zone.write_coord_f64("X", &data).expect("write X");

        zone.write_bc(
            "Wall",
            BcType::Wall,
            PointSetType::PointRange,
            2,
            &[1, 1, 1, 1, nj, nk],
        )
        .expect("write Wall");
        zone.write_bc(
            "Inflow",
            BcType::InflowSubsonic,
            PointSetType::PointRange,
            2,
            &[ni, 1, 1, ni, nj, nk],
        )
        .expect("write Inflow");
        zone.write_bc(
            "Outflow",
            BcType::OutflowSupersonic,
            PointSetType::PointRange,
            2,
            &[1, 1, 1, ni, 1, nk],
        )
        .expect("write Outflow");
        zone.write_bc(
            "Farfield",
            BcType::Farfield,
            PointSetType::PointRange,
            2,
            &[1, nj, 1, ni, nj, nk],
        )
        .expect("write Farfield");
        zone.write_bc(
            "Symmetry",
            BcType::SymmetryPlane,
            PointSetType::PointRange,
            2,
            &[1, 1, nk, ni, nj, nk],
        )
        .expect("write Symmetry");
    }

    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open file");
        let zone = file
            .base("Base")
            .expect("find base")
            .zones()
            .expect("zones")
            .into_iter()
            .next()
            .unwrap();

        assert_eq!(zone.bc_count().expect("BC count"), 5);

        let wall = zone.bc("Wall").expect("find Wall");
        assert_eq!(wall.info().expect("info").bc_type, BcType::Wall);

        let inflow = zone.bc("Inflow").expect("find Inflow");
        assert_eq!(inflow.info().expect("info").bc_type, BcType::InflowSubsonic);

        let outflow = zone.bc("Outflow").expect("find Outflow");
        assert_eq!(
            outflow.info().expect("info").bc_type,
            BcType::OutflowSupersonic
        );

        let farfield = zone.bc("Farfield").expect("find Farfield");
        assert_eq!(farfield.info().expect("info").bc_type, BcType::Farfield);

        let sym = zone.bc("Symmetry").expect("find Symmetry");
        assert_eq!(sym.info().expect("info").bc_type, BcType::SymmetryPlane);
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Write and read multiple 1-to-1 interfaces
// ---------------------------------------------------------------------------
#[test]
fn test_multiple_1to1() {
    let path = test_path("multiple_1to1");
    let _ = std::fs::remove_file(&path);

    let verts = vec![0.0f64; 8];
    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create file");
        let base = file.create_base("Base", 3, 3).expect("create base");
        let z1 = base
            .create_zone_structured("Zone1", &[2, 2, 2, 1, 1, 1, 0, 0, 0])
            .expect("z1");
        let z2 = base
            .create_zone_structured("Zone2", &[2, 2, 2, 1, 1, 1, 0, 0, 0])
            .expect("z2");
        let z3 = base
            .create_zone_structured("Zone3", &[2, 2, 2, 1, 1, 1, 0, 0, 0])
            .expect("z3");
        z1.write_coord_f64("X", &verts).expect("z1 X");
        z2.write_coord_f64("X", &verts).expect("z2 X");
        z3.write_coord_f64("X", &verts).expect("z3 X");

        z1.write_1to1(
            "Z1_to_Z2",
            "Zone2",
            &[2, 1, 1, 2, 2, 2],
            &[1, 1, 1, 1, 2, 2],
            &[1, 2, 3],
        )
        .expect("z1->z2");
        z2.write_1to1(
            "Z2_to_Z3",
            "Zone3",
            &[2, 1, 1, 2, 2, 2],
            &[1, 1, 1, 1, 2, 2],
            &[1, 2, 3],
        )
        .expect("z2->z3");
    }

    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open file");
        let base = file.base("Base").expect("find base");
        let zones = base.zones().expect("zones");

        assert_eq!(zones[0].n1to1().expect("z1 1to1 count"), 1);
        assert_eq!(zones[1].n1to1().expect("z2 1to1 count"), 1);
        assert_eq!(zones[2].n1to1().expect("z3 1to1 count"), 0);

        let oto = &zones[0].one_to_ones().expect("z1 1to1s")[0];
        let info = oto.read().expect("read 1to1");
        assert_eq!(info.name, "Z1_to_Z2");
        assert_eq!(info.donor_name, "Zone2");
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Read back family names correctly
// ---------------------------------------------------------------------------
#[test]
fn test_family_names_readback() {
    let path = test_path("family_names_readback");
    let _ = std::fs::remove_file(&path);

    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create file");
        let base = file.create_base("Base", 3, 3).expect("create base");
        base.write_family("Farfield").expect("fam1");
        base.write_family("Wing").expect("fam2");
        base.write_family("Fuselage").expect("fam3");
        base.write_family("Tail").expect("fam4");
    }

    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open file");
        let base = file.base("Base").expect("find base");
        assert_eq!(base.family_count().expect("family count"), 4);

        let names = base.family_names().expect("family names");
        assert_eq!(names.len(), 4);
        assert!(names.contains(&"Farfield".to_string()));
        assert!(names.contains(&"Wing".to_string()));
        assert!(names.contains(&"Fuselage".to_string()));
        assert!(names.contains(&"Tail".to_string()));
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

// ---------------------------------------------------------------------------
// Boundary conditions — write structured zone with BCs, verify via count
// ---------------------------------------------------------------------------
#[test]
fn test_boundary_conditions() {
    let path = test_path("boundary_conditions");
    let _ = std::fs::remove_file(&path);

    let ni: i64 = 4;
    let nj: i64 = 3;
    let nk: i64 = 5;

    let xs = vec![0.0f64; (ni * nj * nk) as usize];
    let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
    let base = file.create_base("Base", 3, 3).expect("create base");
    let zone = base
        .create_zone_structured("Zone", &[ni, nj, nk, ni - 1, nj - 1, nk - 1, 0, 0, 0])
        .expect("create zone");
    zone.write_coord_f64("X", &xs).expect("write X");

    let imax_vertex = (nk - 1) * nj * ni + (nj - 1) * ni + 1;
    let imin_face: Vec<i64> = vec![1, imax_vertex];
    zone.write_bc(
        "Wall_imin",
        BcType::Wall,
        PointSetType::PointRange,
        2,
        &imin_face,
    )
    .expect("write BC");
    drop(file);

    let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
    let zone = file
        .base("Base")
        .expect("base")
        .zones()
        .expect("zones")
        .into_iter()
        .next()
        .unwrap();
    assert_eq!(zone.bc_count().expect("bc count"), 1);

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Families — write and read back count
// ---------------------------------------------------------------------------
#[test]
fn test_families() {
    let path = test_path("families");
    let _ = std::fs::remove_file(&path);

    let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
    let base = file.create_base("Base", 3, 3).expect("create base");
    let _f1 = base.write_family("Wing").expect("write family");
    let _f2 = base.write_family("Fuselage").expect("write family");
    let _f3 = base.write_family("Tail").expect("write family");
    drop(file);

    let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
    let base = file.base("Base").expect("find base");
    assert_eq!(base.family_count().expect("family count"), 3);

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// 1-to-1 zone interface — write two zones and read back count
// ---------------------------------------------------------------------------
#[test]
fn test_1to1_connectivity() {
    let path = test_path("1to1_connectivity");
    let _ = std::fs::remove_file(&path);

    let verts = vec![0.0f64; 16];
    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
        let base = file.create_base("Base", 3, 3).expect("create base");

        let zone_a = base
            .create_zone_structured("ZoneA", &[2, 2, 2, 1, 1, 1, 0, 0, 0])
            .expect("create zone A");
        zone_a.write_coord_f64("X", &verts).expect("write X");

        let zone_b = base
            .create_zone_structured("ZoneB", &[2, 2, 2, 1, 1, 1, 0, 0, 0])
            .expect("create zone B");
        zone_b.write_coord_f64("X", &verts).expect("write X");

        let range = [2i64, 1, 1, 2, 2, 2];
        let donor_range = [1i64, 1, 1, 1, 2, 2];
        let transform = [1i32, 2, 3];
        zone_a
            .write_1to1("AB_interface", "ZoneB", &range, &donor_range, &transform)
            .expect("write 1to1");
    }

    let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
    let base = file.base("Base").expect("find base");
    let zones = base.zones().expect("zones");
    let zone_a = &zones[0];
    assert_eq!(zone_a.n1to1().expect("n1to1"), 1);

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Base and Zone metadata readback (cell_dim, phys_dim, size, index_dim)
// ---------------------------------------------------------------------------
#[test]
fn test_base_zone_metadata() {
    let path = test_path("base_zone_metadata");
    let _ = std::fs::remove_file(&path);

    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
        let base = file.create_base("Base", 3, 3).expect("create base");

        // Structured zone (3D)
        let _zone = base
            .create_zone_structured("Structured", &[4, 3, 2, 3, 2, 1, 0, 0, 0])
            .expect("create structured zone");

        // Unstructured zone
        let _uzone = base
            .create_zone_unstructured("Unstructured", 10, 5)
            .expect("create unstructured zone");
    }

    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
        let base = file.base("Base").expect("find base");
        assert_eq!(base.cell_dim().expect("cell_dim"), 3);
        assert_eq!(base.phys_dim().expect("phys_dim"), 3);

        let zones = base.zones().expect("zones");
        assert_eq!(zones.len(), 2);

        // Structured zone
        let szone = &zones[0];
        assert_eq!(szone.zone_type().expect("zone_type"), ZoneType::Structured);
        assert_eq!(szone.index_dim().expect("index_dim"), 3);
        let ssize = szone.size().expect("size");
        assert_eq!(ssize.len(), 9);
        assert_eq!(ssize[0], 4);
        assert_eq!(ssize[1], 3);
        assert_eq!(ssize[2], 2);

        // Unstructured zone
        let uzone = &zones[1];
        assert_eq!(
            uzone.zone_type().expect("zone_type"),
            ZoneType::Unstructured
        );
        assert_eq!(uzone.index_dim().expect("index_dim"), 1);
        let usize = uzone.size().expect("size");
        assert_eq!(usize.len(), 3);
        assert_eq!(usize[0], 10);
        assert_eq!(usize[1], 5);
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Solution and field metadata readback
// ---------------------------------------------------------------------------
#[test]
fn test_solution_field_metadata() {
    let path = test_path("solution_field_metadata");
    let _ = std::fs::remove_file(&path);

    let nverts: i64 = 8;
    let data_f64: Vec<f64> = vec![0.0; nverts as usize];
    let data_f32: Vec<f32> = vec![0.0; nverts as usize];
    let data_i32: Vec<i32> = vec![0; nverts as usize];
    let data_i64: Vec<i64> = vec![0; nverts as usize];

    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
        let base = file.create_base("Base", 3, 3).expect("create base");
        let zone = base
            .create_zone_unstructured("Zone", nverts, 0)
            .expect("create zone");
        zone.write_coord_f64("X", &data_f64).expect("write X");

        let sol = zone
            .write_solution("FlowSolution", GridLocation::Vertex)
            .expect("write solution");
        zone.write_field_f32(&sol, "Pressure", &data_f32)
            .expect("write Pressure");
        zone.write_field_i32(&sol, "Index", &data_i32)
            .expect("write Index");
        zone.write_field_i64(&sol, "GID", &data_i64)
            .expect("write GID");
    }

    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
        let base = file.base("Base").expect("find base");
        let zone = &base.zones().expect("zones")[0];
        let sol = zone.solution("FlowSolution").expect("find solution");
        let info = sol.info().expect("sol info");
        assert_eq!(info.name, "FlowSolution");
        assert_eq!(info.location, GridLocation::Vertex);

        let names = sol.field_names().expect("field names");
        assert_eq!(names, vec!["Pressure", "Index", "GID"]);

        let finfos = sol.field_info_list().expect("field info list");
        assert_eq!(finfos.len(), 3);
        assert_eq!(finfos[0].name, "Pressure");
        assert_eq!(finfos[0].data_type, cgns::data::DataType::R4);
        assert_eq!(finfos[1].data_type, cgns::data::DataType::I4);
        assert_eq!(finfos[2].data_type, cgns::data::DataType::I8);
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Units, DataClass, SimulationType readback
// ---------------------------------------------------------------------------
#[test]
fn test_units_dataclass_simulation_type() {
    let path = test_path("units_dataclass");
    let _ = std::fs::remove_file(&path);

    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
        let base = file.create_base("Base", 3, 3).expect("create base");

        use cgns::data::*;
        base.write_units(&UnitsSystem::new(
            MassUnits::Kilogram,
            LengthUnits::Meter,
            TimeUnits::Second,
            TemperatureUnits::Kelvin,
            AngleUnits::Radian,
        ))
        .expect("write units");

        base.write_dataclass(DataClass::Dimensional)
            .expect("write dataclass");

        base.write_simulation_type(SimulationType::NonTimeAccurate)
            .expect("write simulation_type");
    }

    {
        let file = CgnsFile::open(&path.to_string_lossy()).expect("open");
        let base = file.base("Base").expect("find base");

        let units = base.read_units().expect("read units");
        assert_eq!(units.mass, cgns::data::MassUnits::Kilogram);
        assert_eq!(units.length, cgns::data::LengthUnits::Meter);
        assert_eq!(units.time, cgns::data::TimeUnits::Second);
        assert_eq!(units.temperature, cgns::data::TemperatureUnits::Kelvin);
        assert_eq!(units.angle, cgns::data::AngleUnits::Radian);

        let dc = base.read_dataclass().expect("read dataclass");
        assert_eq!(dc, cgns::data::DataClass::Dimensional);

        let st = base.read_simulation_type().expect("read sim type");
        assert_eq!(st, cgns::data::SimulationType::NonTimeAccurate);
    }

    std::fs::remove_file(&path).ok();
}

// ---------------------------------------------------------------------------
// Various data types — write coordinates as f32, i32, i64
// ---------------------------------------------------------------------------
#[test]
fn test_write_coord_f32() {
    let path = test_path("write_coord_f32");
    let _ = std::fs::remove_file(&path);

    let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    {
        let file = CgnsFile::create(&path.to_string_lossy()).expect("create");
        let base = file.create_base("Base", 3, 3).expect("create base");
        let zone = base
            .create_zone_structured("Zone", &[2, 2, 2, 1, 1, 1, 0, 0, 0])
            .expect("create zone");
        zone.write_coord_f32("X", &data).expect("write X f32");
    }

    std::fs::remove_file(&path).ok();
}
