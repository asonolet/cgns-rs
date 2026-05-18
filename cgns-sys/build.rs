/// Build script for cgns-sys.
///
/// ## Summary
///
/// 1. Downloads and builds HDF5 as a static library using CMake.
/// 2. Builds CGNS as a static library using CMake, linked against HDF5.
/// 3. Runs `bindgen` on `cgnslib.h` to auto-generate Rust FFI bindings.
/// 4. Emits `cargo:` link directives so the final binary links both libs.
///
/// ## Updating CGNS
///
/// To update to a newer CGNS version:
///   - Update the git submodule: `git -C cgns-sys/vendor/cgns checkout <new-tag>`
///   - The build script will automatically pick up the new sources.
///
/// If the CGNS API changed, `bindgen` re-runs automatically (the build script
/// is re-executed when any file under `vendor/cgns` is modified, see
/// `cargo:rerun-if-changed=` directives near the bottom).
///
/// ## HDF5 version
///
/// The HDF5 version is pinned in the `HDF5_VERSION` constant below.
/// To update, change the constant and the download URL.
///
/// ## bindgen
///
/// `bindgen` parses the C headers and generates Rust FFI declarations
/// (extern "C" functions, #[repr(C)] structs, constants, etc.).
/// The generated code is written to `$OUT_DIR/bindings.rs` and included
/// from `src/lib.rs`.  Any changes to `wrapper.h` or the CGNS/HDF5 headers
/// will trigger a regeneration.
///
/// ## Prerequisites
///
///   - CMake >= 3.20
///   - A C compiler (gcc, clang, MSVC, …)
///   - curl and tar (for downloading/extracting HDF5 source)
///
/// On Debian/Ubuntu: `apt install cmake build-essential curl`
/// On macOS: `brew install cmake`
/// On Windows: install CMake from https://cmake.org and ensure curl + tar
///   are available (they are on Windows 10+).
///
/// ## Environment variables
///
///   - `HDF5_DIR` – if set, the build script skips downloading/building HDF5
///     and uses the specified directory as the HDF5 installation prefix.
///     This is useful for using a system-provided HDF5.
///   - `CGNS_RS_NO_BUILD_HDF5` – if set to "1", forces skipping the HDF5
///     build step (useful when `HDF5_DIR` is already set).
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// HDF5 version to download & build when a system installation is not used.
const HDF5_VERSION: &str = "1.14.6";

/// Download URL for the HDF5 source tarball.
const HDF5_URL: &str =
    "https://github.com/HDFGroup/hdf5/archive/refs/tags/hdf5-1.14.6.tar.gz";

/// Minimum required CMake version.
const CMAKE_MIN_VER: &str = "3.20";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Run a command, print it for debugging, and fail on non-zero exit.
fn run<S>(cmd: &str, args: &[S], description: &str)
where
    S: AsRef<OsStr>,
{
    println!(
        "cargo:warning=  running: {} {}",
        cmd,
        args.iter()
            .map(|a| a.as_ref().to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(" ")
    );
    let status = Command::new(cmd)
        .args(args)
        .status()
        .unwrap_or_else(|e| panic!("failed to execute `{}`: {}", cmd, e));
    if !status.success() {
        panic!("{} failed (exit: {}): {}", cmd, status, description);
    }
}

/// Check that cmake is available and recent enough.
fn check_cmake() {
    let out = Command::new("cmake")
        .arg("--version")
        .output()
        .expect("cmake not found — install cmake >= 3.20");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let ver_line = stdout.lines().next().unwrap_or("");
    let ver_str = ver_line
        .split_whitespace()
        .nth(2)
        .unwrap_or("0.0.0");
    // Simple semver comparison (enough for our check)
    fn parse_ver(v: &str) -> Vec<u32> {
        v.split('.')
            .map(|x| x.parse::<u32>().unwrap_or(0))
            .collect()
    }
    let have = parse_ver(ver_str);
    let need = parse_ver(CMAKE_MIN_VER);
    if have < need {
        panic!(
            "cmake {} found but >= {} required",
            ver_str, CMAKE_MIN_VER
        );
    }
    println!("cargo:warning=cmake {} found at {}", ver_str, which("cmake"));
}

/// Simple `which`-like lookup (Unix-only fallback).
fn which(name: &str) -> String {
    let path = env::var_os("PATH").unwrap_or_default();
    for dir in env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return candidate.to_string_lossy().to_string();
        }
        #[cfg(windows)]
        {
            let candidate_exe = dir.join(format!("{}.exe", name));
            if candidate_exe.is_file() {
                return candidate_exe.to_string_lossy().to_string();
            }
        }
    }
    name.to_string()
}

