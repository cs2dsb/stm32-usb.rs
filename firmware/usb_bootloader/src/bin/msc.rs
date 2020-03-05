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
    ptr::{
        read_volatile,
        write_volatile,
    },
    convert::TryFrom,
    mem,
    ops::RangeInclusive,
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
use stm32f1xx_hal::{
    usb::{
        Peripheral, 
        UsbBus, 
        UsbBusType,
    },
    pac::FLASH,
};
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
    Flash,
    FlashError,
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

pub struct FlashWrapper {
    page_size: u32,
    min_address: u32,
    max_address: u32,
    page_buffer: [u8; 2048],
    current_page: Option<u32>,
}

impl Flash for FlashWrapper {
    // Return the page size in bytes
    fn page_size(&self) -> u32 {
        self.page_size
    }

    fn address_range(&self) -> RangeInclusive<u32> {
        self.min_address..=self.max_address
    }

    fn current_page(&self) -> &Option<u32> {
        &self.current_page
    }

    fn page_buffer(&mut self) -> &mut [u8] {
        &mut self.page_buffer[..(self.page_size as usize)]
    }

    // Unlock the flash for erasing/writing
    fn unlock_flash(&mut self) -> Result<(), FlashError> {
        const KEY1: u32 = 0x45670123;
        const KEY2: u32 = 0xCDEF89AB;

        let flash = unsafe {
            &(*FLASH::ptr())
        };

        if flash.cr.read().lock().bit_is_set() {
            // Unlock flash
            flash.keyr.write(|w| unsafe { w.key().bits(KEY1) });
            flash.keyr.write(|w| unsafe { w.key().bits(KEY2) });
        }

        if flash.cr.read().lock().bit_is_set() {
            error!("Flash still locked after performing unlock sequence");
            Err(FlashError::HardwareError)?;
        }

        Ok(())
    }

    // Lock the flash to prevent erasing/writing
    fn lock_flash(&mut self) -> Result<(), FlashError> {
        let flash = unsafe {
            &(*FLASH::ptr())
        };

        flash.cr.modify(|_, w| w.lock().set_bit());

        Ok(())
    }

    // Is the flash busy?
    fn is_operation_pending(&self) -> bool {
        let flash = unsafe {
            &(*FLASH::ptr())
        };
        flash.sr.read().bsy().bit_is_set()
    }

    // Check if the page is empty
    fn is_page_erased(&mut self, page_address: u32) -> bool {
        for word in (page_address..(page_address+self.page_size())).step_by(4) {
            let value = unsafe { read_volatile(word as *const u32) };
            if value != 0xFFFFFFFF {
                return false;
            }
        }
        true
    }

    // Erase the page at the given address. Don't check if erase is necessary, that's done at a higher level
    fn erase_page(&mut self, page_address: u32) -> Result<(), FlashError> {
        let flash = unsafe {
            &(*FLASH::ptr())
        };

        // Make sure the flash is unlocked
        self.unlock_flash()?;

        // Indicate we want to do a page erase
        flash.cr.modify(|_, w| w.per().set_bit());

        // Set the address we want to erase
        flash.ar.write(|w| unsafe { w.far().bits(page_address) });

        // Kick off the operation
        flash.cr.modify(|_, w| w.strt().set_bit());

        // Wait until it's done
        self.busy_wait();

        // Clear page erase flag
        flash.cr.modify(|_, w| w.per().clear_bit());

        // Check the erase worked
        if !self.is_page_erased(page_address) {
            error!("Page erase failed");
            Err(FlashError::EraseError)?;
        }
            
        info!("erased 0x{:X?}", page_address);  
        

        Ok(())
    }

    fn read_bytes(&self, address: u32, bytes: &mut [u8]) -> Result<(), FlashError> {
        let range = self.address_range();
        for (i, b) in bytes.iter_mut().enumerate() {
            let hw_addr = address + i as u32;
            if !range.contains(&hw_addr) {
                Err(FlashError::InvalidAddress)?;
            }
            *b = unsafe { read_volatile(hw_addr as *const u8) };
        }

        Ok(())
    }

    fn read_page(&mut self, page_address: u32) -> Result<(), FlashError> {
        if page_address != self.page_address(page_address) {
            Err(FlashError::InvalidAddress)?;
        }
        let range = self.address_range();
        let buffer = self.page_buffer();
        for (i, half_word) in buffer.chunks_exact_mut(2).enumerate() {
            let hw_addr = page_address + i as u32 * 2;
            if !range.contains(&hw_addr) {
                Err(FlashError::InvalidAddress)?;
            }
            let value = unsafe { read_volatile(hw_addr as *const [u8; 2]) };
            half_word.copy_from_slice(&value);
        }

        self.current_page = Some(page_address);

        Ok(())
    }

