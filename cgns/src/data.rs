

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


