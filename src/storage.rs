use alloc::vec::Vec;

use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use embedded_sdmmc::{Error, Mode, SdCard, SdCardError, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use embedded_sdmmc::sdcard::DummyCsPin;
use esp_hal::delay::Delay;
use esp_hal::gpio::{GpioPin, Output, PushPull};
use esp_hal::peripherals::SPI2;
use esp_hal::spi::FullDuplexMode;
use esp_hal::spi::master::Spi;
use esp_println::println;

pub fn sdcard_write(
    volume_mgr: &mut VolumeManager<SdCard<ExclusiveDevice<Spi<'static, SPI2, FullDuplexMode>, DummyCsPin, NoDelay>, GpioPin<Output<PushPull>, 35>, Delay>, SdMmcClock>,
    file_name: &str,
    data: Vec<u8>,
) -> Result<(), Error<SdCardError>>
{
    let mut volume0 = volume_mgr.open_volume(VolumeIdx(0))?;
    println!("Volume 0: {:?}", volume0);
    let mut root_dir = volume0.open_root_dir()?;
    let mut f = root_dir.open_file_in_dir(file_name, Mode::ReadWriteCreateOrTruncate)?;
    f.write(data.as_slice())?;
    Ok(())
}

pub fn sdcard_read(
    volume_mgr: &mut VolumeManager<SdCard<ExclusiveDevice<Spi<'static, SPI2, FullDuplexMode>, DummyCsPin, NoDelay>, GpioPin<Output<PushPull>, 35>, Delay>, SdMmcClock>,
    file_name: &str,
) -> Result<Vec<u8>, Error<SdCardError>>
{
    let mut volume0 = volume_mgr.open_volume(VolumeIdx(0))?;
    println!("Volume 0: {:?}", volume0);
    let mut root_dir = volume0.open_root_dir()?;
    let mut file = root_dir.open_file_in_dir(file_name, Mode::ReadOnly)?;

    let mut buffer: Vec<u8> = Vec::new();
    let mut read_buffer = [0u8; 32];

    while !file.is_eof() {
        let bytes_read = file.read(&mut read_buffer)?;
        buffer.extend_from_slice(&read_buffer[..bytes_read]);
    }
    Ok(buffer)
}

pub struct SdMmcClock;

impl TimeSource for SdMmcClock {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}