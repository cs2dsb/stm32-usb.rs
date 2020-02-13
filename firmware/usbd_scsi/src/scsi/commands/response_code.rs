use packed_struct_codegen::PrimitiveEnum;

#[derive(Clone, Copy, Eq, PartialEq, Debug, PrimitiveEnum)]
pub enum ResponseCode {
    FixedSenseData = 0x70,
    DescriptorSenseData = 0x72,    
}
impl Default for ResponseCode {
    fn default() -> Self {
        ResponseCode::FixedSenseData
    }
}