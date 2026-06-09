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
    pub const fn to_raw(self) -> u32 {
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

    pub const fn from_raw(raw: u32) -> Option<Self> {
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
    pub const fn to_raw(self) -> u32 {
        match self {
            Self::I4 => cgns_sys::DataType_t_Integer,
            Self::I8 => cgns_sys::DataType_t_LongInteger,
            Self::R4 => cgns_sys::DataType_t_RealSingle,
            Self::R8 => cgns_sys::DataType_t_RealDouble,
            Self::Char => cgns_sys::DataType_t_Character,
        }
    }

    pub const fn from_raw(raw: u32) -> Option<Self> {
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
    pub const fn to_raw(self) -> u32 {
        match self {
            Self::Structured => cgns_sys::ZoneType_t_Structured,
            Self::Unstructured => cgns_sys::ZoneType_t_Unstructured,
        }
    }

    pub const fn from_raw(raw: u32) -> Option<Self> {
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
    pub const fn to_raw(self) -> u32 {
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

    pub const fn from_raw(raw: u32) -> Option<Self> {
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
        let _guard = match cgns_sys::lock_cgns() {
            Ok(g) => g,
            Err(_) => return 0,
        };
        let mut n: i32 = 0;
        if unsafe { cgns_sys::cg_npe(self.to_raw(), &mut n) } == cgns_sys::CG_OK as i32 {
            n
        } else {
            0
        }
    }
}

/// CGNS boundary condition type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BcType {
    Null,
    UserDefined,
    AxisymmetricWedge,
    DegenerateLine,
    DegeneratePoint,
    Dirichlet,
    Extrapolate,
    Farfield,
    General,
    Inflow,
    InflowSubsonic,
    InflowSupersonic,
    Neumann,
    Outflow,
    OutflowSubsonic,
    OutflowSupersonic,
    SymmetryPlane,
    SymmetryPolar,
    TunnelInflow,
    TunnelOutflow,
    Wall,
    WallInviscid,
    WallViscous,
    WallViscousHeatFlux,
    WallViscousIsothermal,
}

impl BcType {
    pub const fn to_raw(self) -> u32 {
        match self {
            Self::Null => cgns_sys::BCType_t_BCTypeNull,
            Self::UserDefined => cgns_sys::BCType_t_BCTypeUserDefined,
            Self::AxisymmetricWedge => cgns_sys::BCType_t_BCAxisymmetricWedge,
            Self::DegenerateLine => cgns_sys::BCType_t_BCDegenerateLine,
            Self::DegeneratePoint => cgns_sys::BCType_t_BCDegeneratePoint,
            Self::Dirichlet => cgns_sys::BCType_t_BCDirichlet,
            Self::Extrapolate => cgns_sys::BCType_t_BCExtrapolate,
            Self::Farfield => cgns_sys::BCType_t_BCFarfield,
            Self::General => cgns_sys::BCType_t_BCGeneral,
            Self::Inflow => cgns_sys::BCType_t_BCInflow,
            Self::InflowSubsonic => cgns_sys::BCType_t_BCInflowSubsonic,
            Self::InflowSupersonic => cgns_sys::BCType_t_BCInflowSupersonic,
            Self::Neumann => cgns_sys::BCType_t_BCNeumann,
            Self::Outflow => cgns_sys::BCType_t_BCOutflow,
            Self::OutflowSubsonic => cgns_sys::BCType_t_BCOutflowSubsonic,
            Self::OutflowSupersonic => cgns_sys::BCType_t_BCOutflowSupersonic,
            Self::SymmetryPlane => cgns_sys::BCType_t_BCSymmetryPlane,
            Self::SymmetryPolar => cgns_sys::BCType_t_BCSymmetryPolar,
            Self::TunnelInflow => cgns_sys::BCType_t_BCTunnelInflow,
            Self::TunnelOutflow => cgns_sys::BCType_t_BCTunnelOutflow,
            Self::Wall => cgns_sys::BCType_t_BCWall,
            Self::WallInviscid => cgns_sys::BCType_t_BCWallInviscid,
            Self::WallViscous => cgns_sys::BCType_t_BCWallViscous,
            Self::WallViscousHeatFlux => cgns_sys::BCType_t_BCWallViscousHeatFlux,
            Self::WallViscousIsothermal => cgns_sys::BCType_t_BCWallViscousIsothermal,
        }
    }

    pub const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            x if x == cgns_sys::BCType_t_BCTypeNull => Some(Self::Null),
            x if x == cgns_sys::BCType_t_BCTypeUserDefined => Some(Self::UserDefined),
            x if x == cgns_sys::BCType_t_BCAxisymmetricWedge => Some(Self::AxisymmetricWedge),
            x if x == cgns_sys::BCType_t_BCDegenerateLine => Some(Self::DegenerateLine),
            x if x == cgns_sys::BCType_t_BCDegeneratePoint => Some(Self::DegeneratePoint),
            x if x == cgns_sys::BCType_t_BCDirichlet => Some(Self::Dirichlet),
            x if x == cgns_sys::BCType_t_BCExtrapolate => Some(Self::Extrapolate),
            x if x == cgns_sys::BCType_t_BCFarfield => Some(Self::Farfield),
            x if x == cgns_sys::BCType_t_BCGeneral => Some(Self::General),
            x if x == cgns_sys::BCType_t_BCInflow => Some(Self::Inflow),
            x if x == cgns_sys::BCType_t_BCInflowSubsonic => Some(Self::InflowSubsonic),
            x if x == cgns_sys::BCType_t_BCInflowSupersonic => Some(Self::InflowSupersonic),
            x if x == cgns_sys::BCType_t_BCNeumann => Some(Self::Neumann),
            x if x == cgns_sys::BCType_t_BCOutflow => Some(Self::Outflow),
            x if x == cgns_sys::BCType_t_BCOutflowSubsonic => Some(Self::OutflowSubsonic),
            x if x == cgns_sys::BCType_t_BCOutflowSupersonic => Some(Self::OutflowSupersonic),
            x if x == cgns_sys::BCType_t_BCSymmetryPlane => Some(Self::SymmetryPlane),
            x if x == cgns_sys::BCType_t_BCSymmetryPolar => Some(Self::SymmetryPolar),
            x if x == cgns_sys::BCType_t_BCTunnelInflow => Some(Self::TunnelInflow),
            x if x == cgns_sys::BCType_t_BCTunnelOutflow => Some(Self::TunnelOutflow),
            x if x == cgns_sys::BCType_t_BCWall => Some(Self::Wall),
            x if x == cgns_sys::BCType_t_BCWallInviscid => Some(Self::WallInviscid),
            x if x == cgns_sys::BCType_t_BCWallViscous => Some(Self::WallViscous),
            x if x == cgns_sys::BCType_t_BCWallViscousHeatFlux => Some(Self::WallViscousHeatFlux),
            x if x == cgns_sys::BCType_t_BCWallViscousIsothermal => {
                Some(Self::WallViscousIsothermal)
            }
            _ => None,
        }
    }
}

/// CGNS mass units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MassUnits {
    Null,
    UserDefined,
    Kilogram,
    Gram,
    Slug,
    PoundMass,
}

