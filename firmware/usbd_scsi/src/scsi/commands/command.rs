use packing::Packed;
use packed_struct_codegen::{ PackedStruct, PrimitiveEnum };
use packed_struct::{
    PackedStruct,
};

use crate::scsi::{
    commands::*,
    Error,
    Direction,
    packing::ParsePackedStruct,
};

pub trait Respond<B, P> 
where 
    P: PackedStruct<B> 
{
    fn respond() -> P;
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CommandLength {
    C6,
    C10,
    //C12,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, PrimitiveEnum)]
pub enum CommandStatus {
    CommandOk = 0x00,
    CommandError = 0x01,
    PhaseError = 0x02,
}

/// SCSI op codes as defined by SPC-3
#[derive(Clone, Copy, Eq, PartialEq, Debug, PrimitiveEnum)]
pub enum CommandOpCode {
    TestUnitReady = 0x00,
    RequestSense = 0x03,
    Format = 0x04,
    Read6 = 0x08,
    Write6 = 0x0A,
    Inquiry = 0x12,
    ReadCapacity10 = 0x25,
    Read10 = 0x28,
    SendDiagnostic = 0x1D,
    ReportLuns = 0xA0,

    ModeSense6 = 0x1A,
    ModeSense10 = 0x5A,

    ModeSelect6 = 0x15,
    StartStopUnit = 0x1B,
    PreventAllowMediumRemoval = 0x1E,
    ReadFormatCapacities = 0x23,
    Write10 = 0x2A,
    Verify10 = 0x2F,
    SynchronizeCache10 = 0x35,
    ReadTocPmaAtip = 0x43,
    ModeSelect10 = 0x55,
    Read12 = 0xA8,
    Write12 = 0xAA,
}


/// This is the last byte on all commands
#[derive(Clone, Copy, Eq, PartialEq, Debug, Default, Packed)]
#[packed(big_endian, lsb0)]
pub struct Control {
    #[pkd(7, 6, 0, 0)]
    pub vendor_specific: u8,
    #[pkd(2, 2, 0, 0)]
    pub normal_aca: bool,
}

/// A fully parsed and validated SCSI command
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Command {
    None,
    Inquiry(InquiryCommand),
    TestUnitReady(TestUnitReadyCommand),
    ReadCapacity(ReadCapacity10Command),
    ModeSense(ModeSenseXCommand),
    PreventAllowMediumRemoval(PreventAllowMediumRemovalCommand),
    RequestSense(RequestSenseCommand),
    Read(ReadXCommand),
    Write(WriteXCommand),
    Format(FormatCommand),
    SendDiagnostic(SendDiagnosticCommand),
    ReportLuns(ReportLunsCommand),
    ModeSelect(ModeSelectXCommand),
    StartStopUnit(StartStopUnitCommand),
    ReadFormatCapacities(ReadFormatCapacitiesCommand),
    Verify(Verify10Command),
    SynchronizeCache(SynchronizeCache10Command),
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="lsb")]
/// A wrapper that identifies a command sent from the host to the
/// device on the OUT endpoint. Describes the data transfer IN or OUT
/// that should happen immediatly after this wrapper is received.
/// Little Endian
pub struct CommandBlockWrapper {
    /// Signature that identifies this packet as CBW
    /// Must contain 0x43425355
    pub signature: u32,
    /// Tag sent by the host. Must be echoed back to host in tag
    /// field of the command status wrapper sent after the command
    /// has been executed/rejected. Host uses it to positively 
    /// associate a CSW with the corresponding CBW
    pub tag: u32,
    /// Number of bytes of data that the host expects to receive on
    /// the IN or OUT endpoint (as indicated by the direction field) 
    /// during the execution of this command. If this field is zero, 
    /// must respond directly with CSW
    pub data_transfer_length: u32,
    /// Direction of transfer initiated by this command.
    /// 0b0XXXXXXX = OUT from host to device
    /// 0b1XXXXXXX = IN from device to host
    /// X bits are obsolete or reserved
    #[packed_field(element_size_bytes="1", ty="enum")]
    pub direction: Direction,
    /// The device Logical Unit Number (LUN) to which the command is
    /// for. For devices that don't support multiple LUNs the host will
    /// set this field to zero.
    /// Devices that don't support multiple LUNS must not ignore this 
    /// field and apply all commands to LUN 0, [see General Problems with Commands](http://janaxelson.com/device_errors.htm)
    pub lun: u8,
    /// The number of valid bytes in data field. Note this is decremented
    /// by 1 in `verify` because we are chopping the op code off the front
    /// of the data field into the `command` field during deserialization
    pub data_length: u8,
    /// The operation code as defined by the command set identified by the
    /// interface sub-class (SCSI transparent command set in this case)
    #[packed_field(element_size_bytes="1", ty="enum")]
    pub command: CommandOpCode,
    /// The command set specific data for this command minus the first byte
    /// which was chopped off and put in the `command` field.
    /// The SCSI transparent command set used doesn't have the same endianness
    /// as the rest of this structure - SCSI uses big endian.
    pub data: [u8; 15],
}

fn checked_extract<T>(len: u8, data: &[u8]) -> Result<T, Error>
where
    T: ParsePackedStruct,
    Error: From<<T as Packed>::Error>,
{
    if len < T::BYTES as u8 {
        Err(Error::InsufficientDataForCommand)?
    }
    Ok(T::parse(data)?)
}

impl Default for CommandBlockWrapper {
    fn default() -> Self {
        Self {
            signature: Self::SIGNATURE,
            tag: 0,
            data_transfer_length: 0,
            direction: Direction::ToHost,
            lun: 0,
            data_length: 0,
            command: CommandOpCode::TestUnitReady,
            data: [0; 15],
        }
    }
}

impl CommandBlockWrapper {
    const SIGNATURE: u32 = 0x43425355;

    pub fn extract_command(&self) -> Result<Command, Error> {
        match self.command {
            CommandOpCode::Read6 => Ok(Command::Read(checked_extract::<Read6Command>(self.data_length, &self.data)?.into())),
            CommandOpCode::Read10 => Ok(Command::Read(checked_extract::<Read10Command>(self.data_length, &self.data)?.into())),
            CommandOpCode::Read12 => Ok(Command::Read(checked_extract::<Read12Command>(self.data_length, &self.data)?.into())),
            CommandOpCode::ReadCapacity10 => Ok(Command::ReadCapacity(checked_extract(self.data_length, &self.data)?)), 
            CommandOpCode::ReadFormatCapacities => Ok(Command::ReadFormatCapacities(checked_extract(self.data_length, &self.data)?)),
            CommandOpCode::Inquiry => Ok(Command::Inquiry(checked_extract(self.data_length, &self.data)?)),
            CommandOpCode::TestUnitReady => Ok(Command::TestUnitReady(checked_extract(self.data_length, &self.data)?)),
            CommandOpCode::ModeSense6 => Ok(Command::ModeSense(checked_extract::<ModeSense6Command>(self.data_length, &self.data)?.into())),
            CommandOpCode::ModeSense10 => Ok(Command::ModeSense(checked_extract::<ModeSense10Command>(self.data_length, &self.data)?.into())),
            CommandOpCode::ModeSelect6 => Ok(Command::ModeSelect(checked_extract::<ModeSelect6Command>(self.data_length, &self.data)?.into())),
            CommandOpCode::ModeSelect10 => Ok(Command::ModeSelect(checked_extract::<ModeSelect10Command>(self.data_length, &self.data)?.into())),
            CommandOpCode::PreventAllowMediumRemoval => Ok(Command::PreventAllowMediumRemoval(checked_extract(self.data_length, &self.data)?)),
            CommandOpCode::RequestSense => Ok(Command::RequestSense(checked_extract(self.data_length, &self.data)?)),
            CommandOpCode::Write6 => Ok(Command::Write(checked_extract::<Write6Command>(self.data_length, &self.data)?.into())),
            CommandOpCode::Write10 => Ok(Command::Write(checked_extract::<Write10Command>(self.data_length, &self.data)?.into())),
            CommandOpCode::Write12 => Ok(Command::Write(checked_extract::<Write12Command>(self.data_length, &self.data)?.into())),
            CommandOpCode::Format => Ok(Command::Format(checked_extract(self.data_length, &self.data)?)),
            CommandOpCode::SendDiagnostic => Ok(Command::SendDiagnostic(checked_extract(self.data_length, &self.data)?)),
            CommandOpCode::ReportLuns => Ok(Command::ReportLuns(checked_extract(self.data_length, &self.data)?)),
            CommandOpCode::StartStopUnit => Ok(Command::StartStopUnit(checked_extract(self.data_length, &self.data)?)),
            CommandOpCode::Verify10 => Ok(Command::Verify(checked_extract(self.data_length, &self.data)?)),
            CommandOpCode::SynchronizeCache10 => Ok(Command::SynchronizeCache(checked_extract(self.data_length, &self.data)?)),
            _ => Err(Error::UnhandledOpCode),
        }
    }
}

/// A wrapper that identifies a command sent from the host to the
/// device on the OUT endpoint. Describes the data transfer IN or OUT
/// that should happen immediatly after this wrapper is received.
/// Little Endian
#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="lsb")]
pub struct CommandStatusWrapper {
    /// Signature that identifies this packet as CSW
    /// Must contain 0x53425355
    pub signature: u32,
    /// Tag that matches this CSW back to the CBW that initiated it.
    /// Must be copied from CBW tag field. Host uses it to positively 
    /// associate a CSW with the corresponding CBW
    pub tag: u32,
    /// Difference between the expected data length from CSW.data_transfer_length
    /// and the the actual amount of data sent or received. Cannot be greater
    /// than data_transfer_length. Non-zero for an OUT (host to device) transfer
    /// likely means there was an error whereas non-zero on IN (device to host) may
    /// mean the host allocated enough space for an extended/complete result but
    /// a shorter result was sent.
    pub data_residue: u32,
    /// The status of the command
    /// 0x00 = Command succeeded
    /// 0x01 = Command failed
    /// 0x02 = Phase error. Causes the host to perform a reset recovery on the 
    ///        device. This indicates the device state machine has got messed up
    ///        or similar unrecoverable condition. Processing further CBWs without
    ///        a reset gives indeterminate results.
    #[packed_field(element_size_bytes="1", ty="enum")]
    pub status: CommandStatus,
}
