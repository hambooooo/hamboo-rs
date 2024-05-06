#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::ToString;
use alloc::sync::Arc;
use core::cell::{OnceCell, RefCell};
use core::mem::MaybeUninit;
use core::time::Duration;

use cst816s::CST816S;
use display_interface::WriteOnlyDataCommand;
use display_interface_spi::SPIInterface;
use embedded_graphics::prelude::OriginDimensions;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal_bus::i2c::RefCellDevice;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_backtrace as _;
use esp_hal::{Blocking, entry};
use esp_hal::clock::ClockControl;
use esp_hal::delay::Delay;
use esp_hal::gpio::{GpioPin, IO, Output, PushPull};
use esp_hal::i2c::{Error, I2C};
use esp_hal::peripherals::{I2C1, Peripherals};
use esp_hal::prelude::_fugit_RateExtU32;
use esp_hal::rtc_cntl::Rtc;
use esp_hal::spi::master::Spi;
use esp_hal::spi::SpiMode;
use esp_hal::system::SystemExt;
use esp_hal::systimer::SystemTimer;
use esp_hal::timer::TimerGroup;
use esp_hal::xtensa_lx::timer::delay;
// use esp_println::println;
use mipidsi::{Builder, Display};
use mipidsi::models::ST7789;
use mipidsi::options::{ColorInversion, ColorOrder};
use pcf8563::{DateTime, Error as RtcError, PCF8563};
use slint::{Timer, TimerMode, Weak};
use slint::platform::{Platform, WindowEvent};
use slint::platform::software_renderer::{LineBufferProvider, MinimalSoftwareWindow, RepaintBufferType, Rgb565Pixel};

slint::include_modules!();

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 128 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}


static mut I2C_BUS: OnceCell<RefCell<I2C<I2C1, Blocking>>> = OnceCell::new();
static MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
static mut TOUCH_RELEASED: bool = false;

#[entry]
fn main() -> ! {
    init_heap();
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

    // bl.set_high();

    let spi = Spi::new(
        peripherals.SPI3,
        40u32.MHz(),
        SpiMode::Mode3,
        &clocks,
    );
    let spi = spi.with_sck(clk).with_mosi(mosi);
    // log::info!("spi init.");

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

    // log::info!("display init.");

    let touch_int = io.pins.gpio9.into_pull_up_input();
    let touch_rst = io.pins.gpio10.into_push_pull_output();
    let i2c_sda = io.pins.gpio11;
    let i2c_scl = io.pins.gpio12;

    let i2c = I2C::new(peripherals.I2C1, i2c_sda, i2c_scl, 400u32.kHz(), &clocks, None);

    /// To share i2c bus, see @ https://github.com/rust-embedded/embedded-hal/issues/35
    let i2c_ref_cell = RefCell::new(i2c);

    unsafe {
        I2C_BUS.get_or_init(|| i2c_ref_cell);
    }

    let i2c_ref_cell = unsafe { I2C_BUS.get().unwrap() };

    let mut touch = CST816S::new(
        RefCellDevice::new(i2c_ref_cell),
        touch_int,
        touch_rst,
    );
    touch.setup(&mut delay).unwrap();

    let size = display.size();
    let size = slint::PhysicalSize::new(size.width, size.height);

    let mut rtc = PCF8563::new(RefCellDevice::new(i2c_ref_cell));

    let mut buffer_provider = DrawBuffer {
        display,
        buffer: &mut [Rgb565Pixel(0); 240],
    };

    let window = MinimalSoftwareWindow::new(RepaintBufferType::ReusedBuffer);
    slint::platform::set_platform(Box::new(Backend { window: window.clone() })).expect("Set platform error");
    window.set_size(size);


    let light_timer = Timer::default();
    light_timer.start(TimerMode::SingleShot, Duration::from_secs(1), move || {
        bl.set_high();
    });

    let datetime_timer = Timer::default();
    let app = App::new().unwrap();
    update_datetime(&mut rtc, app.as_weak());
    datetime_timer.start(TimerMode::Repeated, Duration::from_secs(1), move || {
        update_datetime(&mut rtc, app.as_weak());
    });

    let mut touch_delay_timer: Option<Timer> = None;
    let mut last_touch_position = None;
    let mut last_touch_button = None;
    loop {
        slint::platform::update_timers_and_animations();

        let button = slint::platform::PointerEventButton::Left;
        if let Some(event) = touch.read_one_touch_event(true).map(|record| {
            let position = slint::PhysicalPosition::new(record.x as _, record.y as _).to_logical(window.scale_factor());
            // esp_println::println!("{:?}", record);
            last_touch_position = Some(position);
            last_touch_button = Some(button);
            let touch_released = unsafe {TOUCH_RELEASED};
            if touch_released {
                return WindowEvent::PointerPressed { position, button };
            }
            match record.action {
                0 => WindowEvent::PointerPressed { position, button },
                1 => WindowEvent::PointerReleased { position, button },
                2 => WindowEvent::PointerMoved { position },
                _ => WindowEvent::PointerExited,
            }
        }) {
            esp_println::println!("{:?}", event);
            unsafe {
                TOUCH_RELEASED = false;
            };
            window.dispatch_event(event);
            if let Some(touch_timer) = touch_delay_timer {
                touch_timer.stop();
                touch_delay_timer = None;
            }
            let touch_timer = Timer::default();
            let window_copy = window.clone();
            touch_timer.start(TimerMode::SingleShot, Duration::from_millis(500), move || {
                let event = WindowEvent::PointerReleased {
                    position: last_touch_position.unwrap(),
                    button: last_touch_button.unwrap(),
                };
                unsafe {
                    TOUCH_RELEASED = true;
                };
                // esp_println::println!("{:?}", event);
                window_copy.dispatch_event(event);
            });
            touch_delay_timer = Some(touch_timer);
        }

        window.draw_if_needed(|renderer| {
            renderer.render_by_line(&mut buffer_provider);
        });
        if window.has_active_animations() {
            continue;
        }
        delay.delay_us(1u32);
    }
}

