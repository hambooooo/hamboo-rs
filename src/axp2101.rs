use esp_println::println;

// Based on https://github.com/tuupola/axp192
// and https://github.com/m5stack/M5CoreS3/blob/main/src/AXP2101.cpp
const AXP2101_ADDRESS: u8 = 0x34;
/* Power control registers */
const AXP2101_POWER_STATUS: u8 = 0x00;
const AXP2101_CHARGE_STATUS: u8 = 0x01;
const AXP2101_OTG_VBUS_STATUS: u8 = 0x04;
const AXP2101_DATA_BUFFER0: u8 = 0x06;
const AXP2101_DATA_BUFFER1: u8 = 0x07;
const AXP2101_DATA_BUFFER2: u8 = 0x08;
const AXP2101_DATA_BUFFER3: u8 = 0x09;
const AXP2101_DATA_BUFFER4: u8 = 0x0a;
const AXP2101_DATA_BUFFER5: u8 = 0x0b;
/* Output control: 2 EXTEN, 0 DCDC2 */
const AXP2101_EXTEN_DCDC2_CONTROL: u8 = 0x10;
/* Power output control: 6 EXTEN, 4 DCDC2, 3 LDO3, 2 LDO2, 1 DCDC3, 0 DCDC1 */
const AXP2101_DCDC13_LDO23_CONTROL: u8 = 0x12;
const AXP2101_DCDC2_VOLTAGE: u8 = 0x23;
const AXP2101_DCDC2_SLOPE: u8 = 0x25;
const AXP2101_DCDC1_VOLTAGE: u8 = 0x26;
const AXP2101_DCDC3_VOLTAGE: u8 = 0x27;
/* Output voltage control: 7-4 LDO2, 3-0 LDO3 */
const AXP2101_LDO23_VOLTAGE: u8 = 0x28;
const AXP2101_VBUS_IPSOUT_CHANNEL: u8 = 0x30;
const AXP2101_SHUTDOWN_VOLTAGE: u8 = 0x31;
const AXP2101_SHUTDOWN_BATTERY_CHGLED_CONTROL: u8 = 0x32;
const AXP2101_CHARGE_CONTROL_1: u8 = 0x33;
const AXP2101_CHARGE_CONTROL_2: u8 = 0x34;
const AXP2101_BATTERY_CHARGE_CONTROL: u8 = 0x35;
const AXP2101_PEK: u8 = 0x36;
const AXP2101_DCDC_FREQUENCY: u8 = 0x37;
const AXP2101_BATTERY_CHARGE_LOW_TEMP: u8 = 0x38;
const AXP2101_BATTERY_CHARGE_HIGH_TEMP: u8 = 0x39;
const AXP2101_APS_LOW_POWER1: u8 = 0x3A;
const AXP2101_APS_LOW_POWER2: u8 = 0x3B;
const AXP2101_BATTERY_DISCHARGE_LOW_TEMP: u8 = 0x3c;
const AXP2101_BATTERY_DISCHARGE_HIGH_TEMP: u8 = 0x3d;
const AXP2101_DCDC_MODE: u8 = 0x80;
const AXP2101_ADC_ENABLE_1: u8 = 0x82;
const AXP2101_ADC_ENABLE_2: u8 = 0x83;
const AXP2101_ADC_RATE_TS_PIN: u8 = 0x84;
const AXP2101_GPIO30_INPUT_RANGE: u8 = 0x85;
const AXP2101_GPIO0_ADC_IRQ_RISING: u8 = 0x86;
const AXP2101_GPIO0_ADC_IRQ_FALLING: u8 = 0x87;
const AXP2101_TIMER_CONTROL: u8 = 0x8a;
const AXP2101_VBUS_MONITOR: u8 = 0x8b;
const AXP2101_TEMP_SHUTDOWN_CONTROL: u8 = 0x8f;

/* GPIO control registers */

const AXP2101_GPIO0_LDOIO0_VOLTAGE: u8 = 0x91;
const AXP2101_GPIO1_CONTROL: u8 = 0x92;
const AXP2101_GPIO2_CONTROL: u8 = 0x93;
const AXP2101_GPIO20_SIGNAL_STATUS: u8 = 0x94;
const AXP2101_GPIO43_FUNCTION_CONTROL: u8 = 0x95;
const AXP2101_GPIO43_SIGNAL_STATUS: u8 = 0x96;
const AXP2101_GPIO20_PULLDOWN_CONTROL: u8 = 0x97;
const AXP2101_PWM1_FREQUENCY: u8 = 0x98;
const AXP2101_PWM1_DUTY_CYCLE_1: u8 = 0x99;
const AXP2101_PWM1_DUTY_CYCLE_2: u8 = 0x9a;
const AXP2101_PWM2_FREQUENCY: u8 = 0x9b;
const AXP2101_PWM2_DUTY_CYCLE_1: u8 = 0x9c;
const AXP2101_PWM2_DUTY_CYCLE_2: u8 = 0x9d;
const AXP2101_N_RSTO_GPIO5_CONTROL: u8 = 0x9e;