impl MassUnits {
    pub const fn to_raw(self) -> u32 {
        match self {
            Self::Null => cgns_sys::MassUnits_t_MassUnitsNull,
            Self::UserDefined => cgns_sys::MassUnits_t_MassUnitsUserDefined,
            Self::Kilogram => cgns_sys::MassUnits_t_Kilogram,
            Self::Gram => cgns_sys::MassUnits_t_Gram,
            Self::Slug => cgns_sys::MassUnits_t_Slug,
            Self::PoundMass => cgns_sys::MassUnits_t_PoundMass,
        }
    }

    pub const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            x if x == cgns_sys::MassUnits_t_MassUnitsNull => Some(Self::Null),
            x if x == cgns_sys::MassUnits_t_MassUnitsUserDefined => Some(Self::UserDefined),
            x if x == cgns_sys::MassUnits_t_Kilogram => Some(Self::Kilogram),
            x if x == cgns_sys::MassUnits_t_Gram => Some(Self::Gram),
            x if x == cgns_sys::MassUnits_t_Slug => Some(Self::Slug),
            x if x == cgns_sys::MassUnits_t_PoundMass => Some(Self::PoundMass),
            _ => None,
        }
    }
}

/// CGNS length units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthUnits {
    Null,
    UserDefined,
    Meter,
    Centimeter,
    Millimeter,
    Foot,
    Inch,
}