    fn write_page(&mut self) -> Result<(), FlashError> {
        let flash = unsafe {
            &(*FLASH::ptr())
        };

        let page_address = self.current_page.ok_or(FlashError::InvalidAddress)?;

        // Make sure the flash is unlocked
        self.unlock_flash()?;

        let range = self.address_range();
        let buffer = self.page_buffer();

        let mut half_word = [0; 2];
        for (i, c) in buffer.chunks_exact(2).enumerate() {
            let hw_addr = page_address + i as u32 * 2;
            
            if !range.contains(&hw_addr) {
                Err(FlashError::InvalidAddress)?;
            }

            half_word.copy_from_slice(c);

            //let value = unsafe { mem::transmute(half_word) };

            let old_value = unsafe { read_volatile(hw_addr as *const [u8; 2]) };
            if old_value != half_word {
                info!("0x{:X?}: 0x{:X?} => 0x{:X?}", hw_addr, old_value, half_word); 

                // Indicate we want to write to flash
                flash.cr.modify(|_, w| w.pg().set_bit());

                // Write the half word
                unsafe { write_volatile(hw_addr as *mut [u8; 2], half_word); }

                // Wait for write to complete
                while flash.sr.read().bsy().bit_is_set() {}

                // Clear write flag
                flash.cr.modify(|_, w| w.pg().clear_bit());  

                let new_value = unsafe { read_volatile(hw_addr as *const [u8; 2]) };

                if new_value != half_word {
                    error!("write to 0x{:X?} failed", hw_addr);  
                    Err(FlashError::WriteError)?;
                }

                info!("write to 0x{:X?} ok", hw_addr);  

            }
        }

        Ok(())
    }