/* Interrupt control registers */
const AXP2101_ENABLE_CONTROL_1: u8 = 0x40;
const AXP2101_ENABLE_CONTROL_2: u8 = 0x41;
const AXP2101_ENABLE_CONTROL_3: u8 = 0x42;
const AXP2101_ENABLE_CONTROL_4: u8 = 0x43;
const AXP2101_ENABLE_CONTROL_5: u8 = 0x4a;
const AXP2101_IRQ_STATUS_1: u8 = 0x44;
const AXP2101_IRQ_STATUS_2: u8 = 0x45;
const AXP2101_IRQ_STATUS_3: u8 = 0x46;
const AXP2101_IRQ_STATUS_4: u8 = 0x47;
const AXP2101_IRQ_STATUS_5: u8 = 0x4d;

/* ADC data registers */
const AXP2101_ACIN_VOLTAGE: u8 = 0x56;
const AXP2101_ACIN_CURRENT: u8 = 0x58;
const AXP2101_VBUS_VOLTAGE: u8 = 0x5a;
const AXP2101_VBUS_CURRENT: u8 = 0x5c;
const AXP2101_TEMP: u8 = 0x5e;
const AXP2101_TS_INPUT: u8 = 0x62;
const AXP2101_GPIO0_VOLTAGE: u8 = 0x64;
const AXP2101_GPIO1_VOLTAGE: u8 = 0x66;
const AXP2101_GPIO2_VOLTAGE: u8 = 0x68;
const AXP2101_GPIO3_VOLTAGE: u8 = 0x6a;
const AXP2101_BATTERY_POWER: u8 = 0x70;
const AXP2101_BATTERY_VOLTAGE: u8 = 0x78;
const AXP2101_CHARGE_CURRENT: u8 = 0x7a;
const AXP2101_DISCHARGE_CURRENT: u8 = 0x7c;
const AXP2101_APS_VOLTAGE: u8 = 0x7e;
const AXP2101_CHARGE_COULOMB: u8 = 0xb0;
const AXP2101_DISCHARGE_COULOMB: u8 = 0xb4;
const AXP2101_COULOMB_COUNTER_CONTROL: u8 = 0xb8;

/* Computed ADC */
const AXP2101_COULOMB_COUNTER: u8 = 0xff;


// AXP2101 datasheet: https://www.x-powers.com/en.php/Info/download/id/34.html
const AXP2101_CHG_LED: u8 = 0x69;
const AXP2101_ALDO_ENABLE: u8 = 0x90;
const AXP2101_ALDO4: u8 = 0x95;
const AXP2101_DCDC1_3V3: u8 = 0x12;

//12, 25-28, 92-93
pub enum Command {
    ChgLed(bool),
    AldoEnable(bool),
    Aldo4(bool),
    // ExtenDcdc2Control(bool),
    // Dcdc13Ldo23Control(bool),
    // Dcdc2Slope(bool),
    // Dcdc1Voltage(bool),
    // Dcdc3Voltage(bool),
    // Ldo23Voltage(bool),
    // Gpio1Control(bool),
    // Gpio2Control(bool),
    Dcdc1Voltage(u8),
}

pub enum DataFormat<'a> {
    /// Slice of unsigned bytes
    U8(&'a [u8]),
}

impl Command {
    // Send command to AXP2101
    pub fn send<I>(self, iface: &mut I) -> Result<(), Axp2101Error>
        where
            I: Axp2101ReadWrite,
    {
        let (data, len) = match self {
            // Command structure: address, command, data, count & 0xf1
            //Command::Dcdc3Voltage(on) => ([AXP2101_ADDRESS, AXP2101_LDO2 , 0x0], 3),
            Command::ChgLed(_on) => ([AXP2101_CHG_LED, 0b00110101], 2),
            Command::AldoEnable(_on) => ([AXP2101_ALDO_ENABLE, 0xBF], 2),
            Command::Aldo4(_on) => ([AXP2101_ALDO4, 0b00011100], 2),
            // Command::Dcdc2Slope(_on) => ([AXP2101_DCDC2_SLOPE, 0x0], 2),
            // Command::Dcdc1Voltage(_on) => ([AXP2101_DCDC1_VOLTAGE, 106], 2),
            // Command::Dcdc3Voltage(_on) => ([AXP2101_DCDC3_VOLTAGE, 104], 2),
            // Command::Ldo23Voltage(_on) => ([AXP2101_LDO23_VOLTAGE, 242], 2),
            // Command::Gpio1Control(_on) => ([AXP2101_GPIO1_CONTROL, 0x0], 2),
            // Command::Gpio2Control(_on) => ([AXP2101_GPIO2_CONTROL, 104], 2),
            Command::Dcdc1Voltage(voltage) => ([AXP2101_ADC_ENABLE_1, voltage], 2),
        };
        iface.send_commands(DataFormat::U8(&data[0..len]))
    }
}

