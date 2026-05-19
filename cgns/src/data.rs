/// Where on the mesh a solution variable is stored.
///
/// | Variant | Meaning |
/// |---|---|
/// | `Vertex` | Value defined at grid vertices (nodes) |
/// | `CellCenter` | Value defined at cell centres |
/// | `FaceCenter` | Value defined at face centres (3-D) |
/// | `IFaceCenter`, `JFaceCenter`, `KFaceCenter` | Value on constant-i/j/k faces |
/// | `EdgeCenter` | Value at edge centres |
/// | `Null` | Unspecified / placeholder |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridLocation {
    Vertex,
    CellCenter,
    FaceCenter,
    IFaceCenter,
    JFaceCenter,
    KFaceCenter,
    EdgeCenter,
    Null,
}

impl GridLocation {
    pub fn to_raw(self) -> u32 {
        match self {
            Self::Vertex => cgns_sys::GridLocation_t_Vertex,
            Self::CellCenter => cgns_sys::GridLocation_t_CellCenter,
            Self::FaceCenter => cgns_sys::GridLocation_t_FaceCenter,
            Self::IFaceCenter => cgns_sys::GridLocation_t_IFaceCenter,
            Self::JFaceCenter => cgns_sys::GridLocation_t_JFaceCenter,
            Self::KFaceCenter => cgns_sys::GridLocation_t_KFaceCenter,
            Self::EdgeCenter => cgns_sys::GridLocation_t_EdgeCenter,
            Self::Null => cgns_sys::GridLocation_t_GridLocationNull,
        }
    }

    pub fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            x if x == cgns_sys::GridLocation_t_Vertex => Some(Self::Vertex),
            x if x == cgns_sys::GridLocation_t_CellCenter => Some(Self::CellCenter),
            x if x == cgns_sys::GridLocation_t_FaceCenter => Some(Self::FaceCenter),
            x if x == cgns_sys::GridLocation_t_IFaceCenter => Some(Self::IFaceCenter),
            x if x == cgns_sys::GridLocation_t_JFaceCenter => Some(Self::JFaceCenter),
            x if x == cgns_sys::GridLocation_t_KFaceCenter => Some(Self::KFaceCenter),
            x if x == cgns_sys::GridLocation_t_EdgeCenter => Some(Self::EdgeCenter),
            x if x == cgns_sys::GridLocation_t_GridLocationNull => Some(Self::Null),
            _ => None,
        }
    }
}

/// CGNS data-type identifier (integer sizes, float sizes, character).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    I4,
    I8,
    R4,
    R8,
    Char,
}

impl DataType {
    pub fn to_raw(self) -> u32 {
        match self {
            Self::I4 => cgns_sys::DataType_t_Integer,
            Self::I8 => cgns_sys::DataType_t_LongInteger,
            Self::R4 => cgns_sys::DataType_t_RealSingle,
            Self::R8 => cgns_sys::DataType_t_RealDouble,
            Self::Char => cgns_sys::DataType_t_Character,
        }
    }

    pub fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            x if x == cgns_sys::DataType_t_Integer => Some(Self::I4),
            x if x == cgns_sys::DataType_t_LongInteger => Some(Self::I8),
            x if x == cgns_sys::DataType_t_RealSingle => Some(Self::R4),
            x if x == cgns_sys::DataType_t_RealDouble => Some(Self::R8),
            x if x == cgns_sys::DataType_t_Character => Some(Self::Char),
            _ => None,
        }
    }
}

/// Whether a zone uses structured (ijk) or unstructured (element-based) topology.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneType {
    Structured,
    Unstructured,
}

impl ZoneType {
    pub fn to_raw(self) -> u32 {
        match self {
            Self::Structured => cgns_sys::ZoneType_t_Structured,
            Self::Unstructured => cgns_sys::ZoneType_t_Unstructured,
        }
    }

    pub fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            x if x == cgns_sys::ZoneType_t_Structured => Some(Self::Structured),
            x if x == cgns_sys::ZoneType_t_Unstructured => Some(Self::Unstructured),
            _ => None,
        }
    }
}

/// CGNS element type for unstructured mesh sections.
///
/// Each variant includes the number of vertices after the underscore
/// (e.g. `Tri3` has 3 vertices, `Tetra4` has 4).  Use [`npe()`] to
/// query the canonical number of nodes per element at runtime.
///
/// # Connectivity layout
///
/// For a section of type `Tri3` with 2 elements, the flat connectivity is:
///
/// ```text
/// [tri1_v1, tri1_v2, tri1_v3, tri2_v1, tri2_v2, tri2_v3]
/// ```
///
/// All node indices are **1-based** vertex IDs within the zone.
///
/// # Special types
///
/// | Variant | Meaning |
/// |---|---|
/// | `Mixed` | Elements of varying types in one section (uses a leading type-flag per element) |
/// | `NgoneN` | Polygon with `n` vertices (variable per element) |
/// | `NfaceN` | Polyhedron with `n` faces (variable per element) |
/// | `Null` / `UserDefined` | Placeholder / user-defined element type |
///
/// [`npe()`]: ElementType::npe
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementType {
    Null,
    UserDefined,
    Node,
    Bar2,
    Bar3,
    Tri3,
    Tri6,
    Quad4,
    Quad8,
    Quad9,
    Tetra4,
    Tetra10,
    Pyra5,
    Pyra14,
    Penta6,
    Penta15,
    Penta18,
    Hexa8,
    Hexa20,
    Hexa27,
    Mixed,
    NgoneN,
    NfaceN,
}