    fn flush_page(&mut self) -> Result<(), FlashError> {
        let page_address = self.current_page.ok_or(FlashError::InvalidAddress)?;

        let mut erase_needed = false;
        let mut write_needed = false;

        for (i, b) in self.page_buffer().iter().enumerate() {
            let hw_addr = page_address + i as u32;
            let new_value = *b;
            let old_value = unsafe { read_volatile(hw_addr as *const u8) };

            if old_value == new_value {
                // Value already on flash, no write or erase needed
                continue;
            }

            if old_value & new_value == new_value {
                // New value can be written over old value without erase
                write_needed = true;
                trace!("Write needed: 0x{:X}, 0x{:X} => 0x{:X}", hw_addr, old_value, new_value);
                continue;
            } 

            trace!("Erase page: 0x{:X}, 0x{:X} => 0x{:X}", hw_addr, old_value, new_value);
            // Erase is required
            erase_needed = true;
            break;
        }

        if erase_needed {
            info!("Flush: erase needed (page: 0x{:X})", page_address);
            self.erase_page(page_address)?;
        }

        if erase_needed || write_needed {
            info!("Flush: write needed (page: 0x{:X})", page_address);
            self.write_page()?;

            /*
            for c in self.page_buffer().chunks(16) {
                trace!("0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}", 
                    c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7],
                    c[8], c[9], c[10], c[11], c[12], c[13], c[14], c[15],
                );
            } 
            */
        }

        Ok(())
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
        scsi: Scsi<'static, UsbBusType, GhostFat<FlashWrapper>>,
        #[init([0; 256])]
        buf: [u8; 256],
        #[init(0)]
        buf_i: usize,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        static mut USB_BUS: Option<bus::UsbBusAllocator<UsbBusType>> = None;

        // If caches are enabled, write operations to flash cause the core to hang because it
        // is very likely to attempt to load into the prefetch buffer while the write is happening
        // This can be proved by counting busy loops on the SR.BSY flag. With caches enabled this will
        // almost always get < 2 cycles. With caches disabled it's a much more relistic figure of
        // 350 cycles for a write and 150k cycles for a page erase.
        // However, since we're just busy looping while writing it doesn't really matter. Might be 
        // worth disabling them if there was any useful work to be done in this time but for now,
        // leave them enabled.
        //cx.core.SCB.disable_icache();
        //cx.core.SCB.disable_dcache(&mut cx.core.CPUID);

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

        // This may not be 100% accurate. Cube hal has some random IFDEFs that don't even appear
        // to align with the core density.
        let page_size = if flash_kib > 128 {
            2048 
        } else {
            1024
        };

        let flash_wrapper = FlashWrapper { 
            page_size,
            page_buffer: [0; 2048],
            current_page: None,
            min_address: 0x08010000,
            max_address: 0x08000000 + flash_kib as u32 * 1024,
        };

        /*
        unsafe {
            const TEST_ADDR: u32 = 0x08010000;

            let mut bytes = [0; 4096];
            for (i, b) in bytes.iter_mut().enumerate() {
                *b = (i % 255) as u8;
            }

            flash_wrapper.write_bytes(TEST_ADDR, &bytes).unwrap();

            panic!("Whoops");
            //flash_wrapper.flush_page().unwrap();

            /*
            for c in bytes.chunks(16) {
                trace!("0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}", 
                    c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7],
                    c[8], c[9], c[10], c[11], c[12], c[13], c[14], c[15],
                );
            }
            loop {};
            */


            /*
            info!("Page == Address");
            flash_wrapper.write_bytes(TEST_ADDR, &bytes).unwrap();
            for c in flash_wrapper.page_buffer().chunks(16).take(4) {
                trace!("0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}, 0x{:02X?}", 
                    c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7],
                    c[8], c[9], c[10], c[11], c[12], c[13], c[14], c[15],
                );
            }


            flash_wrapper.read_page(TEST_ADDR).unwrap();
            for c in flash_wrapper.page_buffer().chunks(16) {
                trace!("0x{:X?}, 0x{:X?}, 0x{:X?}, 0x{:X?}, 0x{:X?}, 0x{:X?}, 0x{:X?}, 0x{:X?}, 0x{:X?}, 0x{:X?}, 0x{:X?}, 0x{:X?}, 0x{:X?}, 0x{:X?}, 0x{:X?}, 0x{:X?}", 
                    c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7],
                    c[8], c[9], c[10], c[11], c[12], c[13], c[14], c[15],
                );
            }

            flash_wrapper.page_buffer()[18] = 0x00;
            flash_wrapper.read_page(TEST_ADDR).unwrap();
            flash_wrapper.write_page().unwrap();

            flash_wrapper.write_bytes(TEST_ADDR, &[
                0x12,
                0x34,
                0x56,
                0x78,
                0x9A,
                //0xBC,
                //0xDE,
                //0xF0,
            ]).unwrap();
            flash_wrapper.erase_page(TEST_ADDR).unwrap();

            */
            //flash_wrapper.unlock_flash().unwrap();
            //flash_wrapper.erase_page(TEST_ADDR).unwrap();

            //let data = "hellostm32".as_bytes();
            //flash_wrapper.write_pages(TEST_ADDR, data).unwrap();
            //flash_wrapper.erase_range(TEST_ADDR, page_size * 10).unwrap();

            info!("Done");
            loop {};

            const FLASH_TEST: *mut u32 = TEST_ADDR as *mut u32;
            let value = core::ptr::read_volatile(FLASH_TEST);
            info!("Flash value before: 0x{:X?}", value);

            let flash = &(*FLASH::ptr());
            let locked = flash.cr.read().lock().bit_is_set();
            info!("Flash lock: {}", locked);

            const KEY1: u32 = 0x45670123;
            const KEY2: u32 = 0xCDEF89AB;

            // Unlock flash
            flash.keyr.write(|w| w.bits(KEY1));
            flash.keyr.write(|w| w.bits(KEY2));

            // Should be instant but just in case
            while flash.cr.read().lock().bit_is_set() {
                error!("Flash still locked");
            }
            info!("Flash unlocked");

            // Waits until flash isn't busy 
            let busy_wait = || while flash.sr.read().bsy().bit_is_set() {
                info!("Flash busy");
            };

            busy_wait();

            if value != 0xFFFFFFFF {
                let number_of_pages = 1;

                for i in 0..number_of_pages {
                    //TODO: need to make sure start address is on a page boundary
                    let page_addr = TEST_ADDR + i * page_size;

                    info!("Erasing 0x{:X?}", page_addr);

                    // Indicate we want to do a page erase
                    flash.cr.modify(|_, w| w.per().set_bit());

                    // Set the address we want to erase
                    flash.ar.write(|w| w.far().bits(page_addr));

                    // Kick off the operation
                    flash.cr.modify(|_, w| w.strt().set_bit());

                    busy_wait();
                }
                // Clear page erase flag
                flash.cr.modify(|_, w| w.per().clear_bit());
            }

            let data: [u16; 2] = [0xABCD, 0x5678];
            // Indicate we want to write to flash
            flash.cr.modify(|_, w| w.pg().set_bit());

            for (i, hw) in data.iter().enumerate() {
                let address = TEST_ADDR + i as u32 * 2;
                info!("Writing 0x{:X?} to 0x{:X?}", hw, address);

                let addr_ptr: *mut u16 = address as *mut u16;
                core::ptr::write_volatile(addr_ptr, *hw);

                // Wait for bsy to be cleared
                busy_wait();
            }
            // Clear write flag
            flash.cr.modify(|_, w| w.pg().clear_bit());

            let value = core::ptr::read_volatile(FLASH_TEST);
            info!("Flash value after: 0x{:X?}", value);

            loop {}
        }
        */

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

        let ghost_fat = GhostFat::new(flash_wrapper);
        let serial = SerialPort::new(USB_BUS.as_ref().unwrap());
        let scsi = Scsi::new(
            USB_BUS.as_ref().unwrap(), 
            64,
            ghost_fat,
            "Fake Co.",
            "Fake product",
            "FK01",
        );
        
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
    scsi: &mut Scsi<'static, B, GhostFat<FlashWrapper>>,
    _buf: &mut [u8],
    _buf_i: &mut usize,
) {
    if !usb_dev.poll(&mut [serial, scsi]) {
        return;
    }
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