impl LengthUnits {
    pub const fn to_raw(self) -> u32 {
        match self {
            Self::Null => cgns_sys::LengthUnits_t_LengthUnitsNull,
            Self::UserDefined => cgns_sys::LengthUnits_t_LengthUnitsUserDefined,
            Self::Meter => cgns_sys::LengthUnits_t_Meter,
            Self::Centimeter => cgns_sys::LengthUnits_t_Centimeter,
            Self::Millimeter => cgns_sys::LengthUnits_t_Millimeter,
            Self::Foot => cgns_sys::LengthUnits_t_Foot,
            Self::Inch => cgns_sys::LengthUnits_t_Inch,
        }
    }

    pub const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            x if x == cgns_sys::LengthUnits_t_LengthUnitsNull => Some(Self::Null),
            x if x == cgns_sys::LengthUnits_t_LengthUnitsUserDefined => Some(Self::UserDefined),
            x if x == cgns_sys::LengthUnits_t_Meter => Some(Self::Meter),
            x if x == cgns_sys::LengthUnits_t_Centimeter => Some(Self::Centimeter),
            x if x == cgns_sys::LengthUnits_t_Millimeter => Some(Self::Millimeter),
            x if x == cgns_sys::LengthUnits_t_Foot => Some(Self::Foot),
            x if x == cgns_sys::LengthUnits_t_Inch => Some(Self::Inch),
            _ => None,
        }
    }
}

/// CGNS time units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeUnits {
    Null,
    UserDefined,
    Second,
}

impl TimeUnits {
    pub const fn to_raw(self) -> u32 {
        match self {
            Self::Null => cgns_sys::TimeUnits_t_TimeUnitsNull,
            Self::UserDefined => cgns_sys::TimeUnits_t_TimeUnitsUserDefined,
            Self::Second => cgns_sys::TimeUnits_t_Second,
        }
    }

    pub const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            x if x == cgns_sys::TimeUnits_t_TimeUnitsNull => Some(Self::Null),
            x if x == cgns_sys::TimeUnits_t_TimeUnitsUserDefined => Some(Self::UserDefined),
            x if x == cgns_sys::TimeUnits_t_Second => Some(Self::Second),
            _ => None,
        }
    }
}

/// CGNS temperature units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemperatureUnits {
    Null,
    UserDefined,
    Kelvin,
    Celsius,
    Rankine,
    Fahrenheit,
}

impl TemperatureUnits {
    pub const fn to_raw(self) -> u32 {
        match self {
            Self::Null => cgns_sys::TemperatureUnits_t_TemperatureUnitsNull,
            Self::UserDefined => cgns_sys::TemperatureUnits_t_TemperatureUnitsUserDefined,
            Self::Kelvin => cgns_sys::TemperatureUnits_t_Kelvin,
            Self::Celsius => cgns_sys::TemperatureUnits_t_Celsius,
            Self::Rankine => cgns_sys::TemperatureUnits_t_Rankine,
            Self::Fahrenheit => cgns_sys::TemperatureUnits_t_Fahrenheit,
        }
    }