/// Download a URL to a file using curl (or fetch on FreeBSD).
fn download(url: &str, dest: &Path) {
    if dest.exists() {
        println!("cargo:warning=  already downloaded: {}", dest.display());
        return;
    }
    let dir = dest.parent().unwrap();
    fs::create_dir_all(dir).ok();

    // Prefer curl, fall back to wget.
    if Command::new("curl").arg("--version").output().is_ok() {
        run(
            "curl",
            &["-fL", "-o", &dest.to_string_lossy(), url] as &[&str],
            "download HDF5 source",
        );
    } else if Command::new("wget").arg("--version").output().is_ok() {
        run(
            "wget",
            &["-O", &dest.to_string_lossy(), url] as &[&str],
            "download HDF5 source",
        );
    } else {
        panic!("neither curl nor wget found — install one of them to download HDF5");
    }
}

/// Extract a `.tar.gz` archive into `dest_dir`.
fn extract_tar_gz(archive: &Path, dest_dir: &Path) {
    fs::create_dir_all(dest_dir).ok();
    run(
        "tar",
        &["-xzf", &archive.to_string_lossy(), "-C", &dest_dir.to_string_lossy()],
        "extract HDF5 source archive",
    );
}

/// Source directory name inside the extracted HDF5 tarball.
fn hdf5_src_dirname() -> String {
    format!("hdf5-hdf5-{}", HDF5_VERSION)
}

/// Determine the library name for HDF5 depending on platform.
fn hdf5_lib_name() -> &'static str {
    // On MSVC, HDF5 outputs `libhdf5.lib`; on other platforms `libhdf5.a`.
    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("windows") && !target.contains("gnu") {
        "libhdf5"
    } else {
        "hdf5"
    }
}

/// Detect the target operating system family.
fn target_is_windows_msvc() -> bool {
    let target = env::var("TARGET").unwrap_or_default();
    target.contains("windows") && !target.contains("gnu")
}

// ---------------------------------------------------------------------------
// Build steps
// ---------------------------------------------------------------------------

/// Build HDF5 as a static library.
///
/// Returns the installation prefix path.
fn build_hdf5(vendor_dir: &Path) -> PathBuf {
    // Allow user override via HDF5_DIR
    if let Ok(hdf5_dir) = env::var("HDF5_DIR") {
        let dir = PathBuf::from(hdf5_dir);
        if dir.join("lib").join(format!(
            "{}libhdf5.{}",
            if cfg!(unix) { "lib" } else { "" },
            if cfg!(windows) { "lib" } else { "a" },
        ))
        .exists()
            || dir.join("lib").join(format!("libhdf5.a")).exists()
            || dir.join("lib").join("hdf5.lib").exists()
            || dir.join("lib").join("libhdf5.lib").exists()
        {
            println!("cargo:warning=using system HDF5 from HDF5_DIR={}", dir.display());
            return dir;
        }
        // HDF5_DIR set but no library found — we will still use it as a hint
        // (the CGNS cmake will fail if it can't find HDF5, which is an
        // acceptable error).
        println!("cargo:warning=HDF5_DIR set but no HDF5 library found in {:?}", dir);
    }

    if env::var("CGNS_RS_NO_BUILD_HDF5").as_deref() == Ok("1") {
        panic!(
            "CGNS_RS_NO_BUILD_HDF5=1 but no HDF5_DIR pointing to a valid \
             installation was provided"
        );
    }

    let build_dir = vendor_dir.join("hdf5-build");
    let install_dir = vendor_dir.join("hdf5-install");
    let src_dir = vendor_dir.join(hdf5_src_dirname());

    if !src_dir.join("CMakeLists.txt").exists() {
        // Download and extract HDF5 sources
        let archive = vendor_dir.join(format!("hdf5-{}.tar.gz", HDF5_VERSION));
        println!(
            "cargo:warning=Downloading HDF5 {} (this may take a while)...",
            HDF5_VERSION
        );
        download(HDF5_URL, &archive);
        extract_tar_gz(&archive, vendor_dir);
    }

    if install_dir.join("lib").join(format!("lib{}.a", hdf5_lib_name())).exists()
        || install_dir.join("lib").join(format!("{}.lib", hdf5_lib_name())).exists()
    {
        println!("cargo:warning=HDF5 already built, skipping build");
        return install_dir;
    }

    println!("cargo:warning=Configuring HDF5 {}...", HDF5_VERSION);

    // Remove stale build directory
    if build_dir.exists() {
        fs::remove_dir_all(&build_dir).unwrap();
    }
    fs::create_dir_all(&build_dir).unwrap();
    fs::create_dir_all(&install_dir).unwrap();

    let src_dir_s = src_dir.to_string_lossy().to_string();
    let build_dir_s = build_dir.to_string_lossy().to_string();
    let install_prefix = format!("-DCMAKE_INSTALL_PREFIX={}", install_dir.to_string_lossy());

    let mut cmake_args: Vec<&str> = vec![
        &src_dir_s,
        "-B",
        &build_dir_s,
        "-DCMAKE_BUILD_TYPE=Release",
        "-DBUILD_SHARED_LIBS=OFF",
        "-DHDF5_BUILD_FORTRAN=OFF",
        "-DHDF5_BUILD_EXAMPLES=OFF",
        "-DHDF5_BUILD_TOOLS=OFF",
        "-DHDF5_ENABLE_Z_LIB_SUPPORT=OFF",
        "-DHDF5_ENABLE_SZIP_SUPPORT=OFF",
        "-DHDF5_BUILD_UTILS=OFF",
        "-DHDF5_BUILD_HL_LIB=OFF",
        "-DBUILD_TESTING=OFF",
        "-DCMAKE_POSITION_INDEPENDENT_CODE=ON",
        &install_prefix,
    ];

    // On MSVC, ensure static runtime is used (optional, but avoids
    // MSVCRT conflicts in Rust builds).
    if target_is_windows_msvc() {
        cmake_args.push("-DCMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded");
    }

    run("cmake", &cmake_args, "configure HDF5");

    println!("cargo:warning=Building HDF5...");
    run(
        "cmake",
        &[
            "--build",
            &build_dir.to_string_lossy(),
            "--config",
            "Release",
            "--target",
            "hdf5-static",
            "-j",
            &num_cpus(),
        ],
        "build HDF5",
    );

    println!("cargo:warning=Installing HDF5...");
    run(
        "cmake",
        &[
            "--install",
            &build_dir.to_string_lossy(),
            "--config",
            "Release",
            "--prefix",
            &install_dir.to_string_lossy(),
        ],
        "install HDF5",
    );

    install_dir
}