#[derive(Debug)]
pub enum Axp2101Error {
    NotSupported,
    InvalidArgument,
    ReadError,
    WriteError,
}

pub trait Axp2101ReadWrite {
    fn send_commands(&mut self, cmd: DataFormat<'_>) -> Result<(), Axp2101Error>;
    // fn read(&self, addr: u8, reg: u8, buffer: &mut [u8]) -> Result<(), Axp2101Error>;
    // fn write(&self, addr: u8, reg: u8, buffer: &[u8]) -> Result<(), Axp2101Error>;
}

pub struct Axp2101<I> {
    interface: I,
}

// Implement Axp2101ReadWrite for I2CInterface
impl<I> Axp2101ReadWrite for I2CInterface<I>
    where
        I: embedded_hal::i2c::I2c,
{
    // Send commands over I2C to AXP2101
    fn send_commands(&mut self, cmd: DataFormat<'_>) -> Result<(), Axp2101Error> {
        let mut data_buf = [0];

        match cmd {
            DataFormat::U8(data) => {
                self.i2c
                    .write_read(self.addr, &[data[0]], &mut data_buf)
                    .map_err(|_| Axp2101Error::WriteError)?;
                println!("read value for command {:?}: {:?}", data[0], data_buf[0]);

                println!("write value for command {:?}: {:?}", data[0], data[1]);
                self.i2c
                    .write(self.addr, data)
                    .map_err(|_| Axp2101Error::WriteError)
            }
        }
    }

    // fn read(&self, addr: u8, reg: u8, buffer: &mut [u8]) -> Result<(), Axp2101Error> {
    //     // Implement read logic here
    //     unimplemented!()
    // }

    // fn write(&self, addr: u8, reg: u8, buffer: &[u8]) -> Result<(), Axp2101Error> {
    //     // Implement write logic here
    //     unimplemented!()
    // }
}

impl<I> Axp2101<I>
    where
        I: Axp2101ReadWrite,
{
    // Create a new AXP2101 interface
    pub fn new(interface: I) -> Self {
        Self { interface }
    }

    // Initialize AXP2101
    pub fn init(&mut self) -> Result<(), Axp2101Error> {
        // Command::Ldo23Voltage(true).send(&mut self.interface)?;
        Command::ChgLed(true).send(&mut self.interface)?;
        Command::AldoEnable(true).send(&mut self.interface)?;
        Command::Aldo4(true).send(&mut self.interface)?;
        Command::Dcdc1Voltage(AXP2101_DCDC1_3V3).send(&mut self.interface)?;

        // Command::Dcdc2Slope(true).send(&mut self.interface)?;
        // Command::Dcdc1Voltage(true).send(&mut self.interface)?;
        // Command::Dcdc3Voltage(true).send(&mut self.interface)?;
        // Command::Ldo23Voltage(true).send(&mut self.interface)?;
        // Command::Gpio1Control(true).send(&mut self.interface)?;
        // Command::Gpio2Control(true).send(&mut self.interface)?;
        // Command::ExtenDcdc2Control(true).send(&mut self.interface)?;

        Ok(())
    }
}

pub struct I2CInterface<I2C> {
    i2c: I2C,
    addr: u8,
    data_byte: u8,
}

impl<I2C> I2CInterface<I2C>
    where
        I2C: embedded_hal::i2c::I2c,
{
    /// Create new I2C interface for communication with a display driver
    pub fn new(i2c: I2C, addr: u8, data_byte: u8) -> Self {
        Self {
            i2c,
            addr,
            data_byte,
        }
    }

    /// Consume the display interface and return
    /// the underlying peripherial driver
    pub fn release(self) -> I2C {
        self.i2c
    }
}

#[derive(Debug, Copy, Clone)]
pub struct I2CPowerManagementInterface(());

impl I2CPowerManagementInterface {
    pub fn new<I>(i2c: I) -> I2CInterface<I>
        where
            I: embedded_hal::i2c::I2c,
    {
        Self::new_custom_address(i2c, AXP2101_ADDRESS)
    }

    /// Create a new I2C interface with a custom address.
    pub fn new_custom_address<I>(i2c: I, address: u8) -> I2CInterface<I>
        where
            I: embedded_hal::i2c::I2c,
    {
        I2CInterface::new(i2c, address, 0x34)
    }
}
