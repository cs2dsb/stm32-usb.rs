use packed_struct_codegen::PrimitiveEnum;

/// The status of a command
#[derive(Clone, Copy, Eq, PartialEq, Debug, PrimitiveEnum)]
pub enum CommandStatus {
    /// Ok, command completed successfully
    CommandOk = 0x00,
    /// Error, command failed
    CommandError = 0x01,
    /// Fatal device error, reset required
    PhaseError = 0x02,
}