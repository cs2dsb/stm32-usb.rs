#![no_std]

mod scsi;
mod ghost_fat;

pub use scsi::*;
pub use ghost_fat::*;

mod logging {
    pub use itm_logger::*;

    #[cfg(feature = "trace-scsi-command")]
    pub use itm_logger::trace as trace_scsi_command;
    #[cfg(not(feature = "trace-scsi-command"))]
    pub use itm_logger::stub as trace_scsi_command;
    
    #[cfg(feature = "trace-scsi-fs")]
    pub use itm_logger::trace as trace_scsi_fs;
    #[cfg(not(feature = "trace-scsi-fs"))]
    pub use itm_logger::stub as trace_scsi_fs;
}
