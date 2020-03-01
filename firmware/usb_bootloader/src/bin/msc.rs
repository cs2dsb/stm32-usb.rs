//! CDC-ACM serial port example using cortex-m-rtfm.
#![no_main]
#![no_std]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]

use core::{
    panic::PanicInfo,
    sync::atomic::{self, Ordering},
    str::from_utf8_unchecked,
};
use cortex_m::{
    interrupt,
    asm::*,
};    
use embedded_hal::digital::v2::OutputPin;
use rtfm::app;
use stm32f1xx_hal::{
    prelude::*,
    time::Hertz,
};
use stm32f1xx_hal::usb::{Peripheral, UsbBus, UsbBusType};
use usb_device::{
    bus,
    device::{ 
        UsbDevice, 
        UsbDeviceBuilder, 
        UsbVidPid,
    },
};
//use usb_device::prelude::*;
use usbd_serial::{CdcAcmClass, SerialPort, USB_CLASS_CDC};
use usbd_mass_storage::USB_CLASS_MSC;
use usbd_scsi::{
    Scsi,
    GhostFat,
    BlockDevice,
};
use itm_logger::*;

// VID and PID are from dapboot bluepill bootloader
const USB_VID: u16 = 0x1209; 
const USB_PID: u16 = 0xDB42;
const USB_CLASS_MISCELLANEOUS: u8 =  0xEF;

macro_rules! define_ptr_type {
    ($name: ident, $ptr: expr) => (
        impl $name {
            fn ptr() -> *const Self {
                $ptr as *const _
            }

            /// Returns a wrapped reference to the value in flash memory
            pub fn get() -> &'static Self {
                unsafe { &*Self::ptr() }
            }
        }
    )
}

#[derive(Hash, Debug)]
#[repr(C)]
pub struct Uid {
    a: u32,
    b: u32,
    c: u32,
}
define_ptr_type!(Uid, 0x1FFF_F7E8);

fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe {
    core::slice::from_raw_parts(
            (p as *const T) as *const u8,
            core::mem::size_of::<T>(),
        )
    }
}

impl Uid {
    pub fn update_serial(&self, serial: &mut [u8; SERIAL_LEN]) {
        const CHARS: &str = "0123456789ABCDEF";
        let chars = CHARS.as_bytes();
        let bytes = any_as_u8_slice(self);
        
        for (i, b) in bytes.iter().enumerate() {
            let c1 = chars[((b >> 4) & 0xF_u8) as usize];
            let c2 = chars[((b >> 0) & 0xF_u8) as usize];

            let i = i * 2;
            if i < SERIAL_LEN {
                serial[i] = c1;
            }
            if i + 1 < SERIAL_LEN {
                serial[i+1] = c2;
            }
        }
    }
}

/// Size of integrated flash
#[derive(Debug)]
#[repr(C)]
pub struct FlashSize(u16);
define_ptr_type!(FlashSize, 0x1FFF_F7E0);

impl FlashSize {
    /// Read flash size in kibi bytes
    pub fn kibi_bytes(&self) -> u16 {
        self.0
    }

    /// Read flash size in bytes
    pub fn bytes(&self) -> usize {
        usize::from(self.kibi_bytes()) * 1024
    }
}

const ITM_BAUD_RATE: u32 = 8_000_000;

const SERIAL_LEN: usize = 24;
static mut SERIAL_BYTES: [u8; SERIAL_LEN] = [0; SERIAL_LEN];

#[cfg(feature = "itm")] 
use cortex_m::{iprintln, peripheral::ITM};