/// Build CGNS as a static library with HDF5 support.
///
/// Returns the installation prefix path.
fn build_cgns(vendor_dir: &Path, hdf5_install: &Path) -> PathBuf {
    let cgns_src = vendor_dir.join("cgns");
    let build_dir = vendor_dir.join("cgns-build");
    let install_dir = vendor_dir.join("cgns-install");

    if install_dir.join("lib").join("libcgns.a").exists()
        || install_dir.join("lib").join("cgns.lib").exists()
    {
        println!("cargo:warning=CGNS already built, skipping build");
        return install_dir;
    }

    if build_dir.exists() {
        fs::remove_dir_all(&build_dir).unwrap();
    }
    fs::create_dir_all(&build_dir).unwrap();
    fs::create_dir_all(&install_dir).unwrap();

    println!("cargo:warning=Configuring CGNS...");

    let cgns_src_s = cgns_src.to_string_lossy().to_string();
    let build_dir_s = build_dir.to_string_lossy().to_string();
    let install_prefix = format!("-DCMAKE_INSTALL_PREFIX={}", install_dir.to_string_lossy());
    let hdf5_root = format!("-DHDF5_ROOT={}", hdf5_install.to_string_lossy());
    let build_flag = format!("-B{}", build_dir_s);

    let mut cmake_args: Vec<&str> = vec![
        &cgns_src_s,
        &build_flag,
        "-DCMAKE_BUILD_TYPE=Release",
        "-DCGNS_BUILD_SHARED=OFF",
        "-DCGNS_ENABLE_HDF5=ON",
        "-DCMAKE_POSITION_INDEPENDENT_CODE=ON",
        &install_prefix,
        &hdf5_root,
    ];

    // On MSVC, link with static runtime
    if target_is_windows_msvc() {
        cmake_args.push("-DCMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded");
    }

    run("cmake", &cmake_args, "configure CGNS");

    println!("cargo:warning=Building CGNS...");
    run(
        "cmake",
        &[
            "--build",
            &build_dir.to_string_lossy(),
            "--config",
            "Release",
            "-j",
            &num_cpus(),
        ],
        "build CGNS",
    );

    println!("cargo:warning=Installing CGNS...");
    run(
        "cmake",
        &[
            "--install",
            &build_dir.to_string_lossy(),
            "--config",
            "Release",
            "--prefix",
            &install_dir.to_string_lossy(),
        ],
        "install CGNS",
    );

    install_dir
}

