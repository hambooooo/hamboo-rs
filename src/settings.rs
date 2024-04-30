// board v1
pub mod board {
    // use esp_hal::gpio::{GpioPin, IO, Unknown};
    // use esp_hal::peripherals::Peripherals;
    //
    // static EPS32S3_GPIO: IO = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    // static PIN_SPI_CS_NUM: GpioPin<Unknown, 0> = EPS32S3_GPIO.pins.gpio0;
    // I2C引脚编号
    static PIN_I2C_SCL_NUM: u8 = 15;
    static PIN_I2C_SDA_NUM: u8 = 15;
    static PIN_I2C_INT_NUM: u8 = 15;

    // 扬声器PWM引脚编号
    static PIN_SPEAK_PWM_NUM: u8 = 15;
    // 麦克风ADC引脚编号
    static PIN_MIC_ADC_NUM: u8 = 15;
    // 震动器PWM引脚编号
    static PIN_MOTO_PWM_NUM: u8 = 15;
    // 陀螺仪INT引脚编号
    static PIN_GYRO_INT1_NUM: u8 = 15;
    static PIN_GYRO_INT2_NUM: u8 = 15;
    // SD_MMC引脚编号
    static PIN_SDMMC_DAT0_NUM: u8 = 15;
    static PIN_SDMMC_DAT1_NUM: u8 = 15;
    static PIN_SDMMC_DAT2_NUM: u8 = 15;
    static PIN_SDMMC_DAT3_NUM: u8 = 15;
    static PIN_SDMMC_SCL_NUM: u8 = 15;
    static PIN_SDMMC_CMD_NUM: u8 = 15;
}