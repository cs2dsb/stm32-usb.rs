use packed_struct_codegen::PrimitiveEnum;

#[derive(Clone, Copy, Eq, PartialEq, Debug, PrimitiveEnum)]
pub enum Direction {
    ToDevice = 0x00,
    ToHost = 0x80,
}