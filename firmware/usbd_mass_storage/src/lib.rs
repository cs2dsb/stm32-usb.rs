//!
//! # Glossary
//!
//! | Term   | Description | More Info |
//! |--------|-------------|-----------|
//! | ZLP    | Zero length packet. Used to terminate the current data transfer when the final packet is full and the total data length is less than the header specified | Section 5.5.3 [USB 2.0 Bus Spec][USB2Bus] |
//! | CBW    | Command block wrapper. Header that contains information about the data that is expected to be sent/received next | Section 5.1 [USB Bulk Only Transport Spec][USBBot] |
//! | CSW    | Command status wrapper. Status sent after data transfer to indicate success/failure and confirm length of data sent | Section 5.2 [USB Bulk Only Transport Spec][USBBot] |
//! | Data Residue | Data residue (bytes) is the difference in the length requested in the CBW and the actual amount of data sent/received | Section 5.2 [USB Bulk Only Transport Spec][USBBot] |
//!
//! [USB2Bus]: https://www.usb.org/document-library/usb-20-specification
//! [USBBot]: https://www.usb.org/document-library/mass-storage-bulk-only-10
//!

#![no_std]
#![feature(arbitrary_enum_discriminant)]

// TODO: remove
#![allow(dead_code)]

mod msc;
mod scsi;
mod ghost_fat;
mod interface_subclass;
mod interface_protocol;
mod bulk_only_transport;

pub use usb_device::{Result, UsbError};
pub use msc::*;
pub use scsi::*;
pub use ghost_fat::*;
pub use interface_subclass::*;
pub use interface_protocol::*;

pub use bulk_only_transport::{
    BulkOnlyTransport,
    Error as BulkOnlyTransportError,
};

mod logging {
    pub use itm_logger::{
        warn,
        error,
        Level,
        log,
        debug,
        info,
    };

    #[cfg(feature = "trace-bot-headers")]
    pub use itm_logger::trace as trace_bot_headers;
    #[cfg(not(feature = "trace-bot-headers"))]
    pub use itm_logger::stub as trace_bot_headers;

    #[cfg(feature = "trace-bot-states")]
    pub use itm_logger::trace as trace_bot_states;
    #[cfg(not(feature = "trace-bot-states"))]
    pub use itm_logger::stub as trace_bot_states;

    #[cfg(feature = "trace-bot-bytes")]
    pub use itm_logger::trace as trace_bot_bytes;
    #[cfg(not(feature = "trace-bot-bytes"))]
    pub use itm_logger::stub as trace_bot_bytes;

    #[cfg(feature = "trace-bot-zlp")]
    pub use itm_logger::trace as trace_bot_zlp;
    #[cfg(not(feature = "trace-bot-zlp"))]
    pub use itm_logger::stub as trace_bot_zlp;

    #[cfg(feature = "trace-bot-buffer")]
    pub use itm_logger::trace as trace_bot_buffer;
    #[cfg(not(feature = "trace-bot-buffer"))]
    pub use itm_logger::stub as trace_bot_buffer;
    
    #[cfg(feature = "trace-scsi-command")]
    pub use itm_logger::trace as trace_scsi_command;
    #[cfg(not(feature = "trace-scsi-command"))]
    pub use itm_logger::stub as trace_scsi_command;
    
    #[cfg(feature = "trace-scsi-fs")]
    pub use itm_logger::trace as trace_scsi_fs;
    #[cfg(not(feature = "trace-scsi-fs"))]
    pub use itm_logger::stub as trace_scsi_fs;

}
