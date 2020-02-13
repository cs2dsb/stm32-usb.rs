use packed_struct_codegen::PackedStruct;
use packed_struct::PackedStruct;
use crate::scsi::{
    packing::{
        ParsePackedStruct,
        ResizeSmaller,
    },
    commands::{
        version_descriptor::*,
        target_port_group_support::TargetPortGroupSupport,
        spc_version::SpcVersion,
        peripheral_qualifier::PeripheralQualifier,
        peripheral_device_type::PeripheralDeviceType,
        response_data_format::ResponseDataFormat,
    },
};



#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct, Default)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct InquiryCommand {
    /// If set, return vital data related to the page_code field
    #[packed_field(bits="7")]
    pub enable_vital_product_data: bool,
    /// What kind of vital data to return
    pub page_code: u8,
    ///TODO: (check) Should match data_transfer_length in CBW
    pub allocation_length: u16,
    pub control: u8,
}
impl<A: ResizeSmaller<[u8; InquiryCommand::BYTES]>>  ParsePackedStruct<A, [u8; InquiryCommand::BYTES]> for InquiryCommand {}

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct, Default)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct InquiryResponse {
    #[packed_field(bits="0..3", ty="enum")]
    peripheral_qualifier: PeripheralQualifier,
    #[packed_field(bits="3..8", ty="enum")]
    peripheral_device_type: PeripheralDeviceType,
    ///A removable medium ( RMB ) bit set to zero indicates that the medium is not removable. A RMB bit set to one indicates that the medium is removable.
    #[packed_field(bits="8..9")]
    removable_medium: bool,
    ///The VERSION field indicates the implemented version of this standard and is defined in table 142
    #[packed_field(bits="16..24", ty="enum")]
    version: SpcVersion,
    ///The Normal ACA Supported (NORMACA) bit set to one indicates that the device server supports a NACA bit set to one in the CDB CONTROL byte and supports the ACA task attribute (see SAM-4). A N ORM ACA bit set to zero indicates that the device server does not support a NACA bit set to one and does not support the ACA task attribute.
    #[packed_field(bits="26..27")]
    normal_aca: bool,
    ///A hierarchical support (HISUP) bit set to zero indicates the SCSI target device does not use the hierarchical addressing model to assign LUNs to logical units. A H I S UP bit set to one indicates the SCSI target device uses the hierarchical addressing model to assign LUNs to logical units.
    #[packed_field(bits="27..28")]
    hierarchical_support: bool, 
    ///The RESPONSE DATA FORMAT field indicates the format of the standard INQUIRY data and shall be set as shown in table 139. A RESPONSE DATA FORMAT field set to 2h indicates that the standard INQUIRY data is in the format defined in this standard. Response data format values less than 2h are obsolete. Response data format values greater than 2h are reserved.
    #[packed_field(bits="28..32", ty="enum")]
    response_data_format: ResponseDataFormat,
    ///The ADDITIONAL LENGTH field indicates the length in bytes of the remaining standard INQUIRY data. The relationship between the ADDITIONAL LENGTH field and the CDB ALLOCATION LENGTH field is defined in 4.3.5.6.
    ///Set to total length in bytes minus 4
    #[packed_field(bits="32..40")]
    additional_length: u8, 
    ///An SCC Supported ( SCCS ) bit set to one indicates that the SCSI target device contains an embedded storage array controller component that is addressable through this logical unit. See SCC-2 for details about storage array controller devices. An SCCS bit set to zero indicates that no embedded storage array controller component is addressable through this logical unit.
    #[packed_field(bits="40..41")]
    scc_supported: bool,
    ///An Access Controls Coordinator ( ACC ) bit set to one indicates that the SCSI target device contains an access controls coordinator (see 3.1.4) that is addressable through this logical unit. An ACC bit set to zero indicates that no access controls coordinator is addressable through this logical unit. If the SCSI target device contains an access controls coordinator that is addressable through any logical unit other than the ACCESS CONTROLS well known logical unit (see 8.3), then the ACC bit shall be set to one for LUN 0.
    #[packed_field(bits="41..42")]
    access_controls_coordinator: bool,
    ///The contents of the target port group support ( TPGS ) field (see table 143) indicate the support for asymmetric logical unit access (see 5.11).
    #[packed_field(bits="42..44", ty="enum")]
    target_port_group_support: TargetPortGroupSupport,
    ///A Third-Party Copy (3PC) bit set to one indicates that the SCSI target device contains a copy manager that is addressable through this logical unit. A 3 PC bit set to zero indicates that no copy manager is addressable through this logical unit.
    #[packed_field(bits="44..45")]
    third_party_copy: bool,
    ///A PROTECT bit set to zero indicates that the logical unit does not support protection information. A PROTECT bit set to one indicates that the logical unit supports:
    /// a) type 1 protection, type 2 protection, or type 3 protection (see SBC-3); or
    /// b) logical block protection (see SSC-4).
    ///More information about the type of protection the logical unit supports is available in the SPT field (see 7.8.7).
    #[packed_field(bits="47..48")]
    protect: bool,
    ///An Enclosure Services (ENCSERV) bit set to one indicates that the SCSI target device contains an embedded enclosure services component that is addressable through this logical unit. See SES-3 for details about enclosure services. An E NC S ERV bit set to zero indicates that no embedded enclosure services component is addressable through this logical unit.
    #[packed_field(bits="49..50")]
    enclosure_services: bool,
    #[packed_field(bits="50..51")]
    _vendor_specific: bool, 
    ///A Multi Port (MULTIP) bit set to one indicates that this is a multi-port (two or more ports) SCSI target device and conforms to the SCSI multi-port device requirements found in the applicable standards (e.g., SAM-4, a SCSI transport protocol standard and possibly provisions of a command standard). A M ULTI P bit set to zero indicates that this SCSI target device has a single port and does not implement the multi-port requirements.
    #[packed_field(bits="51..52")]
    multi_port: bool,
    /// SPI-5 only, reserved for all others
    #[packed_field(bits="55..56")]
    _addr_16: bool,
    /// SPI-5 only, reserved for all others
    #[packed_field(bits="58..59")]
    _wbus_16: bool,
    /// SPI-5 only, reserved for all others
    #[packed_field(bits="59..60")]
    _sync: bool,
    ///The CMDQUE bit shall be set to one indicating that the logical unit supports the command management model defined in SAM-4.
    #[packed_field(bits="62..63")]
    command_queue: bool,
    #[packed_field(bits="63..64")]
    _vendor_specific2: bool,
    ///The T10 VENDOR IDENTIFICATION field contains eight bytes of left-aligned ASCII data (see 4.4.1) identifying the vendor of the logical unit. The T10 vendor identification shall be one assigned by INCITS. A list of assigned T10 vendor identifications is in Annex E and on the T10 web site (http://www.t10.org).
    #[packed_field(bits="64..128")]
    vendor_identification: [u8; 8],
    ///The PRODUCT IDENTIFICATION field contains sixteen bytes of left-aligned ASCII data (see 4.4.1) defined by the vendor.
    #[packed_field(bits="128..256")]
    product_identification: [u8; 16],
    ///The PRODUCT REVISION LEVEL field contains four bytes of left-aligned ASCII data defined by the vendor.
    #[packed_field(bits="256..288")]
    product_revision_level: [u8; 4],
    #[packed_field(bits="288..448")]
    _vendor_specific3: [u8; 19],
    /// SPI-5 only, reserved for all others
    #[packed_field(bits="452..454")]
    _clocking: u8,
    /// SPI-5 only, reserved for all others
    #[packed_field(bits="454..455")]
    _qas: bool,
    /// SPI-5 only, reserved for all others
    #[packed_field(bits="455..456")]
    _ius: bool,
    ///The VERSION DESCRIPTOR fields provide for identifying up to eight standards to which the SCSI target device and/or logical unit claim conformance. The value in each VERSION DESCRIPTOR field shall be selected from table 144. All version descriptor values not listed in table 144 are reserved. Technical Committee T10 of INCITS maintains an electronic copy of the information in table 144 on its world wide web site (http://www.t10.org/). In the event that the T10 world wide web site is no longer active, access may be possible via the INCITS world wide web site (http://www.incits.org), the ANSI world wide web site (http://www.ansi.org), the IEC site (http://www.iec.ch/), the ISO site (http://www.iso.ch/), or the ISO/IEC JTC 1 web site (http://www.jtc1.org/). It is recommended that the first version descriptor be used for the SCSI architecture standard, followed by the physical transport standard if any, followed by the SCSI transport protocol standard, followed by the appropriate SPC-x version, followed by the device type command set, followed by a secondary command set if any.
    #[packed_field(bits="464..480", ty="enum")]
    compliant_standard_1: VersionDescriptor,
    #[packed_field(bits="480..496", ty="enum")]
    compliant_standard_2: VersionDescriptor,
    #[packed_field(bits="496..512", ty="enum")]
    compliant_standard_3: VersionDescriptor,
    #[packed_field(bits="512..528", ty="enum")]
    compliant_standard_4: VersionDescriptor,
    #[packed_field(bits="528..544", ty="enum")]
    compliant_standard_5: VersionDescriptor,
    #[packed_field(bits="544..560", ty="enum")]
    compliant_standard_6: VersionDescriptor,
    #[packed_field(bits="560..576", ty="enum")]
    compliant_standard_7: VersionDescriptor,
    #[packed_field(bits="576..592", ty="enum")]
    compliant_standard_8: VersionDescriptor,
}