fn update_datetime(rtc: &mut PCF8563<RefCellDevice<I2C<'_, I2C1, Blocking>>>, app_weak: Weak<App>) {
    match app_weak.upgrade() {
        Some(app) => {
            match rtc.get_datetime() {
                Ok(date_time) => {
                    app.set_hours_text(format!("{:02}", date_time.hours).into());
                    app.set_minutes_text(format!("{:02}", date_time.minutes).into());
                    let date = format!("{}th {}", date_time.day, MONTHS[(date_time.month - 1) as usize]);
                    app.set_date_text(date.into());
                    app.set_datetime_show(true);
                }
                Err(_) => {}
            };
        }
        None => {}
    }
}


struct Backend {
    window: Rc<MinimalSoftwareWindow>,
}

impl Platform for Backend {
    fn create_window_adapter(&self) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        // Since on MCUs, there can be only one window, just return a clone of self.window.
        // We'll also use the same window in the event loop.
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> Duration {
        Duration::from_millis(
            SystemTimer::now() * 1_000 / SystemTimer::TICKS_PER_SECOND,
        )
    }

    // fn run_event_loop(&self) -> Result<(), slint::PlatformError>
    fn debug_log(&self, arguments: core::fmt::Arguments) {
        // log::debug!("Slint: {:?}", arguments);
    }
}

struct DrawBuffer<'a, Display> {
    display: Display,
    buffer: &'a mut [Rgb565Pixel],
}

impl<DI, RST> LineBufferProvider for &mut DrawBuffer<'_, Display<DI, ST7789, RST>>
    where
        DI: WriteOnlyDataCommand,
        RST: OutputPin<Error=core::convert::Infallible>,
{
    type TargetPixel = Rgb565Pixel;

    fn process_line(
        &mut self,
        line: usize,
        range: core::ops::Range<usize>,
        render_fn: impl FnOnce(&mut [Rgb565Pixel]),
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