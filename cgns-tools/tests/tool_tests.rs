use std::path::PathBuf;
use std::process::Command;
use std::sync::Once;

fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().unwrap().to_path_buf()
}

fn binary_path(tool: &str) -> PathBuf {
    let exe = if cfg!(windows) {
        format!("{}.exe", tool)
    } else {
        tool.to_string()
    };
    workspace_root().join("target").join("debug").join(exe)
}

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
}

static BUILD: Once = Once::new();
fn ensure_built() {
    BUILD.call_once(|| {
        let status = Command::new("cargo")
            .args(["build", "-p", "cgns-tools", "-q"])
            .current_dir(workspace_root())
            .status()
            .expect("failed to run cargo build");
        assert!(status.success(), "cargo build for cgns-tools failed");
    });
}

#[test]
fn test_cgns_list() {
    ensure_built();
    let bin = binary_path("cgns-list");
    let path = data_dir().join("test_grid.cgns");
    let output = Command::new(&bin)
        .arg(&path)
        .output()
        .expect("failed to run cgns-list");
    assert!(output.status.success(), "cgns-list failed: {}",
        String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    // The tree view uses +- prefixes; pick a few known entries
    assert!(stdout.contains("BaseStructured2D"), "missing BaseStructured2D");
    assert!(stdout.contains("BaseUnstructured"), "missing BaseUnstructured");
    assert!(stdout.contains("ElementConnectivity"), "missing connectivity");
}

#[test]
fn test_cgns_check() {
    ensure_built();
    let bin = binary_path("cgns-check");
    let path = data_dir().join("test_grid.cgns");
    let output = Command::new(&bin)
        .arg(&path)
        .output()
        .expect("failed to run cgns-check");
    assert!(output.status.success(), "cgns-check failed: {}",
        String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_cgns_diff_self() {
    ensure_built();
    // HDF5 cannot open the same file twice simultaneously, so copy first.
    let src = data_dir().join("test_grid.cgns");
    let copy = data_dir().join("test_grid_copy.cgns");
    std::fs::copy(&src, &copy).expect("copy test file");

    let output = Command::new(binary_path("cgns-diff"))
        .arg(&src)
        .arg(&copy)
        .output()
        .expect("failed to run cgns-diff");
    let _ = std::fs::remove_file(&copy);

    assert!(output.status.success(), "cgns-diff (copy) failed: {}",
        String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim().is_empty(), "unexpected diff output: {}", stdout);
}

#[test]
fn test_cgns_compress() {
    ensure_built();
    let src = data_dir().join("test_grid.cgns");
    let dst = data_dir().join("test_grid_compressed.cgns");
    let _ = std::fs::remove_file(&dst);

    let output = Command::new(binary_path("cgns-compress"))
        .arg(&src)
        .arg(&dst)
        .output()
        .expect("failed to run cgns-compress");
    assert!(output.status.success(), "cgns-compress failed: {}",
        String::from_utf8_lossy(&output.stderr));
    assert!(dst.exists(), "compressed file not created");
    assert!(dst.metadata().map(|m| m.len() > 0).unwrap_or(false),
        "compressed file is empty");

    let check_output = Command::new(binary_path("cgns-check"))
        .arg(&dst)
        .output()
        .expect("failed to run cgns-check on compressed file");
    assert!(check_output.status.success(), "compressed file fails cgns-check");

    let _ = std::fs::remove_file(&dst);
}

#[test]
fn test_cgns_names() {
    ensure_built();
    let bin = binary_path("cgns-names");
    let output = Command::new(&bin)
        .output()
        .expect("failed to run cgns-names");
    assert!(output.status.success(), "cgns-names failed: {}",
        String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Density"), "cgns-names missing Density");
    assert!(stdout.contains("Pressure"), "cgns-names missing Pressure");
}
