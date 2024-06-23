#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

extern crate alloc;

use alloc::{format, vec};
use alloc::vec::Vec;
use core::mem::MaybeUninit;

use embassy_executor::Spawner;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{SdCard, VolumeManager};
use esp_backtrace as _;
use esp_hal as hal;
use esp_println::println;
use hal::clock::ClockControl;
use hal::embassy;
use hal::gpio::IO;
use hal::gpio::NO_PIN;
use hal::peripherals::Peripherals;
use hal::prelude::*;
use hal::spi::master::Spi;
use hal::spi::SpiMode;

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

#[main]
async fn main(_spawner: Spawner) {
    esp_println::logger::init_logger(log::LevelFilter::Debug);

    const HEAP_SIZE: usize = 256 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
        log::info!("heap initialized");
    }

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();
    let delay = hal::delay::Delay::new(&clocks);

    embassy::init(
        &clocks,
        hal::timer::TimerGroup::new_async(peripherals.TIMG0, &clocks),
    );
    log::info!("embassy::init embassy-time-timg0");

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let sd_spi_sck = io.pins.gpio36;
    let sd_spi_mosi = io.pins.gpio37;
    let sd_spi_miso = io.pins.gpio39;
    let sd_spi_cs = io.pins.gpio35.into_push_pull_output();

    let sd_spi = Spi::new(
        peripherals.SPI2,
        24u32.MHz(),
        SpiMode::Mode0,
        &clocks,
    ).with_pins(Some(sd_spi_sck), Some(sd_spi_mosi), Some(sd_spi_miso), NO_PIN);

    // About spi device but not spi bus @see https://github.com/rust-embedded-community/embedded-sdmmc-rs/issues/126
    let sd_spi_device = ExclusiveDevice::new_no_delay(sd_spi, embedded_sdmmc::sdcard::DummyCsPin);

    let sdcard = SdCard::new(sd_spi_device, sd_spi_cs, delay);
    // println!("Card size is {} bytes", sdcard.num_bytes()?);
    let mut volume_manager = VolumeManager::new(sdcard, hamboo::storage::SdMmcClock);


    let byte_slices: Vec<&[u8]> = vec![
        include_bytes!("../ui/images/0001-face-pointer-hour.pxs"),
        include_bytes!("../ui/images/0002-face-pointer-minute.pxs"),
        include_bytes!("../ui/images/0003-face-pointer-second.pxs"),
        include_bytes!("../ui/images/0004-app-calculate.pxs"),
    ];
    let mut index = 0;
    for image_bytes in byte_slices {
        index += 1;
        let file_name = format!("{:04}.pxs", index);
        hamboo::storage::sdcard_write(&mut volume_manager, file_name.clone().as_str(), image_bytes).expect("Write file to sdcard error");
        println!("successfully write image {}", file_name);
    }
}


pub struct SerializableImage {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl SerializableImage {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let width_bytes = self.width.to_be_bytes();
        let height_bytes = self.height.to_be_bytes();

        let mut serialized_data = Vec::new();
        serialized_data.extend_from_slice(&width_bytes);
        serialized_data.extend_from_slice(&height_bytes);
        serialized_data.extend_from_slice(self.data.as_slice());

        serialized_data
    }
}
