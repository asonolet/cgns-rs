/// Build script for cgns-tools.
///
/// Compiles each CGNS C tool source into a static library (with `main`
/// renamed via `-Dmain=<tool>_main`), then each Rust binary provides the
/// `main()` that calls the corresponding C entry point via FFI.
///
/// Link directives for CGNS and HDF5 are emitted here because
/// `cargo:rustc-link-lib` from `cgns-sys` does NOT propagate to dependents
/// (only `rustc-link-search` propagates).
use std::env;
use std::path::PathBuf;

/// Tools and their source files.
const TOOLS: &[(&str, &[&str])] = &[
    ("cgnslist", &["cgnslist.c"]),
    ("cgnscheck", &["cgnscheck.c"]),
    ("cgnsconvert", &["cgnsconvert.c"]),
    ("cgnsdiff", &["cgnsdiff.c"]),
    ("cgnscompress", &["cgnscompress.c"]),
    ("cgnsnames", &["cgnsnames.c"]),
];

/// Common source files needed by all (or most) tools.
const COMMON_SOURCES: &[&str] = &["getargs.c", "hash.c", "cgnames.c"];

/// Files with extra warnings to suppress (from upstream CGNS code).
const WARN_SUPPRESS_EXTRA: &[(&str, &[&str])] = &[
    ("cgnscheck.c", &["-Wno-unused-but-set-variable"]),
    ("cgnsnames.c", &["-Wno-unused-parameter"]),
];

fn target_is_windows_msvc() -> bool {
    let target = env::var("TARGET").unwrap_or_default();
    target.contains("windows") && !target.contains("gnu")
}

fn main() {
    // The tools source directory is emitted by cgns-sys's build script via
    // cargo's `links` metadata as `DEP_CGNS_TOOLS_SRC_DIR`.
    let tools_src_dir = PathBuf::from(
        env::var("DEP_CGNS_TOOLS_SRC_DIR")
            .expect("DEP_CGNS_TOOLS_SRC_DIR not set — cgns-sys build may have failed"),
    );
    // Include directories for CGNS and HDF5 headers.
    let include_dirs = env::var("DEP_CGNS_INCLUDE_DIR").expect("DEP_CGNS_INCLUDE_DIR not set");
    let cgns_include: PathBuf = include_dirs.split(':').next().unwrap().into();
    let hdf5_include: PathBuf = include_dirs.split(':').nth(1).unwrap().into();
    // CGNS source dir (for internal headers like cgns_header.h)
    let cgns_src_include: PathBuf = tools_src_dir.parent().unwrap().into(); // vendor/cgns/src/
                                                                            // CGNS build dir (for generated headers like cg_hash_types.h)
    let cgns_build_include: PathBuf = {
        let vendor = tools_src_dir
            .parent()
            .unwrap() // src/
            .parent()
            .unwrap() // cgns/
            .parent()
            .unwrap(); // vendor/
        vendor.join("cgns-build").join("src")
    };

    // Compile common sources into a shared static library.
    let mut shared = cc::Build::new();
    shared.include(&cgns_include);
    shared.include(&hdf5_include);
    shared.include(&tools_src_dir);
    shared.include(&cgns_src_include);
    shared.include(&cgns_build_include);
    for src in COMMON_SOURCES {
        shared.file(tools_src_dir.join(src));
    }
    shared.compile("cgns_tools_common");

    // Compile each tool's source (with renamed main) into its own static lib.
    for (tool, sources) in TOOLS {
        let main_name = format!("{}_main", tool);
        let mut build = cc::Build::new();
        build.include(&cgns_include);
        build.include(&hdf5_include);
        build.include(&tools_src_dir);
        build.include(&cgns_src_include);
        build.include(&cgns_build_include);
        build.define("main", Some(main_name.as_str()));
        for src in *sources {
            let path = tools_src_dir.join(src);
            build.file(path);
            // Suppress known upstream warnings
            if *src == "cgnscheck.c" {
                build.flag_if_supported("-Wno-unused-function");
            }
            for &(fname, flags) in WARN_SUPPRESS_EXTRA {
                if *src == fname {
                    for flag in flags {
                        build.flag_if_supported(flag);
                    }
                }
            }
        }
        build.compile(&format!("cgns_tool_{}", tool));
    }

    // --- Link native libraries -------------------------------------------
    //
    // `cgns-sys`'s build script emits `rustc-link-search` (which propagates)
    // and `rustc-link-lib` (which does NOT propagate).  We must re-emit the
    // link-lib directives here so that the final binary links CGNS, HDF5,
    // and their system dependencies.
    //
    // Search paths already come from `cgns-sys` via propagated `-L` flags.
    println!("cargo:rustc-link-lib=static=cgns");
    if target_is_windows_msvc() {
        println!("cargo:rustc-link-lib=static=libhdf5");
    } else {
        println!("cargo:rustc-link-lib=static=hdf5");
        println!("cargo:rustc-link-lib=dylib=m");
        println!("cargo:rustc-link-lib=dylib=dl");
        println!("cargo:rustc-link-lib=dylib=pthread");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
