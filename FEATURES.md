# CGNS Features & Development Plan

> Comprehensive list of CGNS v4.5.2 features and their implementation status in `cgns-rs`.
> **Legend:** ✅ High-level API (`cgns`) | 🟡 FFI wrapper (`cgns-sys`) | 🔲 Not yet bound | ❌ Not applicable / blocked

---

## 1. File Operations

| Feature | Status | Notes |
|---|---|---|
| Open for reading | ✅ `CgnsFile::open` | |
| Create new file | ✅ `CgnsFile::create` | HDF5 backend forced |
| Open for modify | ✅ `CgnsFile::modify` | |
| Close | ✅ `CgnsFile::close` + `Drop` | Double-close safe |
| File type selection | 🟡 `ensure_hdf5_backend` | HDF5 only; ADF not exposed |
| File save / flush | 🔲 | `cg_save` not wrapped |
| File version query | 🔲 | `cg_version`, `cg_get_file_type` |

---

## 2. Bases

| Feature | Status | Notes |
|---|---|---|
| Create | ✅ `Base` via `CgnsFile::create_base` | |
| Count | ✅ `base_count` | |
| Enumerate | ✅ `bases()` | |
| Query by name | ✅ `base(name)` | |
| Read metadata | 🔲 | cell_dim, phys_dim not exposed |
| Delete | 🔲 | `cg_base_delete` not wrapped |

---

## 3. Zones

| Feature | Status | Notes |
|---|---|---|
| Structured zone create | ✅ `create_zone_structured` | |
| Unstructured zone create | ✅ `create_zone_unstructured` | |
| Count | ✅ `zone_count` | |
| Enumerate | ✅ `zones()` | |
| Query zone type | ✅ `zone_type()` | |
| Read zone size | 🔲 | `cg_zone_read` not wrapped |
| Zone delete | 🔲 | `cg_zone_delete` not wrapped |
| Zone iterative data | 🔲 | `cg_ziter_write`/`read` |

---

## 4. Grid Coordinates

| Feature | Status | Notes |
|---|---|---|
| Write f64 | ✅ `write_coord_f64` | |
| Write f32 | ✅ `write_coord_f32` | |
| Read f64 full | ✅ `read_coord_f64` | Sub-range via rmin/rmax |
| Read f64 sub-range | ✅ | |
| Read f32 | 🔲 | |
| Coord info (names, dims) | ✅ `coord_names`, `coord_count` | |
| Grid coordinates (`cg_grid_*`) | 🔲 | Grid family info not wrapped |

---

## 5. Element Sections (Unstructured)

| Feature | Status | Notes |
|---|---|---|
| Write section | ✅ `Zone::write_section` | |
| Count | ✅ `section_count` | |
| Enumerate | ✅ `sections()` | |
| Query by name | ✅ `section(name)` | |
| Read metadata | ✅ `SectionInfo` | name, type, range, nbndry, parent_flag |
| Read connectivity | ✅ `read_connectivity` → `Vec<i64>` | |
| Read connectivity ndarray | ✅ `read_connectivity_ndarray` → `Array2<i64>` | ndarray integration |
| Element count | ✅ `element_count` | |
| NPE query | ✅ `ElementType::npe()` | Calls `cg_npe` |
| Element types enum | ✅ 22 variants | Null, Bar2, Tri3, … NfaceN |
| Mixed elements | 🔲 | Mixed element sections |
| Parent data | 🔲 | Parent elements for boundary |
| Section delete | 🔲 | `cg_section_delete` not wrapped |

---

## 6. Solutions & Fields

| Feature | Status | Notes |
|---|---|---|
| Write solution | ✅ `Zone::write_solution` | |
| Count | ✅ `solution_count` | |
| Enumerate | ✅ `solutions()` | |
| Query by name | ✅ `solution(name)` | |
| Write field f64 | ✅ `Solution::write_field_f64` | |
| Write field f32 | ✅ `Zone::write_field_f32` | Via Zone |
| Write field i32 | ✅ `Zone::write_field_i32` | Via Zone |
| Write field i64 | ✅ `Zone::write_field_i64` | Via Zone |
| Read field f64 full | ✅ `Solution::read_field_f64` | |
| Read field f64 sub-range | ✅ | rmin/rmax |
| Read field f32 | 🔲 | |
| Read field i32 | 🔲 | |
| Read field i64 | 🔲 | |
| Field info (count, names) | ✅ `field_count` | |
| Grid location enum | ✅ 8 variants | Vertex, CellCenter, … |
| Solution delete | 🔲 | `cg_sol_delete` not wrapped |

---

## 7. Boundary Conditions

