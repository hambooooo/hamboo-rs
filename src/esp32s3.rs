extern crate alloc;

use alloc::boxed::Box;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use core::any;
use core::cell::{OnceCell, RefCell};

use cst816s::CST816S;
use display_interface::WriteOnlyDataCommand;
use display_interface_spi::SPIInterface;
use embedded_graphics::prelude::*;
use embedded_hal::digital::OutputPin;
use embedded_hal::i2c::I2c;
use embedded_hal_bus::i2c::RefCellDevice;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_alloc::EspHeap;
use esp_backtrace as _;
use esp_hal::{
    Blocking, clock::ClockControl, delay::Delay, gpio::IO, i2c::I2C, peripherals::Peripherals,
    prelude::*, rtc_cntl::Rtc, spi::{master::Spi, SpiMode}, systimer::SystemTimer, timer::TimerGroup,
};
use esp_hal::clock::Clocks;
use esp_hal::gpio::{GpioPin, Input, Output, PullUp, PushPull};
use esp_hal::peripheral::Peripheral;
use esp_hal::peripherals::{I2C1, SPI3};
use esp_hal::spi::FullDuplexMode;
use esp_println::println;
use log::log;
use mipidsi::{
    {Builder, options::ColorInversion},
    Display,
    models::ST7789,
    options::ColorOrder,
};
use pcf8563::{DateTime, PCF8563};
use slint::Model;
use slint::platform::WindowEvent;

slint::include_modules!();

pub fn init_heap() {
    // HEAP configuration
    const HEAP_SIZE: usize = 250 * 1024;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    #[global_allocator]
    static ALLOCATOR: EspHeap = EspHeap::empty();
    unsafe { ALLOCATOR.init(&mut HEAP as *mut u8, core::mem::size_of_val(&HEAP)) }
}


#[derive(Default)]
pub struct EspPlatform {
    pub window: RefCell<Option<Rc<slint::platform::software_renderer::MinimalSoftwareWindow>>>,
}

impl slint::platform::Platform for EspPlatform {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError>
    {
        let window = slint::platform::software_renderer::MinimalSoftwareWindow::new(
            slint::platform::software_renderer::RepaintBufferType::ReusedBuffer,
        );
        self.window.replace(Some(window.clone()));
        Ok(window)
    }

    fn duration_since_start(&self) -> core::time::Duration {
        core::time::Duration::from_millis(
            SystemTimer::now() / (SystemTimer::TICKS_PER_SECOND / 1000),
        )
    }

    fn debug_log(&self, arguments: core::fmt::Arguments) {
        esp_println::println!("{}", arguments);
    }

    fn run_event_loop(&self) -> Result<(), slint::PlatformError> {
        let peripherals = Peripherals::take();
        let system = peripherals.SYSTEM.split();
        let clocks = ClockControl::max(system.clock_control).freeze();
        let mut delay = Delay::new(&clocks);
        esp_println::logger::init_logger_from_env();

        let mut rtc = Rtc::new(peripherals.LPWR, None);

        // Create timer groups
        let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks, None);
        // Get watchdog timers of timer groups
        let mut wdt0 = timer_group0.wdt;
        let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks, None);
        let mut wdt1 = timer_group1.wdt;

        // Disable watchdog timers
        rtc.swd.disable();
        rtc.rwdt.disable();
        wdt0.disable();
        wdt1.disable();

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
        let display = Builder::new(ST7789, di)
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

        /// To share i2c bus
        /// https://github.com/rust-embedded/embedded-hal/issues/35
        let i2c_ref_cell = RefCell::new(i2c);

        let mut touch = CST816S::new(
            RefCellDevice::new(&i2c_ref_cell),
            touch_int,
            touch_rst,
        );
        touch.setup(&mut delay).unwrap();

        let size = display.size();
        let size = slint::PhysicalSize::new(size.width, size.height);

        let mut rtc = PCF8563::new(RefCellDevice::new(&i2c_ref_cell));
        rtc.rtc_init().unwrap();

        let now = DateTime {
            year: 24, // 2022
            month: 5, // January
            weekday: 5, // Saturday
            day: 1,
            hours: 22,
            minutes: 14,
            seconds: 00,
        };

        // set date and time in one go
        rtc.set_datetime(&now).unwrap();

        let date_time = rtc.get_datetime().unwrap();
        println!("{}", format!("{} {}", date_time.hours, date_time.minutes));

        self.window.borrow().as_ref().unwrap().set_size(size);

        let mut buffer_provider = DrawBuffer {
            display,
            buffer: &mut [slint::platform::software_renderer::Rgb565Pixel(0); 240],
        };

        loop {
            slint::platform::update_timers_and_animations();

            if let Some(window) = self.window.borrow().clone() {
                let button = slint::platform::PointerEventButton::Left;
                if let Some(event) = touch.read_one_touch_event(true).map(|record| {
                    let position = slint::PhysicalPosition::new(record.x as _, record.y as _)
                        .to_logical(window.scale_factor());
                    // esp_println::println!("{:?}", record);
                    match record.action {
                        0 => WindowEvent::PointerPressed { position, button },
                        1 => WindowEvent::PointerReleased { position, button },
                        2 => WindowEvent::PointerMoved { position },
                        _ => WindowEvent::PointerExited,
                    }
                }) {
                    // esp_println::println!("{:?}", event);
                    let is_pointer_release_event: bool = matches!(event, WindowEvent::PointerReleased { .. });
                    window.dispatch_event(event);

                    // removes hover state on widgets
                    if is_pointer_release_event {
                        window.dispatch_event(WindowEvent::PointerExited);
                    }
                }

                window.draw_if_needed(|renderer| {
                    renderer.render_by_line(&mut buffer_provider);
                });
                if window.has_active_animations() {
                    continue;
                }
            }
        }
    }
}

struct DrawBuffer<'a, Display> {
    display: Display,
    buffer: &'a mut [slint::platform::software_renderer::Rgb565Pixel],
}

impl<
    DI: WriteOnlyDataCommand,
    RST: OutputPin<Error=core::convert::Infallible>,
> slint::platform::software_renderer::LineBufferProvider
for &mut DrawBuffer<'_, Display<DI, ST7789, RST>>
{
    type TargetPixel = slint::platform::software_renderer::Rgb565Pixel;

    fn process_line(
        &mut self,
        line: usize,
        range: core::ops::Range<usize>,
        render_fn: impl FnOnce(&mut [slint::platform::software_renderer::Rgb565Pixel]),
    ) {
        let buffer = &mut self.buffer[range.clone()];

        render_fn(buffer);

        // We send empty data just to get the device in the right window
        self.display.set_pixels(
            range.start as u16,
            line as _,
            range.end as u16,
            line as u16,
            buffer.iter().map(|x| embedded_graphics::pixelcolor::raw::RawU16::new(x.0).into()),
        ).unwrap();
    }
}