    pub const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            x if x == cgns_sys::TemperatureUnits_t_TemperatureUnitsNull => Some(Self::Null),
            x if x == cgns_sys::TemperatureUnits_t_TemperatureUnitsUserDefined => {
                Some(Self::UserDefined)
            }
            x if x == cgns_sys::TemperatureUnits_t_Kelvin => Some(Self::Kelvin),
            x if x == cgns_sys::TemperatureUnits_t_Celsius => Some(Self::Celsius),
            x if x == cgns_sys::TemperatureUnits_t_Rankine => Some(Self::Rankine),
            x if x == cgns_sys::TemperatureUnits_t_Fahrenheit => Some(Self::Fahrenheit),
            _ => None,
        }
    }
}

/// CGNS angle units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AngleUnits {
    Null,
    UserDefined,
    Degree,
    Radian,
}

impl AngleUnits {
    pub const fn to_raw(self) -> u32 {
        match self {
            Self::Null => cgns_sys::AngleUnits_t_AngleUnitsNull,
            Self::UserDefined => cgns_sys::AngleUnits_t_AngleUnitsUserDefined,
            Self::Degree => cgns_sys::AngleUnits_t_Degree,
            Self::Radian => cgns_sys::AngleUnits_t_Radian,
        }
    }

    pub const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            x if x == cgns_sys::AngleUnits_t_AngleUnitsNull => Some(Self::Null),
            x if x == cgns_sys::AngleUnits_t_AngleUnitsUserDefined => Some(Self::UserDefined),
            x if x == cgns_sys::AngleUnits_t_Degree => Some(Self::Degree),
            x if x == cgns_sys::AngleUnits_t_Radian => Some(Self::Radian),
            _ => None,
        }
    }
}

/// CGNS data classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataClass {
    Null,
    UserDefined,
    Dimensional,
    NormalizedByDimensional,
    NormalizedByUnknownDimensional,
    NondimensionalParameter,
    DimensionlessConstant,
}

impl DataClass {
    pub const fn to_raw(self) -> u32 {
        match self {
            Self::Null => cgns_sys::DataClass_t_DataClassNull,
            Self::UserDefined => cgns_sys::DataClass_t_DataClassUserDefined,
            Self::Dimensional => cgns_sys::DataClass_t_Dimensional,
            Self::NormalizedByDimensional => cgns_sys::DataClass_t_NormalizedByDimensional,
            Self::NormalizedByUnknownDimensional => {
                cgns_sys::DataClass_t_NormalizedByUnknownDimensional
            }
            Self::NondimensionalParameter => cgns_sys::DataClass_t_NondimensionalParameter,
            Self::DimensionlessConstant => cgns_sys::DataClass_t_DimensionlessConstant,
        }
    }

    pub const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            x if x == cgns_sys::DataClass_t_DataClassNull => Some(Self::Null),
            x if x == cgns_sys::DataClass_t_DataClassUserDefined => Some(Self::UserDefined),
            x if x == cgns_sys::DataClass_t_Dimensional => Some(Self::Dimensional),
            x if x == cgns_sys::DataClass_t_NormalizedByDimensional => {
                Some(Self::NormalizedByDimensional)
            }
            x if x == cgns_sys::DataClass_t_NormalizedByUnknownDimensional => {
                Some(Self::NormalizedByUnknownDimensional)
            }
            x if x == cgns_sys::DataClass_t_NondimensionalParameter => {
                Some(Self::NondimensionalParameter)
            }
            x if x == cgns_sys::DataClass_t_DimensionlessConstant => {
                Some(Self::DimensionlessConstant)
            }
            _ => None,
        }
    }
}

/// Type of grid connectivity interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridConnectivityType {
    Null,
    UserDefined,
    Overset,
    Abutting,
    Abutting1to1,
}

impl GridConnectivityType {
    pub const fn to_raw(self) -> u32 {
        match self {
            Self::Null => cgns_sys::GridConnectivityType_t_GridConnectivityTypeNull,
            Self::UserDefined => cgns_sys::GridConnectivityType_t_GridConnectivityTypeUserDefined,
            Self::Overset => cgns_sys::GridConnectivityType_t_Overset,
            Self::Abutting => cgns_sys::GridConnectivityType_t_Abutting,
            Self::Abutting1to1 => cgns_sys::GridConnectivityType_t_Abutting1to1,
        }
    }

