use packed_struct_codegen::PrimitiveEnum;

#[derive(Clone, Copy, Eq, PartialEq, Debug, PrimitiveEnum)]
pub enum MediumType {
    Sbc = 0x00,
}

impl Default for MediumType {
    fn default() -> Self {
        MediumType::Sbc
    }
}