use packing::{
    Packed,
    PackedSize,
};
use usb_device::class_prelude::*;
use usb_device::Result as UsbResult;

use usbd_bulk_only_transport::{
    BulkOnlyTransport,
    Error as BulkOnlyTransportError,
    TransferState,
};

use usbd_mass_storage::InterfaceSubclass;

use crate::{
    logging::*,
    block_device::BlockDevice,
    scsi::{
        Error,
        Command,
        InquiryResponse,
        inquiry_response,
        ModeParameterHeader6,
        CachingModePage,
        RequestSenseResponse,
        ModeSenseXCommand,
        PageControl,
        CommandLength,
        PageCode,
    },
};

/// # Scsi Transparent Command Set implementation
///
/// Built on top of [BulkOnlyTransport](struct.BulkOnlyTransport.html)
///
/// [Glossary](index.html#glossary)
pub struct Scsi<'a, B: UsbBus, BD: BlockDevice> {
    inner: BulkOnlyTransport<'a, B>,
    current_command: Command,
    inquiry_response: InquiryResponse,
    request_sense_response: RequestSenseResponse,
    block_device: BD,
    lba: u32,
    lba_end: u32,
}

impl<B: UsbBus, BD: BlockDevice> Scsi<'_, B, BD> {
    pub fn new(alloc: &UsbBusAllocator<B>, max_packet_size: u16, block_device: BD) -> Scsi<'_, B, BD> {
        //TODO: This is reasonable for FAT but not FAT32 or others. BOT buffer should probably be 
        //configurable from here, perhaps passing in BD::BLOCK_BYTES.max(BOT::MIN_BUFFER) or something 
        assert!(BD::BLOCK_BYTES <= BulkOnlyTransport::<B>::BUFFER_BYTES);
        Scsi {
            inner: BulkOnlyTransport::new(
                alloc, 
                max_packet_size, 
                InterfaceSubclass::ScsiTransparentCommandSet,
                0,
            ),
            current_command: Command::None,
            inquiry_response: inquiry_response(),
            request_sense_response: Default::default(),
            block_device,
            lba: 0,
            lba_end: 0,
        }
    }

    fn get_new_command(&mut self) -> bool {
        if self.current_command != Command::None {
            return false;
        }

        if let Some(cbw) = self.inner.get_current_command() {
            // TODO: handle failure better
            match Command::extract_from_cbw(cbw) {
                Ok(c) => {
                    self.current_command = c;
                    true
                },
                Err(e) => {
                    error!("Failed to extract scsi command with op code: 0x{:X?}: {:?}",
                        cbw.data[0],
                        e,
                    );
                    false
                },
            }
        } else {
            false
        }
    }

    fn process_command(&mut self) -> Result<(), Error> {
        let transfer_state = self.inner.transfer_state();
        // These calls all assume only a single block will fit in the buffer which 
        // is true here because we configure BOT that way but we could make the inner
        // buffer length a multiple of BLOCK_SIZE and queue up more than one block
        // at a time. I don't know if there's any benefit to that but the option is there
        let skip = match transfer_state {
            TransferState::ReceivingDataFromHost { full, done, .. } => {
                !(full || done)
            },
            TransferState::SendingDataToHost { empty, .. } => {
                !empty
            },
            // We still need to check if the buffer is empty because if a CSW is being sent
            // we won't be able to grab a full block buffer if the next command happens to be
            // a Read
            TransferState::NotTransferring { empty, .. } => {
                !empty
            }
        };

        if skip {
            Err(UsbError::WouldBlock)?;
        }

        let new_command = self.get_new_command();

        let mut err = false;
        let mut done = true;

        trace_scsi_command!("COMMAND> {:?}", self.current_command);
        match self.current_command {
            Command::None => { done = false },
            Command::Inquiry(_) => {
                let buf = self.inner.take_buffer_space(InquiryResponse::BYTES)?;
                self.inquiry_response.pack(buf)?;
            },
            Command::TestUnitReady(_) => { info!("TestUnitReady"); }
            Command::PreventAllowMediumRemoval(_) => { info!("PreventAllowMediumRemoval"); },
            Command::ReadCapacity(_)  => {
                let max_lba = self.block_device.max_lba();
                let block_size = BD::BLOCK_BYTES;

                let cap_len = 8;
                
                let buf = self.inner.take_buffer_space(cap_len)?;
                buf[0] = ((max_lba >> 24) & 0xFF) as u8;
                buf[1] = ((max_lba >> 16) & 0xFF) as u8;
                buf[2] = ((max_lba >> 8) & 0xFF) as u8;
                buf[3] = ((max_lba >> 0) & 0xFF) as u8;
                buf[4] = ((block_size >> 24) & 0xFF) as u8;
                buf[5] = ((block_size >> 16) & 0xFF) as u8;
                buf[6] = ((block_size >> 8) & 0xFF) as u8;
                buf[7] = ((block_size >> 0) & 0xFF) as u8;
            },
            Command::ModeSense(ModeSenseXCommand { command_length: CommandLength::C6, page_control: PageControl::CurrentValues })  => {
                
                let mut header = ModeParameterHeader6::default();
                header.increase_length_for_page(PageCode::CachingModePage);
                
                let cache_page = CachingModePage::default();

                let buf = self.inner.take_buffer_space(
                    ModeParameterHeader6::BYTES + CachingModePage::BYTES
                )?;   

                header.pack(&mut buf[..ModeParameterHeader6::BYTES])?;
                cache_page.pack(&mut buf[ModeParameterHeader6::BYTES..])?;

                //header.device_specific_parameter.write_protect = true;
                /*
                let cache_page = CachingModePage::default().pack();
                header.increase_length_for_page(&cache_page);
                let header = header.pack();

                let buf = self.inner.take_buffer_space(
                    ModeParameterHeader6::BYTES + CachingModePage::BYTES
                )?;
                buf[..ModeParameterHeader6::BYTES].copy_from_slice(&header);
                buf[ModeParameterHeader6::BYTES..].copy_from_slice(&cache_page);
                */
            },
            Command::RequestSense(_) => {
                let buf = self.inner.take_buffer_space(RequestSenseResponse::BYTES)?;
                self.request_sense_response.pack(buf)?;
            },
            Command::Read(r) => {
                if new_command {
                    self.lba = r.lba;
                    self.lba_end = r.lba + r.transfer_length - 1;
                }

                trace_scsi_fs!("FS> Read; new: {}, lba: 0x{:X?}, lba_end: 0x{:X?}, done: {}",
                    new_command, self.lba, self.lba_end, self.lba == self.lba_end);

                // We only get here if the buffer is empty 
                let buf = self.inner.take_buffer_space(BD::BLOCK_BYTES).expect("Buffer should have been empty");
                self.block_device.read_block(self.lba, buf)?;
                self.lba += 1;

                if self.lba <= self.lba_end {
                    done = false;
                }
            },
            Command::Write(w) => {
                if new_command {
                    self.lba = w.lba;
                    self.lba_end = w.lba + w.transfer_length - 1;
                }

                trace_scsi_fs!("FS> Write; new: {}, lba: 0x{:X?}, lba_end: 0x{:X?}, done: {}",
                    new_command, self.lba, self.lba_end, self.lba == self.lba_end);

                let len = match transfer_state {
                    TransferState::ReceivingDataFromHost { done: true, full: false, bytes_available: b } => b,
                    _ => BD::BLOCK_BYTES,
                };

                let buf = self.inner.take_buffered_data(len, false).expect("Buffer should have enough data");
                self.block_device.write_block(self.lba, buf)?;
                self.lba += 1;

                if self.lba <= self.lba_end {
                    done = false;
                }
            },
            _ => {
                err = true;
            },
        };

        if done || err {
            if err {
                self.inner.send_command_error()?;
            } else {
                self.inner.send_command_ok()?;
            }
            self.current_command = Command::None;
        }

        Ok(())
    }

    fn update(&mut self) -> Result<(), Error> {

        // Send anything that's already queued
        accept_would_block(
            self.inner.write()
                .map_err(|e| e.into())
        )?;

        // Read new data if available
        accept_would_block(
            self.inner.read()
                .map_err(|e| e.into())
        )?;

        // Progress the current command or attempt to accept a new command
        accept_would_block(self.process_command())?;

        // Send anything we may have generated this go around
        accept_would_block(
            self.inner.write()
                .map_err(|e| e.into())
        )?;

        Ok(())
    }
}

fn accept_would_block(r: Result<(), Error>) -> Result<(), Error> {
    match r {
        Ok(_) | Err(Error::BulkOnlyTransportError(BulkOnlyTransportError::UsbError(UsbError::WouldBlock))) => Ok(()),
        e => e
    }
}

impl<B: UsbBus, BD: BlockDevice> UsbClass<B> for Scsi<'_, B, BD> {
    fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> UsbResult<()> {
        self.inner.get_configuration_descriptors(writer)
    }

    fn reset(&mut self) { 
        self.inner.reset()
    }

    fn control_in(&mut self, xfer: ControlIn<B>) {
        self.inner.control_in(xfer)
    }

    fn control_out(&mut self, xfer: ControlOut<B>) {
        self.inner.control_out(xfer)
    }

    fn poll(&mut self) { 
        if let Err(e) = self.update() {
            error!("Error from Scsi::update: {:?}", e);
        }
    }
}