    pub const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            x if x == cgns_sys::GridConnectivityType_t_GridConnectivityTypeNull => Some(Self::Null),
            x if x == cgns_sys::GridConnectivityType_t_GridConnectivityTypeUserDefined => {
                Some(Self::UserDefined)
            }
            x if x == cgns_sys::GridConnectivityType_t_Overset => Some(Self::Overset),
            x if x == cgns_sys::GridConnectivityType_t_Abutting => Some(Self::Abutting),
            x if x == cgns_sys::GridConnectivityType_t_Abutting1to1 => Some(Self::Abutting1to1),
            _ => None,
        }
    }
}

/// CGNS simulation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationType {
    Null,
    UserDefined,
    TimeAccurate,
    NonTimeAccurate,
}

impl SimulationType {
    pub const fn to_raw(self) -> u32 {
        match self {
            Self::Null => cgns_sys::SimulationType_t_SimulationTypeNull,
            Self::UserDefined => cgns_sys::SimulationType_t_SimulationTypeUserDefined,
            Self::TimeAccurate => cgns_sys::SimulationType_t_TimeAccurate,
            Self::NonTimeAccurate => cgns_sys::SimulationType_t_NonTimeAccurate,
        }
    }

    pub const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            x if x == cgns_sys::SimulationType_t_SimulationTypeNull => Some(Self::Null),
            x if x == cgns_sys::SimulationType_t_SimulationTypeUserDefined => {
                Some(Self::UserDefined)
            }
            x if x == cgns_sys::SimulationType_t_TimeAccurate => Some(Self::TimeAccurate),
            x if x == cgns_sys::SimulationType_t_NonTimeAccurate => Some(Self::NonTimeAccurate),
            _ => None,
        }
    }
}

/// A system of physical units for the base.
#[derive(Debug, Clone, PartialEq)]
pub struct UnitsSystem {
    pub mass: MassUnits,
    pub length: LengthUnits,
    pub time: TimeUnits,
    pub temperature: TemperatureUnits,
    pub angle: AngleUnits,
}

impl UnitsSystem {
    pub const fn new(
        mass: MassUnits,
        length: LengthUnits,
        time: TimeUnits,
        temperature: TemperatureUnits,
        angle: AngleUnits,
    ) -> Self {
        Self {
            mass,
            length,
            time,
            temperature,
            angle,
        }
    }
}

/// How a point set is specified for a boundary condition or connectivity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointSetType {
    PointList,
    PointListDonor,
    PointRange,
    PointRangeDonor,
    ElementRange,
    ElementList,
    CellListDonor,
}

impl PointSetType {
    pub const fn to_raw(self) -> u32 {
        match self {
            Self::PointList => cgns_sys::PointSetType_t_PointList,
            Self::PointListDonor => cgns_sys::PointSetType_t_PointListDonor,
            Self::PointRange => cgns_sys::PointSetType_t_PointRange,
            Self::PointRangeDonor => cgns_sys::PointSetType_t_PointRangeDonor,
            Self::ElementRange => cgns_sys::PointSetType_t_ElementRange,
            Self::ElementList => cgns_sys::PointSetType_t_ElementList,
            Self::CellListDonor => cgns_sys::PointSetType_t_CellListDonor,
        }
    }

    pub const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            x if x == cgns_sys::PointSetType_t_PointList => Some(Self::PointList),
            x if x == cgns_sys::PointSetType_t_PointListDonor => Some(Self::PointListDonor),
            x if x == cgns_sys::PointSetType_t_PointRange => Some(Self::PointRange),
            x if x == cgns_sys::PointSetType_t_PointRangeDonor => Some(Self::PointRangeDonor),
            x if x == cgns_sys::PointSetType_t_ElementRange => Some(Self::ElementRange),
            x if x == cgns_sys::PointSetType_t_ElementList => Some(Self::ElementList),
            x if x == cgns_sys::PointSetType_t_CellListDonor => Some(Self::CellListDonor),
            _ => None,
        }
    }
}
