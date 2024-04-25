#![no_std]
#![cfg_attr(not(feature = "simulator"), no_main)]

extern crate alloc;

#[cfg(not(feature = "simulator"))]
use {
    alloc::boxed::Box,
    alloc::rc::Rc,
    core::cell::RefCell,
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
    embedded_hal::digital::OutputPin,
    embedded_hal_bus::spi::ExclusiveDevice,
    esp_alloc::EspHeap,
    esp_backtrace as _,
    esp_hal::{
        clock::ClockControl,
        delay::Delay,
        gpio::IO,
        i2c::I2C,
        peripherals,
        peripherals::Peripherals,
        prelude::*,
        rtc_cntl::Rtc,
        spi::{master::Spi, SpiMode},
        systimer::SystemTimer,
        timer::TimerGroup,
    },
    mipidsi::{
        {Builder, options::ColorInversion},
        Display,
        models::ST7789,
        options::ColorOrder,
    },
    slint::platform::WindowEvent,
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
#[entry]
fn main() -> ! {
    // HEAP configuration
    const HEAP_SIZE: usize = 250 * 1024;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    #[global_allocator]
    static ALLOCATOR: EspHeap = EspHeap::empty();
    unsafe { ALLOCATOR.init(&mut HEAP as *mut u8, core::mem::size_of_val(&HEAP)) }

    // Configure platform for Slint
    slint::platform::set_platform(Box::new(SlintPlatform::default())).expect("backend already initialized");

    create_slint_app().run().unwrap();
    panic!("The MCU demo should not quit");
}

#[cfg(not(feature = "simulator"))]
#[derive(Default)]
struct SlintPlatform {
    window: RefCell<Option<Rc<slint::platform::software_renderer::MinimalSoftwareWindow>>>,
}

#[cfg(not(feature = "simulator"))]
impl slint::platform::Platform for SlintPlatform {
    fn create_window_adapter(
        &self,
    ) -> Result<alloc::rc::Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError>
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

        let mut buffer_provider = DrawBuffer {
            display,
            buffer: &mut [slint::platform::software_renderer::Rgb565Pixel(0); 240],
        };

        loop {
            slint::platform::update_timers_and_animations();

            if let Some(window) = self.window.borrow().clone() {
                window.draw_if_needed(|renderer| {
                    renderer.render_by_line(&mut buffer_provider);
                });
                if window.has_active_animations() {
                    continue;
                }
            }

            // let button = slint::platform::PointerEventButton::Left;
            // if let Some(event) = touch.read_one_touch_event(true).map(|record| {
            //     let position = slint::PhysicalPosition::new(record.x as _, record.y as _)
            //         .to_logical(self.window.scale_factor());
            //     esp_println::println!("{:?}", record);
            //     match record.action {
            //         0 => WindowEvent::PointerPressed { position, button },
            //         1 => WindowEvent::PointerReleased { position, button },
            //         2 => WindowEvent::PointerMoved { position },
            //         _ => WindowEvent::PointerExited,
            //     }
            // }) {
            //     esp_println::println!("{:?}", event);
            //     let is_pointer_release_event: bool = matches!(event, WindowEvent::PointerReleased { .. });
            //     self.window.dispatch_event(event);
            //
            //     // removes hover state on widgets
            //     if is_pointer_release_event {
            //         self.window.dispatch_event(WindowEvent::PointerExited);
            //     }
            // }
        }
    }
}

#[cfg(not(feature = "simulator"))]
struct DrawBuffer<'a, Display> {
    display: Display,
    buffer: &'a mut [slint::platform::software_renderer::Rgb565Pixel],
}

#[cfg(not(feature = "simulator"))]
impl<
    DI: display_interface::WriteOnlyDataCommand,
    RST: OutputPin<Error = core::convert::Infallible>,
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
        self.display
            .set_pixels(
                range.start as u16,
                line as _,
                range.end as u16,
                line as u16,
                buffer
                    .iter()
                    .map(|x| embedded_graphics::pixelcolor::raw::RawU16::new(x.0).into()),
            )
            .unwrap();
    }
}