/// Emit cargo link directives for both HDF5 and CGNS.
fn emit_link_directives(hdf5_install: &Path, cgns_install: &Path) {
    let lib_dirs = [
        ("HDF5", hdf5_install),
        ("CGNS", cgns_install),
    ];

    for (name, prefix) in &lib_dirs {
        let lib_dir = prefix.join("lib");
        if lib_dir.exists() {
            println!(
                "cargo:rustc-link-search=native={}",
                lib_dir.to_string_lossy()
            );
        } else {
            let lib64 = prefix.join("lib64");
            if lib64.exists() {
                println!(
                    "cargo:rustc-link-search=native={}",
                    lib64.to_string_lossy()
                );
            } else {
                panic!(
                    "{} library directory not found under {}",
                    name,
                    prefix.display()
                );
            }
        }
    }

    // CGNS must come BEFORE HDF5 in link order for static linking:
    // the linker resolves CGNS symbols against HDF5, so CGNS must be
    // processed first (left-to-right resolution).
    println!("cargo:rustc-link-lib=static=cgns");

    if target_is_windows_msvc() {
        println!("cargo:rustc-link-lib=static=libhdf5");
    } else {
        println!("cargo:rustc-link-lib=static=hdf5");
    }

    // System libraries needed by HDF5 on Unix
    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("linux") || target.contains("apple") {
        println!("cargo:rustc-link-lib=dylib=m");
        println!("cargo:rustc-link-lib=dylib=dl");
        println!("cargo:rustc-link-lib=dylib=pthread");
    }
}

/// Generate Rust FFI bindings using bindgen.
fn generate_bindings(cgns_install: &Path, hdf5_install: &Path) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let cgns_inc = cgns_install.join("include");
    let hdf5_inc = hdf5_install.join("include");

    let bindings = bindgen::Builder::default()
        // wrapper.h includes cgnslib.h which pulls in all CGNS headers
        .header("wrapper.h")
        .clang_args(["-I", &cgns_inc.to_string_lossy()])
        .clang_args(["-I", &hdf5_inc.to_string_lossy()])
        // Tell bindgen to use cargo's rustc-link-lib (it won't emit anything
        // for CGNS/HDF5 though, since we handle that ourselves).
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Suppress some CGNS-specific warnings
        .allowlist_function("cg_.*")
        .allowlist_function("cgio_.*")
        .allowlist_type("cgsize_t")
        .allowlist_type("cgerr_t")
        .allowlist_type("cgint_t")
        .allowlist_type("cgid_t")
        .allowlist_type("cgint_f")
        .allowlist_type("cglong_t")
        .allowlist_type("cgulong_t")
        .allowlist_type("CGNS_ENUM.*")
        .allowlist_type("CGNS_ENUMT.*")
        .allowlist_var("CG_.*")
        .allowlist_var("CGNS_.*")
        .blocklist_function("^cg_pt.*")
        .derive_default(true)
        .generate()
        .expect("bindgen failed to generate CGNS bindings");

    let output = out_dir.join("bindings.rs");
    bindings
        .write_to_file(&output)
        .expect("bindgen failed to write bindings");

    println!("cargo:warning=bindings written to {}", output.display());
}

/// Return a string like "4" (number of available CPUs + 1).
fn num_cpus() -> String {
    let n = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2);
    (n + 1).to_string()
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let vendor_dir = manifest_dir.join("vendor");

    println!("cargo:warning=Starting CGNS build (cgns-sys)");

    // 1. Prerequisites
    check_cmake();

    // 2. Build HDF5 (downloads source if needed)
    let hdf5_install = build_hdf5(&vendor_dir);

    // 3. Build CGNS from the vendored submodule
    let cgns_install = build_cgns(&vendor_dir, &hdf5_install);

    // 4. Emit link directives
    emit_link_directives(&hdf5_install, &cgns_install);

    // 5. Generate Rust bindings
    generate_bindings(&cgns_install, &hdf5_install);

    // Tell cargo when to re-run the build script
    // (CGNS source changes or wrapper.h changes)
    let cgns_src = vendor_dir.join("cgns");
    if cgns_src.exists() {
        println!(
            "cargo:rerun-if-changed={}",
            cgns_src.to_string_lossy()
        );
    }
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:warning=CGNS build complete");
}
