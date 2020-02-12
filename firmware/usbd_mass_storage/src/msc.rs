use usb_device::{
    class_prelude::*,
    descriptor::descriptor_type,
};
use usb_device::Result;
use packed_struct::PrimitiveEnum;
use crate::{
    InterfaceSubclass,
    InterfaceProtocol,
};

use itm_logger::*;

/// This should be used as `device_class` when building `UsbDevice`
///
/// Section 4.3 [USB Bulk Only Transport Spec](https://www.usb.org/document-library/mass-storage-bulk-only-10)
pub const USB_CLASS_MSC: u8                = 0x08;


// (B) Table 3.1 Mass Storage Interface Class Control Protocol Codes
const USB_MSC_PROTOCOL_CBI: u8         = 0x00;
const USB_MSC_PROTOCOL_CBI_ALT: u8     = 0x01;
const USB_MSC_PROTOCOL_BBB: u8         = 0x50;

// (B) Table 4.1 Mass Storage Request Codes
const USB_MSC_REQ_CODES_ADSC: u8       = 0x00;
const USB_MSC_REQ_CODES_GET: u8        = 0xFC;
const USB_MSC_REQ_CODES_PUT: u8        = 0xFD;
const USB_MSC_REQ_CODES_BOMSR: u8      = 0xFF;

// (A) Table 3.1/3.2 Class-Specific Request Codes
const USB_MSC_REQ_BULK_ONLY_RESET: u8  = 0xFF;
const USB_MSC_REQ_GET_MAX_LUN: u8      = 0xFE;


const DESCRIPTOR_TYPE_DEVICE_QUALIFIER: u8 = 6;
fn is_standard_descriptor_type(value: u8) -> bool {
    match value {
        descriptor_type::DEVICE |
        descriptor_type::CONFIGURATION |
        descriptor_type::STRING |
        descriptor_type::INTERFACE |
        descriptor_type::ENDPOINT |
        descriptor_type::BOS |
        descriptor_type::CAPABILITY |
        DESCRIPTOR_TYPE_DEVICE_QUALIFIER => true,
        _ => false,
    }
}

/// # USB Mass Storage Class Device
///
/// So far only tested with the Bulk Only protocol and the SCSI transparent command set - see 
/// [Scsi](struct.Scsi.html) and [Bulk Only Transport](struct.BulkOnlyTransport.html)
pub struct MscClass<'a, B: UsbBus> {
    pub(crate) msc_if: InterfaceNumber,
    pub(crate) read_ep: EndpointOut<'a, B>,
    pub(crate) write_ep: EndpointIn<'a, B>,
    pub(crate) subclass: InterfaceSubclass,
    pub(crate) protocol: InterfaceProtocol,
}

impl<B: UsbBus> MscClass<'_, B> {
    pub fn new(
        alloc: &UsbBusAllocator<B>, 
        max_packet_size: u16, 
        subclass: InterfaceSubclass,
        protocol: InterfaceProtocol,
    ) -> MscClass<'_, B> {
        MscClass {
            msc_if: alloc.interface(),
            write_ep: alloc.bulk(max_packet_size),
            read_ep: alloc.bulk(max_packet_size),
            subclass,
            protocol,
        }
    }

    pub fn max_packet_size(&self) -> u16 {
        // The size is the same for both endpoints.
        self.read_ep.max_packet_size()
    }

    pub fn read_packet(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.read_ep.read(buf)
    }

    pub fn write_packet(&mut self, buf: &[u8]) -> Result<usize> {
        self.write_ep.write(buf)
    }
}

impl<B: UsbBus> UsbClass<B> for MscClass<'_, B> {
    fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> Result<()> {
        writer.interface(
            self.msc_if,
            USB_CLASS_MSC,
            self.subclass.to_primitive(),
            self.protocol.to_primitive(),
        )?;

        writer.endpoint(&self.read_ep)?;
        writer.endpoint(&self.write_ep)?;

        Ok(())
    }

    fn reset(&mut self) { }

    fn control_in(&mut self, xfer: ControlIn<B>) {
        let req = xfer.request();

        if req.index != u8::from(self.msc_if) as u16 {
            return;
        }
        
        let (dtype, _index) = req.descriptor_type_index();

        match (req.request_type, req.request) {
            (control::RequestType::Standard, control::Request::GET_DESCRIPTOR) => {
                if !is_standard_descriptor_type(dtype) {
                    debug!("Standard => GET_DESCRIPTOR, v: {}", dtype);
                }
            },
            (control::RequestType::Class, USB_MSC_REQ_BULK_ONLY_RESET) => {
                xfer.accept(|_| {
                    self.reset();
                    debug!("Performed to bulk_only_reset request");
                    Ok(0)
                }).ok();
            },
            (control::RequestType::Class, USB_MSC_REQ_GET_MAX_LUN) => {
                xfer.accept(|data| {
                    // We use 0 LUNs (Logical Unit Number)
                    data[0] = 0;
                    debug!("get_max_lun");
                    Ok(1)
                }).ok();
            },
            _ => { trace!("Unhandled IN: {:?}", req); },
        };
    }

    fn control_out(&mut self, xfer: ControlOut<B>) {
        let req = xfer.request();

        if req.index != u8::from(self.msc_if) as u16 {
            return;
        }

        //let (dtype, index) = req.descriptor_type_index();

        match (req.request_type, req.request) {
            (control::RequestType::Standard, control::Request::SET_ADDRESS) => {
                debug!("set address");
            },
            (control::RequestType::Standard, control::Request::SET_CONFIGURATION) => {
                debug!("set configuration");
            }
            _ => { trace!("Unhandled OUT: {:?}", req); },
        };
    }
}