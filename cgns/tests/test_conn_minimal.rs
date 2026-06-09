use cgns::CgnsFile;

/// Minimal test: write a general connectivity, close, re-open, read back.
/// Ensures the round-trip works without crash or wrong-sized buffers.
#[test]
fn test_conn_minimal() {
    let path = "/tmp/test_conn_minimal.cgns";
    let _ = std::fs::remove_file(path);

    let verts = vec![0.0f64; 8];

    /* ---------- write ---------- */
    {
        let file = CgnsFile::create(path).expect("create");
        let base = file.create_base("Base", 3, 3).expect("create base");
        let zone = base
            .create_zone_structured("Zone", &[2, 2, 2, 1, 1, 1, 0, 0, 0])
            .expect("create zone");
        zone.write_coord_f64("X", &verts).expect("write X");

        let points: Vec<i64> = vec![1, 1, 1, 1, 2, 2];
        let donor: Vec<i64> = vec![1, 1, 1, 1, 2, 2];
        zone.write_conn(
            "TestConn",
            cgns::data::GridLocation::Vertex,
            cgns::data::GridConnectivityType::Abutting,
            cgns::data::PointSetType::PointRange,
            2,
            &points,
            "Zone",
            cgns::data::ZoneType::Structured,
            cgns::data::PointSetType::PointListDonor,
            2,
            &donor,
        )
        .expect("write general connection");
    }

    /* ---------- read ---------- */
    {
        let file = CgnsFile::open(path).expect("open");
        let base = file.base("Base").expect("find base");
        let zone = &base.zones().expect("zones")[0];

        assert_eq!(zone.nconns().expect("nconns"), 1);
        let conn = zone.conn("TestConn").expect("find conn by name");
        let info = conn.info().expect("conn info");
        assert_eq!(info.name, "TestConn");
        assert_eq!(info.num_points, 2);
        assert_eq!(info.num_donor_data, 2);

        let data = conn.read_data(3).expect("read_data");
        assert_eq!(data.points, vec![1i64, 1, 1, 1, 2, 2]);
        assert_eq!(data.donor_data, vec![1i64, 1, 1, 1, 2, 2]);
    }

    std::fs::remove_file(path).ok();
}