pub fn inquiry_response() -> InquiryResponse {
    InquiryResponse {
        removable_medium: true,
        //TODO: Work out why off by 1, docs say -4 but that's one byte too long
        //      It could be that sg_inq is adding 1 for some reason, the OS hasn't
        //      actually followed up with a longer request in real use.
        additional_length: (InquiryResponse::BYTES - 4) as u8, 
        vendor_identification: [0x56, 0x65, 0x4E, 0x64, 0x4F, 0x72, 0x49, 0x64], // "VeNdOrId" in utf8,
        product_identification: [0x50, 0x72, 0x4F, 0x64, 0x55, 0x63, 0x54, 0x20, 
                                 0x69, 0x44, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20], // "PrOdUcT iD      " in utf8
        product_revision_level: [0x33, 0x2E, 0x31, 0x34], // "3.14" in utf8
        compliant_standard_1: VersionDescriptor::SAM3NoVersionClaimed,
        compliant_standard_2: VersionDescriptor::SPC4NoVersionClaimed,
        compliant_standard_3: VersionDescriptor::SBC3NoVersionClaimed,
        ..Default::default()
    }
}

/*
 if evpd 
    return data related to page_code (spc-4 section 7.8)
    if unsupported(page_code) 
        return CHECK_CONDITION and set SENSE:   
            key: ILLEGAL_REQUEST
            additional code: INVALID_FIELD_IN_CBD
 
 if !evpd
    return standard inquiry data (spc-4 section 6.4.2)
    if page_code != 0
        return CHECK_CONDITION and set SENSE:   
            key: ILLEGAL_REQUEST
            additional code: INVALID_FIELD_IN_CBD 
*/      


#[test]
fn test_inquiry() {
    let mut bytes = [0; 5];
    let mut cmd = InquiryCommand::default();
    assert_eq!(cmd, InquiryCommand::unpack(&bytes).unwrap());

    bytes[0] |= 0b00000001;
    cmd.enable_vital_product_data = true;
    assert_eq!(cmd, InquiryCommand::unpack(&bytes).unwrap());    

    bytes[1] = 0x99;
    cmd.page_code = 0x99;
    assert_eq!(cmd, InquiryCommand::unpack(&bytes).unwrap());    

    let al = 9999;
    bytes[2] = ((al >> 8) & 0xFF) as u8;
    bytes[3] = ((al >> 0) & 0xFF) as u8;
    cmd.allocation_length = al;
    assert_eq!(cmd, InquiryCommand::unpack(&bytes).unwrap());    

    bytes[4] = 0x31;
    cmd.control = 0x31;
    assert_eq!(cmd, InquiryCommand::unpack(&bytes).unwrap());    
}