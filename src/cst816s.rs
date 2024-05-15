#![no_std]

use core::fmt::Debug;

use embedded_hal as hal;
use hal::delay::DelayNs;

/// Errors in this crate
#[derive(Debug)]
pub enum Error<CommE, PinE> {
    Comm(CommE),
    Pin(PinE),

    GenericError,
}

pub struct CST816S<I2C, PINT, RST> {
    i2c: I2C,
    pin_int: PINT,
    pin_rst: RST,
    blob_buf: [u8; BLOB_BUF_LEN],
}

#[derive(Debug)]
pub struct TouchEvent {
    pub x: i32,
    pub y: i32,
    /// the gesture that this touch is part of
    pub gesture: TouchGesture,
    /// 0 down, 1 lift, 2 contact
    pub action: u8,
    /// identifies the finger that touched (0-9)
    pub finger_id: u8,
    /// pressure level of touch
    pub pressure: u8,
    /// the surface area of the touch
    pub area: u8,
}

impl<I2C, PINT, RST, CommE, PinE> CST816S<I2C, PINT, RST>
    where
        I2C: hal::i2c::I2c<Error = CommE>,
        PINT: hal::digital::InputPin,
        RST: hal::digital::StatefulOutputPin<Error = PinE>,
{
    pub fn new(port: I2C, interrupt_pin: PINT, reset_pin: RST) -> Self {
        Self {
            i2c: port,
            pin_int: interrupt_pin,
            pin_rst: reset_pin,
            blob_buf: [0u8; BLOB_BUF_LEN],
        }
    }

    /// setup the driver to communicate with the device
    pub fn setup(&mut self, delay_source: &mut impl DelayNs) -> Result<(), Error<CommE, PinE>> {
        // reset the chip
        self.pin_rst.set_low().map_err(Error::Pin)?;
        delay_source.delay_us(20_000);
        self.pin_rst.set_high().map_err(Error::Pin)?;
        delay_source.delay_us(400_000);

        self.set_irq_pluse_width(200)?;
        self.set_nor_scan_per(30)?;
        Ok(())
    }

    pub fn set_irq_pluse_width(&mut self, ms: u8) -> Result<(), Error<CommE, PinE>> {
        let read_reg = [0xed, ms * 10];
        self.i2c
            .write_read(Self::DEFAULT_I2C_ADDRESS, &read_reg, self.blob_buf.as_mut())
            .map_err(Error::Comm)?;
        Ok(())
    }

    pub fn set_nor_scan_per(&mut self, t_ms: u8) -> Result<(), Error<CommE, PinE>> {
        let read_reg = [0xee, t_ms];
        self.i2c
            .write_read(Self::DEFAULT_I2C_ADDRESS, &read_reg, self.blob_buf.as_mut())
            .map_err(Error::Comm)?;
        Ok(())
    }

    /// Read enough registers to fill our read buf
    pub fn read_registers(&mut self) -> Result<(), Error<CommE, PinE>> {
        let read_reg = [Self::REG_FIRST; 1];
        self.i2c
            .write_read(Self::DEFAULT_I2C_ADDRESS, &read_reg, self.blob_buf.as_mut())
            .map_err(Error::Comm)?;
        Ok(())
    }

    pub fn read_truncated_registers(&mut self) -> Result<(), Error<CommE, PinE>> {
        let read_reg = [Self::REG_FIRST; 1];
        self.i2c
            .write_read(
                Self::DEFAULT_I2C_ADDRESS,
                &read_reg,
                self.blob_buf[0..ONE_EVENT_LEN].as_mut(),
            )
            .map_err(Error::Comm)?;
        Ok(())
    }

    ///
    /// Translate raw register data into touch events
    ///
    fn touch_event_from_data(buf: &[u8]) -> Option<TouchEvent> {
        let mut touch = TouchEvent {
            x: 0,
            y: 0,
            gesture: TouchGesture::None,
            action: 0,
            finger_id: 0,
            pressure: 0,
            area: 0,
        };

        // two of the registers mix 4 bits of position with other values
        // four high bits of X and 2 bits of Action:
        let touch_x_h_and_action = buf[Self::TOUCH_X_H_AND_ACTION_OFF];
        // four high bits of Y and 4 bits of Finger:
        let touch_y_h_and_finger = buf[Self::TOUCH_Y_H_AND_FINGER_OFF];

        // X and Y position are both 12 bits, in pixels from top left corner?
        touch.x = (buf[Self::TOUCH_X_L_OFF] as i32) | (((touch_x_h_and_action & 0x0F) as i32) << 8);
        touch.y = (buf[Self::TOUCH_Y_L_OFF] as i32) | (((touch_y_h_and_finger & 0x0F) as i32) << 8);

        // action of touch (0 = down, 1 = up, 2 = contact)
        touch.action = (touch_x_h_and_action >> 6) as u8;
        touch.finger_id = (touch_y_h_and_finger >> 4) as u8;

        //  Compute touch pressure and area
        touch.pressure = buf[Self::TOUCH_PRESURE_OFF];
        touch.area = buf[Self::TOUCH_AREA_OFF] >> 4;

        Some(touch)
    }

    /// The main method for getting the current touch event.
    /// Returns a touch event if available.
    ///
    /// - `check_int_pin` -- True if we should check the interrupt pin before attempting i2c read.
    /// On some devices, attempting to read registers when there is no data available results
    /// in a hang in the i2c read.
    ///
    pub fn read_one_touch_event(&mut self, check_int_pin: bool) -> Option<TouchEvent> {
        let mut one_event: Option<TouchEvent> = None;
        // the interrupt pin should typically be low if there is data available;
        // otherwise, attempting to read i2c will cause a stall
        let data_available = !check_int_pin || self.pin_int.is_low().unwrap_or(false);
        if data_available {
            if self.read_truncated_registers().is_ok() {
                let gesture_id = self.blob_buf[Self::GESTURE_ID_OFF];
                let num_points = (self.blob_buf[Self::NUM_POINTS_OFF] & 0x0F) as usize;
                if num_points <= Self::MAX_TOUCH_CHANNELS {
                    //In testing with a PineTime we only ever seem to get one event
                    let evt_start: usize = Self::GESTURE_HEADER_LEN;
                    if let Some(mut evt) = Self::touch_event_from_data(
                        self.blob_buf[evt_start..evt_start + Self::RAW_TOUCH_EVENT_LEN].as_ref(),
                    ) {
                        evt.gesture = gesture_id.into();
                        one_event = Some(evt);
                    }
                }
            }
        }
        one_event
    }

    const DEFAULT_I2C_ADDRESS: u8 = 0x15;

    pub const GESTURE_HEADER_LEN: usize = 3;
    /// Number of bytes for a single touch event
    pub const RAW_TOUCH_EVENT_LEN: usize = 6;

    /// In essence, max number of fingers
    pub const MAX_TOUCH_CHANNELS: usize = 10;

    /// The first register on the device
    const REG_FIRST: u8 = 0x00;

    /// Header bytes (first three of every register block read)
    // const RESERVED_0_OFF: usize = 0;
    const GESTURE_ID_OFF: usize = 1;
    const NUM_POINTS_OFF: usize = 2;

    /// These offsets are relative to the body start (after NUM_POINTS_OFF)
    /// offset of touch X position high bits and Action bits
    const TOUCH_X_H_AND_ACTION_OFF: usize = 0;
    /// offset of touch X position low bits
    const TOUCH_X_L_OFF: usize = 1;
    /// offset of touch Y position high bits and Finger bits
    const TOUCH_Y_H_AND_FINGER_OFF: usize = 2;
    /// offset of touch Y position low bits
    const TOUCH_Y_L_OFF: usize = 3;
    const TOUCH_PRESURE_OFF: usize = 4;
    const TOUCH_AREA_OFF: usize = 5;
}

const BLOB_BUF_LEN: usize = (10 * 6) + 3; // (MAX_TOUCH_CHANNELS * RAW_TOUCH_EVENT_LEN) + GESTURE_HEADER_LEN;
const ONE_EVENT_LEN: usize = 6 + 3; // RAW_TOUCH_EVENT_LEN + GESTURE_HEADER_LEN

#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum TouchGesture {
    None = 0x00,
    SlideDown = 0x01,
    SlideUp = 0x02,
    SlideLeft = 0x03,
    SlideRight = 0x04,
    SingleClick = 0x05,
    DoubleClick = 0x0B,
    LongPress = 0x0C,
}

impl core::convert::From<u8> for TouchGesture {
    fn from(val: u8) -> Self {
        match val {
            0x01 => Self::SlideDown,
            0x02 => Self::SlideUp,
            0x03 => Self::SlideLeft,
            0x04 => Self::SlideRight,
            0x05 => Self::SingleClick,
            0x0B => Self::DoubleClick,
            0x0C => Self::LongPress,
            _ => Self::None,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
