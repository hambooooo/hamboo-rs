#![no_std]
#![no_main]


use cst816s::{CST816S, TouchEvent, TouchGesture};
use display_interface_spi::SPIInterface;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, Primitive, PrimitiveStyle, Rectangle, Triangle},
};
use embedded_graphics::mono_font::ascii::FONT_8X13;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::text::Text;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay,
    delay::Delay,
    gpio::{IO, rtc_io::IntoLowPowerPin},
    peripherals::Peripherals,
    prelude::*,
    spi::{master::Spi, SpiMode},
    xtensa_lx::timer::delay,
};
use esp_hal::clock::CpuClock;
use esp_hal::i2c::I2C;
use mipidsi::{Builder, Display, options::ColorInversion};
use mipidsi::models::ST7789;
use mipidsi::options::{ColorOrder, Orientation, Rotation};

const SCREEN_HEIGHT: u16 = 280;

const SCREEN_WIDTH: u16 = 240;

const SWIPE_LENGTH: u32 = 20;
const SWIPE_WIDTH: u32 = 2;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    esp_println::logger::init_logger_from_env();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let cs = io.pins.gpio16.into_push_pull_output();
    let dc = io.pins.gpio17.into_push_pull_output();
    let rst = io.pins.gpio13.into_push_pull_output();
    let clk = io.pins.gpio15.into_push_pull_output();
    let mosi = io.pins.gpio14.into_push_pull_output();
    let mut bl = io.pins.gpio18.into_push_pull_output();

    bl.set_high();

    let mut spi = Spi::new(
        peripherals.SPI3,
        40u32.MHz(),
        SpiMode::Mode3,
        &clocks,
    );
    let spi = spi.with_sck(clk).with_mosi(mosi);
    log::info!("spi init.");

    let spi_device = ExclusiveDevice::new(spi, cs, delay);
    let di = SPIInterface::new(spi_device, dc);
    let mut display = Builder::new(ST7789, di)
        .reset_pin(rst)
        // .orientation(Orientation::new().rotate(Rotation::Deg90))
        .display_size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .display_offset(0, 20)
        .color_order(ColorOrder::Rgb)
        .invert_colors(ColorInversion::Inverted)
        .init(&mut delay)
        .unwrap();
    log::info!("display init.");

    let touch_int = io.pins.gpio9.into_pull_up_input();
    let touch_rst = io.pins.gpio10.into_push_pull_output();
    let touch_sda = io.pins.gpio11;
    let touch_scl = io.pins.gpio12;


    let i2c = I2C::new(peripherals.I2C0, touch_sda, touch_scl, 100u32.kHz(), &clocks, None);
    let mut touchpad = CST816S::new(i2c, touch_int, touch_rst);
    touchpad.setup(&mut delay).unwrap();

    delay.delay(1.millis());

    // Make the display all black
    // display.clear(Rgb565::BLACK).unwrap();
    // Draw a smiley face with white eyes and a red mouth
    // draw_smiley(&mut display).unwrap();
    // draw_hello_world(&mut display).unwrap();

    loop {
        draw_hello_world(&mut display);
        delay.delay(2.secs());
        draw_smiley(&mut display);
        delay.delay(2.secs());
        // if let Some(evt) = touchpad.read_one_touch_event(false) {
        //     log::info!("{:?}",evt);
        //
        //     draw_marker(&mut display, &evt, Rgb565::RED);
        // } else {
        //     delay.delay(1.millis());
        // }
    }
}

fn draw_hello_world<T: DrawTarget<Color=Rgb565>>(display: &mut T) -> Result<(), T::Error> {
    display.clear(Rgb565::BLACK)?;
    Rectangle::new(Point::new(0, 0), Size::new(240, 280))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_DARK_GRAY))
        .draw(display)?;
    Rectangle::new(Point::new(5, 5), Size::new(230, 270))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
        .draw(display)?;

    Circle::new(Point::new(0, 0), 80)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
        .draw(display)?;

    Circle::new(Point::new(160, 0), 80)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
        .draw(display)?;
    Circle::new(Point::new(160, 200), 80)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLUE))
        .draw(display)?;

    Circle::new(Point::new(0, 200), 80)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::YELLOW))
        .draw(display)?;

    Text::new(
        "Hello World!",
        Point::new(80, 140),
        MonoTextStyle::new(&FONT_8X13, RgbColor::WHITE),
    )
        .draw(display)?;
    Ok(())
}

fn draw_smiley<T: DrawTarget<Color=Rgb565>>(display: &mut T) -> Result<(), T::Error> {
    display.clear(Rgb565::BLACK)?;
    // Draw the left eye as a circle located at (50, 100), with a diameter of 40, filled with white
    Circle::new(Point::new(50, 80), 40)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(display)?;

    // Draw the right eye as a circle located at (50, 200), with a diameter of 40, filled with white
    Circle::new(Point::new(50, 180), 40)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(display)?;

    // Draw an upside down red triangle to represent a smiling mouth
    Triangle::new(
        Point::new(130, 120),
        Point::new(130, 180),
        Point::new(160, 150),
    )
        .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
        .draw(display)?;

    // Cover the top part of the mouth with a black triangle so it looks closed instead of open
    Triangle::new(
        Point::new(130, 130),
        Point::new(130, 170),
        Point::new(150, 150),
    )
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
        .draw(display)?;
    Ok(())
}

/// Draw an indicator of the kind of gesture we detected
fn draw_marker(display: &mut impl DrawTarget<Color=Rgb565>, event: &TouchEvent, color: Rgb565) {
    let x_pos = event.x;
    let y_pos = event.y;

    match event.gesture {
        TouchGesture::SlideLeft | TouchGesture::SlideRight => {
            Rectangle::new(
                Point::new(x_pos - SWIPE_LENGTH  as i32, y_pos - SWIPE_WIDTH  as i32),
                Size::new(x_pos as u32 + SWIPE_LENGTH, y_pos  as u32 + SWIPE_WIDTH),
            )
                .into_styled(PrimitiveStyle::with_fill(color))
                .draw(display)
                .map_err(|_| ())
                .unwrap();
        }
        TouchGesture::SlideUp | TouchGesture::SlideDown => {
            Rectangle::new(
                Point::new(x_pos - SWIPE_LENGTH  as i32, y_pos - SWIPE_WIDTH  as i32),
                Size::new(x_pos as u32 + SWIPE_LENGTH, y_pos  as u32 + SWIPE_WIDTH),
            )
                .into_styled(PrimitiveStyle::with_fill(color))
                .draw(display)
                .map_err(|_| ())
                .unwrap();
        }
        TouchGesture::SingleClick => Circle::new(Point::new(x_pos, y_pos), 20)
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(display)
            .map_err(|_| ())
            .unwrap(),
        TouchGesture::LongPress => {
            Circle::new(Point::new(x_pos, y_pos), 40)
                .into_styled(PrimitiveStyle::with_stroke(color, 4))
                .draw(display)
                .map_err(|_| ())
                .unwrap();
        }
        _ => {}
    }
}