impl ElementType {
    pub fn to_raw(self) -> u32 {
        match self {
            Self::Null => cgns_sys::ElementType_t_ElementTypeNull,
            Self::UserDefined => cgns_sys::ElementType_t_ElementTypeUserDefined,
            Self::Node => cgns_sys::ElementType_t_NODE,
            Self::Bar2 => cgns_sys::ElementType_t_BAR_2,
            Self::Bar3 => cgns_sys::ElementType_t_BAR_3,
            Self::Tri3 => cgns_sys::ElementType_t_TRI_3,
            Self::Tri6 => cgns_sys::ElementType_t_TRI_6,
            Self::Quad4 => cgns_sys::ElementType_t_QUAD_4,
            Self::Quad8 => cgns_sys::ElementType_t_QUAD_8,
            Self::Quad9 => cgns_sys::ElementType_t_QUAD_9,
            Self::Tetra4 => cgns_sys::ElementType_t_TETRA_4,
            Self::Tetra10 => cgns_sys::ElementType_t_TETRA_10,
            Self::Pyra5 => cgns_sys::ElementType_t_PYRA_5,
            Self::Pyra14 => cgns_sys::ElementType_t_PYRA_14,
            Self::Penta6 => cgns_sys::ElementType_t_PENTA_6,
            Self::Penta15 => cgns_sys::ElementType_t_PENTA_15,
            Self::Penta18 => cgns_sys::ElementType_t_PENTA_18,
            Self::Hexa8 => cgns_sys::ElementType_t_HEXA_8,
            Self::Hexa20 => cgns_sys::ElementType_t_HEXA_20,
            Self::Hexa27 => cgns_sys::ElementType_t_HEXA_27,
            Self::Mixed => cgns_sys::ElementType_t_MIXED,
            Self::NgoneN => cgns_sys::ElementType_t_NGON_n,
            Self::NfaceN => cgns_sys::ElementType_t_NFACE_n,
        }
    }

    pub fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            x if x == cgns_sys::ElementType_t_ElementTypeNull => Some(Self::Null),
            x if x == cgns_sys::ElementType_t_ElementTypeUserDefined => Some(Self::UserDefined),
            x if x == cgns_sys::ElementType_t_NODE => Some(Self::Node),
            x if x == cgns_sys::ElementType_t_BAR_2 => Some(Self::Bar2),
            x if x == cgns_sys::ElementType_t_BAR_3 => Some(Self::Bar3),
            x if x == cgns_sys::ElementType_t_TRI_3 => Some(Self::Tri3),
            x if x == cgns_sys::ElementType_t_TRI_6 => Some(Self::Tri6),
            x if x == cgns_sys::ElementType_t_QUAD_4 => Some(Self::Quad4),
            x if x == cgns_sys::ElementType_t_QUAD_8 => Some(Self::Quad8),
            x if x == cgns_sys::ElementType_t_QUAD_9 => Some(Self::Quad9),
            x if x == cgns_sys::ElementType_t_TETRA_4 => Some(Self::Tetra4),
            x if x == cgns_sys::ElementType_t_TETRA_10 => Some(Self::Tetra10),
            x if x == cgns_sys::ElementType_t_PYRA_5 => Some(Self::Pyra5),
            x if x == cgns_sys::ElementType_t_PYRA_14 => Some(Self::Pyra14),
            x if x == cgns_sys::ElementType_t_PENTA_6 => Some(Self::Penta6),
            x if x == cgns_sys::ElementType_t_PENTA_15 => Some(Self::Penta15),
            x if x == cgns_sys::ElementType_t_PENTA_18 => Some(Self::Penta18),
            x if x == cgns_sys::ElementType_t_HEXA_8 => Some(Self::Hexa8),
            x if x == cgns_sys::ElementType_t_HEXA_20 => Some(Self::Hexa20),
            x if x == cgns_sys::ElementType_t_HEXA_27 => Some(Self::Hexa27),
            x if x == cgns_sys::ElementType_t_MIXED => Some(Self::Mixed),
            x if x == cgns_sys::ElementType_t_NGON_n => Some(Self::NgoneN),
            x if x == cgns_sys::ElementType_t_NFACE_n => Some(Self::NfaceN),
            _ => None,
        }
    }

    /// Return the canonical number of nodes (vertices) per element.
    ///
    /// Calls the CGNS library's [`cg_npe`] internally.
    ///
    /// Returns `0` for variable-size types (`Mixed`, `NgoneN`, `NfaceN`)
    /// and for unknown types.
    pub fn npe(self) -> i32 {
        let _guard = cgns_sys::lock_cgns();
        let mut n: i32 = 0;
        if unsafe { cgns_sys::cg_npe(self.to_raw(), &mut n) } == cgns_sys::CG_OK as i32 {
            n
        } else {
            0
        }
    }
}
