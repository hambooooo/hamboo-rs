use embedded_sdmmc::{Error, Mode, SdCard, SdCardError, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use esp_println::{print, println};

pub fn sdmmc<S, CS, D, T>(spi: S, cs: CS, delay: D, ts: T) -> Result<(), Error<SdCardError>>
    where
        S: embedded_hal::spi::SpiDevice,
        CS: embedded_hal::digital::OutputPin,
        D: embedded_hal::delay::DelayNs,
        T: TimeSource,
{
    let sdcard = SdCard::new(spi, cs, delay);
    println!("Card size is {} bytes", sdcard.num_bytes()?);
    let mut volume_mgr = VolumeManager::new(sdcard, ts);
    let mut volume0 = volume_mgr.open_volume(VolumeIdx(0))?;
    println!("Volume 0: {:?}", volume0);
    let mut root_dir = volume0.open_root_dir()?;
    // let mut my_file = root_dir.open_file_in_dir("MY_FILE.TXT", Mode::ReadOnly)?;
    // while !my_file.is_eof() {
    //     let mut buffer = [0u8; 32];
    //     let num_read = my_file.read(&mut buffer)?;
    //     for b in &buffer[0..num_read] {
    //         print!("{}", *b as char);
    //     }
    // }
    Ok(())
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