#[app(device = stm32f1xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        usb_dev: UsbDevice<'static, UsbBusType>,
        serial: SerialPort<'static, UsbBusType>,
        scsi: Scsi<'static, UsbBusType, GhostFat>,
        #[init([0; 256])]
        buf: [u8; 256],
        #[init(0)]
        buf_i: usize,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        static mut USB_BUS: Option<bus::UsbBusAllocator<UsbBusType>> = None;

        #[cfg(feature = "itm")]
        {        
            update_tpiu_baudrate(8_000_000, ITM_BAUD_RATE).expect("Failed to reset TPIU baudrate");
            logger_init();
        }

        info!("Hello");

        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();

        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(48.mhz())
            .pclk1(24.mhz())
            .freeze(&mut flash.acr);

        #[cfg(feature = "itm")]
        {
            let sysclk: Hertz = clocks.sysclk().into();
            update_tpiu_baudrate(sysclk.0, ITM_BAUD_RATE).expect("Failed to reset TPIU baudrate");
        }

        assert!(clocks.usbclk_valid());

        let flash_kib = FlashSize::get().kibi_bytes();
        info!("Flash: {} KiB", flash_kib);

        let mut gpioa = cx.device.GPIOA.split(&mut rcc.apb2);

        // BluePill board has a pull-up resistor on the D+ line.
        // Pull the D+ pin down to send a RESET condition to the USB bus.
        // This forced reset is needed only for development, without it host
        // will not reset your device when you upload new firmware.
        let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
        usb_dp.set_low().unwrap();
        delay(clocks.sysclk().0 / 100);

        let usb_dm = gpioa.pa11;
        let usb_dp = usb_dp.into_floating_input(&mut gpioa.crh);

        let usb = Peripheral {
            usb: cx.device.USB,
            pin_dm: usb_dm,
            pin_dp: usb_dp,
        };

        *USB_BUS = Some(UsbBus::new(usb));

        let ghost_fat = GhostFat::new();
        let serial = SerialPort::new(USB_BUS.as_ref().unwrap());
        let scsi = Scsi::new(USB_BUS.as_ref().unwrap(), 64, ghost_fat);
        
        // Fetch the serial info from the device electronic signature registers 
        // and convert it to a utf string
        let serial_number = unsafe {
            Uid::get().update_serial(&mut SERIAL_BYTES);
            from_utf8_unchecked(&SERIAL_BYTES)
        };
        info!("Serial number: {}", serial_number);

        let usb_dev = UsbDeviceBuilder::new(USB_BUS.as_ref().unwrap(), UsbVidPid(USB_VID, USB_PID))
            .manufacturer("Fake company")
            .product("Serial port")
            .serial_number(serial_number)
            .self_powered(true)
            .device_class(0) //USB_CLASS_MSC)
            .build();

        init::LateResources { usb_dev, scsi, serial }
    }

    #[task(binds = USB_HP_CAN_TX, resources = [usb_dev, scsi, buf, buf_i, serial])]
    fn usb_tx(mut cx: usb_tx::Context) {
        usb_poll(&mut cx.resources.usb_dev, &mut cx.resources.serial, &mut cx.resources.scsi, cx.resources.buf, &mut cx.resources.buf_i);
    }

    #[task(binds = USB_LP_CAN_RX0, resources = [usb_dev, scsi, buf, buf_i, serial])]
    fn usb_rx0(mut cx: usb_rx0::Context) {
        usb_poll(&mut cx.resources.usb_dev, &mut cx.resources.serial, &mut cx.resources.scsi, cx.resources.buf, &mut cx.resources.buf_i);
    }
};

fn usb_poll<B: bus::UsbBus>(
    usb_dev: &mut UsbDevice<'static, B>,
    serial: &mut SerialPort<'static, B>,
    scsi: &mut Scsi<'static, B, GhostFat>,
    _buf: &mut [u8],
    _buf_i: &mut usize,
) {
    if !usb_dev.poll(&mut [serial, scsi]) {
        return;
    }

    /*

    loop {
        if let Ok(bytes) = msc.read_packet(&mut buf[*buf_i..]) {
            *buf_i += bytes;
        } else {
            break;
        }
    }

    trace!("i: {}", *buf_i);
    if *buf_i >= 31 {

        #[repr(C, packed)]
        #[derive(Debug)]
        struct cbw {
            dCBWSignature: u32,
            dCBWTag: u32,
            dCBWDataTransferLength: u32, 
            bmCBWFlags: u8,
            bCBWLUN: u8,
            bCBWCBLength: u8,
            CBWCB: [u8; 16],
        }
        let mut x = [0; 31];
        x.copy_from_slice(&buf[0..31]);
        let x: cbw = unsafe { core::mem::transmute(x) };
        trace!("CBW {:?}", x);
        *buf_i = 0;
    }
    */
/*
    match serial.read(&mut buf) {
        Ok(count) if count > 0 => {
            // Echo back in upper case
            for c in buf[0..count].iter_mut() {
                if 0x61 <= *c && *c <= 0x7a {
                    *c &= !0x20;
                }
            }

            serial.write(&buf[0..count]).ok();
        }
        _ => {}
    }
    */
}


#[panic_handler]
fn panic(
    #[cfg_attr(not(feature = "itm"), allow(unused_variables))]
    info: &PanicInfo
) -> ! {
    interrupt::disable();

    #[cfg(feature = "itm")]
    {
        let itm = unsafe { &mut *ITM::ptr() };
        let stim = &mut itm.stim[0];

        iprintln!(stim, "{}", info);
    }

    loop {
        // add some side effect to prevent this from turning into a UDF instruction
        // see rust-lang/rust#28728 for details
        atomic::compiler_fence(Ordering::SeqCst)
    }
}