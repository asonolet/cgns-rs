/*
 * wrapper.h
 *
 * This header is the single entry point for `bindgen`.  It includes the
 * CGNS public API header.  The include paths (-I flags) are provided
 * by build.rs, pointing to the CGNS and HDF5 installation directories.
 *
 * To add more CGNS-related declarations to the generated bindings,
 * add the corresponding #include here and re-run the build.
 */
#include "cgnslib.h"
