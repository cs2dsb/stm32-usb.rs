use codegen::asc_list_to_enum;

asc_list_to_enum!{
    pub AdditionalSenseCode = "src/scsi/enums/asc-num.txt";
}           

impl Default for AdditionalSenseCode {
    fn default() -> Self {
        AdditionalSenseCode::NoAdditionalSenseInformation
    }
}