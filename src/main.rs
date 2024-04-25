#![no_std]
#![cfg_attr(not(feature = "simulator"), no_main)]

extern crate alloc;

#[cfg(not(feature = "simulator"))]
use {
    cst816s::{CST816S, TouchEvent},
    display_interface_spi::SPIInterface,
    embedded_graphics::{
        pixelcolor::Rgb565,
        prelude::*,
        primitives::{Circle, Primitive, PrimitiveStyle, Rectangle},
    },
    embedded_graphics::{
        mono_font::{ascii::FONT_10X20, MonoTextStyle},
        text::Text,
    },
    embedded_hal_bus::spi::ExclusiveDevice,
    esp_backtrace as _,
    esp_hal::{
        clock::ClockControl,
        delay::Delay,
        gpio::IO,
        i2c::I2C,
        peripherals::Peripherals,
        prelude::*,
        spi::{master::Spi, SpiMode},
    },
    mipidsi::{
        {Builder, options::ColorInversion},
        models::ST7789,
        options::ColorOrder,
    },
};

slint::include_modules!();

fn create_slint_app() -> AppWindow {
    let ui = AppWindow::new().expect("Failed to load UI");

    let ui_handle = ui.as_weak();
    ui.on_request_increase_value(move || {
        let ui = ui_handle.unwrap();
        ui.set_counter(ui.get_counter() + 1);
    });
    ui
}

#[cfg(feature = "simulator")]
fn main() -> Result<(), slint::PlatformError> {
    create_slint_app().run()
}

#[cfg(not(feature = "simulator"))]
#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

#[cfg(not(feature = "simulator"))]
fn init_heap() {
    const HEAP_SIZE: usize = 250 * 1024;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe { ALLOCATOR.init(&mut HEAP as *mut u8, core::mem::size_of_val(&HEAP)) }
    // slint::platform::set_platform(Box::new(EspBackend::default()))
    //     .expect("backend already initialized");
}

#[cfg(not(feature = "simulator"))]
#[entry]
fn main() -> ! {
    init_heap();
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

    let spi = Spi::new(
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
        .display_size(240, 280)
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

    let i2c = I2C::new(peripherals.I2C1, touch_sda, touch_scl, 400u32.kHz(), &clocks, None);
    let mut touch = CST816S::new(i2c, touch_int, touch_rst);
    touch.setup(&mut delay).unwrap();

    // let _ui = create_slint_app();

    draw_hello_world(&mut display).unwrap();

    loop {
        if let Some(evt) = touch.read_one_touch_event(true) {
            // log::info!("{:?}",evt);

            draw_marker(&mut display, &evt, Rgb565::RED);
        } else {
            delay.delay(1.millis());
        }
    }
}

#[cfg(not(feature = "simulator"))]
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
        .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_GOLD))
        .draw(display)?;
    Circle::new(Point::new(160, 200), 80)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLUE))
        .draw(display)?;

    Circle::new(Point::new(0, 200), 80)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::YELLOW))
        .draw(display)?;

    Text::new(
        "Hello World!",
        Point::new(70, 140),
        MonoTextStyle::new(&FONT_10X20, RgbColor::WHITE),
    )
        .draw(display)?;
    Ok(())
}

#[cfg(not(feature = "simulator"))]
/// Draw an indicator of the kind of gesture we detected
fn draw_marker(display: &mut impl DrawTarget<Color=Rgb565>, event: &TouchEvent, color: Rgb565) {
    let x_pos = event.x;
    let y_pos = event.y;

    Circle::new(Point::new(x_pos, y_pos), 5)
        .into_styled(PrimitiveStyle::with_fill(color))
        .draw(display)
        .map_err(|_| ())
        .unwrap();
}