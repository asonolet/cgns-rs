/// Enum mapping CGNS data types to Rust types.
pub enum CgnsDataType {
    Integer(i32),
    RealSingle(f32),
    RealDouble(f64),
    Character(String),
    LongInteger(i64),
}