| Feature | Status | Notes |
|---|---|---|
| Write | ✅ `Zone::write_bc` | PointList or PointRange |
| Count | ✅ `bc_count` | |
| Enumerate | ✅ `bcs()` | |
| Query by name | ✅ `bc(name)` | |
| Read metadata | ✅ `BcInfo` | name, type, point_set_type, npnts |
| Read point data | ✅ `read_points` → `Vec<i64>` | |
| BC types enum | ✅ 24 variants | Wall, Inflow, Farfield, … |
| Point set types enum | ✅ 4 variants | PointList, PointRange, ElementRange, ElementList |
| BC dataset (`BCDataSet`) | 🔲 | `cg_bcdataset_*` not wrapped |
| BC data types | 🔲 | `cg_bcdata_write` not wrapped |
| BC delete | 🔲 | `cg_boco_delete` not wrapped |

---

## 8. Zone Connectivity

### 8.1 1-to-1 Interfaces

| Feature | Status | Notes |
|---|---|---|
| Write | ✅ `Zone::write_1to1` | |
| Count | ✅ `n1to1` | |
| Enumerate | ✅ `one_to_ones()` | |
| Read metadata | ✅ `OneToOneInfo` | name, donor, range, donor_range, transform |

### 8.2 General Connectivity (`cg_conn_*`)

| Feature | Status | Notes |
|---|---|---|
| Write general connectivity | 🔲 | `cg_conn_write` |
| Read general connectivity | 🔲 | `cg_conn_read` |
| Enumerate connections | 🔲 | `cg_nconns` |
| Periodic connections | 🔲 | |

### 8.3 Overset Connectivity

| Feature | Status | Notes |
|---|---|---|
| Overset holes | 🔲 | `cg_hole_*` |
| Overset fringe | 🔲 | |

---

## 9. Families

| Feature | Status | Notes |
|---|---|---|
| Write family | ✅ `Base::write_family` | |
| Count | ✅ `family_count` | |
| Read family names | ✅ `family_names` | |
| Family tree | 🔲 | `cg_family_tree_*` |
| Family name assignment | 🔲 | `cg_famname_write` on nodes |
| Family BC references | 🔲 | `cg_fambc_*` |
| Family user data | 🔲 | `cg_famuserdata_*` |
| Family geometry references | 🔲 | `cg_geometry_*` |

---

## 10. Iterative / Unsteady Data

| Feature | Status | Notes |
|---|---|---|
| Base iterative data | 🔲 | `cg_biter_write`, `cg_biter_read` |
| Zone iterative data | 🔲 | `cg_ziter_write`, `cg_ziter_read` |
| State (time values) | 🔲 | `cg_state_write`, `cg_state_read` |
| Convergence history | 🔲 | `cg_convergence_write`, `cg_convergence_read` |
| Integrals | 🔲 | `cg_integral_write`, `cg_integral_read` |

---

## 11. Reference State & Descriptors

| Feature | Status | Notes |
|---|---|---|
| Reference state | 🔲 | `cg_governing_*`, `cg_equationset_*` |
| Descriptor text nodes | 🔲 | `cg_descriptor_*` |
| User-defined data | 🔲 | `cg_user_data_write` |

---

## 12. Discrete Data

| Feature | Status | Notes |
|---|---|---|
| Discrete data write | 🔲 | `cg_discrete_*` |
| Discrete data read | 🔲 | |

---

## 13. Grid Motion

| Feature | Status | Notes |
|---|---|---|
| Rigid grid motion | 🔲 | `cg_gridmotion_*` |
| Arbitrary grid motion | 🔲 | |

---

## 14. Links

| Feature | Status | Notes |
|---|---|---|
| Write link node | 🔲 | `cg_link_write` |
| Read link node | 🔲 | `cg_link_read` |

---

## 15. Low-Level CGNS Tree (`cg_goto`)

| Feature | Status | Notes |
|---|---|---|
| Tree navigation | 🔲 | `cg_goto`, `cg_gorel` (raw FFI available) |
| Node info | 🔲 | `cg_nchildren`, `cg_children` |

---

## 16. The CGNS Array API (`cg_array_*`)

| Feature | Status | Notes |
|---|---|---|
| Write arrays | 🔲 | `cg_array_write` |
| Read arrays | 🔲 | `cg_array_read` |
| Array info | 🔲 | `cg_array_info` |

---

## 17. Data Types & Enums

| Feature | Status | Notes |
|---|---|---|
| `DataType` enum | ✅ I4, I8, R4, R8, Char | |
| `ZoneType` enum | ✅ Structured, Unstructured | |
| `GridLocation` enum | ✅ 8 variants | |
| `PointSetType` enum | ✅ 4 variants | |
| `ElementType` enum | ✅ 22 variants | |
| `BcType` enum | ✅ 24 variants | |
| `to_raw()` / `from_raw()` conversion | ✅ All enums | |

---

## 18. Error Handling

| Feature | Status | Notes |
|---|---|---|
| `CgnsError` enum | ✅ | CgnsCode, Invalid, NotFound, Io |
| `CgnsResult<T>` | ✅ | Type alias |
| Error message from C lib | 🟡 `cg_get_error` | Exposed via `error_message()` |
| Structured error codes | 🟡 | C error code propagation |

---

## 19. Thread Safety

| Feature | Status | Notes |
|---|---|---|
| Global Mutex | 🟡 `CGNS_MUTEX` in `cgns-sys` | Serialises all FFI calls |
| `lock_cgns()` | 🟡 | For raw FFI users |
| Auto-lock in safe wrappers | ✅ | |

---

## 20. Parallel CGNS

| Feature | Status | Notes |
|---|---|---|
| `cgp_*` parallel functions | 🔲 Blocklisted in bindgen | MPI-dependent; requires MPI build |

---

## 21. CLI Tools

| Feature | Status | Notes |
|---|---|---|
| `cgns-list` | ✅ | Via upstream C source |
| `cgns-check` | ✅ | |
| `cgns-convert` | ✅ | |
| `cgns-diff` | ✅ | |
| `cgns-compress` | ✅ | |
| `cgns-names` | ✅ | |

---

# Development Plan

## Phase 1: Polish Current API (Short-term)

| Priority | Task | Effort |
|---|---|---|
| P0 | Add field reads for f32, i32, i64 types | Small |
| P0 | Add f32 coordinate read | Small |
| P0 | Expose zone size read (`cg_zone_read`) | Small |
| P0 | Expose cell_dim and phys_dim on `Base` | Small |
| P1 | Read field / coord via ndarray | Small |
| P1 | Add `CgnsFile::save` / flush | Small |
| P1 | Add `Zone::coord_info` (data type per coord) | Small |

## Phase 2: Remaining Core Nodes (Medium-term)

| Priority | Task | Effort |
|---|---|---|
| P0 | General connectivity (`cg_conn_*`): write, read, enumerate | Medium |
| P1 | BC datasets (`cg_bcdataset_*`): write, read | Medium |
| P1 | BC data on BC (`cg_bcdata_write`/`read`) | Medium |
| P1 | Descriptors (`cg_descriptor_*`): write, read | Small |
| P1 | User-defined data nodes (`cg_user_data`) | Small |
| P2 | Families: family tree, famname on nodes, fam BC refs | Medium |
| P2 | Element parent data | Small |
| P2 | Mixed element sections | Medium |

## Phase 3: Unsteady / Iterative Data (Medium-term)

| Priority | Task | Effort |
|---|---|---|
| P1 | Base and Zone iterative data (`cg_biter_*`, `cg_ziter_*`) | Medium |
| P1 | State (time values) write/read | Small |
| P2 | Convergence history | Small |
| P2 | Integrals | Small |
| P2 | Reference state | Medium |

## Phase 4: Advanced Features (Long-term)

| Priority | Task | Effort |
|---|---|---|
| P2 | Discrete data (`cg_discrete_*`) | Medium |
| P2 | Grid motion (`cg_gridmotion_*`) | Medium |
| P2 | Links (`cg_link_*`) | Small |
| P3 | Overset connectivity (`cg_hole_*`) | Medium |
| P3 | `cg_goto` tree navigation API | Medium |
| P3 | `cg_array_*` for generic data read/write | Medium |
| P3 | Periodic connectivity | Small |

## Phase 5: Quality & Infrastructure

| Priority | Task | Effort |
|---|---|---|
| P0 | Update README coverage table (BC, connectivity, families are now done) | Small |
| P1 | Integration tests for each new feature | Ongoing |
| P1 | Round-trip fuzz testing for all types | Medium |
| P2 | Benchmark suite for large files | Medium |
| P2 | Documentation examples for all feature areas | Medium |
| P2 | CI with system HDF5 variant | Small |

## Feature Count Summary

| Category | Total Features | ✅ High-Level | 🟡 FFI Only | 🔲 Not Bound |
|---|---|---|---|---|
| File Operations | 5 | 3 | 1 | 1 |
| Bases | 6 | 4 | 0 | 2 |
| Zones | 7 | 4 | 0 | 3 |
| Grid Coordinates | 7 | 4 | 0 | 3 |
| Element Sections | 12 | 9 | 0 | 3 |
| Solutions & Fields | 15 | 9 | 0 | 6 |
| Boundary Conditions | 8 | 5 | 0 | 3 |
| 1-to-1 Connectivity | 4 | 4 | 0 | 0 |
| General Connectivity | 4 | 0 | 0 | 4 |
| Overset | 2 | 0 | 0 | 2 |
| Families | 7 | 3 | 0 | 4 |
| Iterative / Unsteady | 5 | 0 | 0 | 5 |
| Reference / Descriptors | 3 | 0 | 0 | 3 |
| Discrete Data | 2 | 0 | 0 | 2 |
| Grid Motion | 2 | 0 | 0 | 2 |
| Links | 2 | 0 | 0 | 2 |
| Low-Level Tree | 2 | 0 | 0 | 2 |
| Array API | 3 | 0 | 0 | 3 |
| Enums / Types | 6 | 6 | 0 | 0 |
| Error Handling | 4 | 2 | 2 | 0 |
| Thread Safety | 3 | 1 | 2 | 0 |
| Parallel CGNS | 1 | 0 | 0 | 1 |
| CLI Tools | 6 | 6 | 0 | 0 |
| **Total** | **114** | **60** | **5** | **49** |
