use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

use axp2101::{Axp2101, I2CInterface};
use cst816s::CST816S;
use display_interface::WriteOnlyDataCommand;
use display_interface_spi::SPIInterface;
use embassy_time::{Duration, Timer};
use embedded_hal::digital::OutputPin;
use embedded_hal_bus::i2c::RefCellDevice;
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use embedded_sdmmc::{SdCard, VolumeManager};
use embedded_sdmmc::sdcard::DummyCsPin;
use esp_hal::Blocking;
use esp_hal::delay::Delay;
use esp_hal::gpio::{GpioPin, Input, Output, PullUp, PushPull};
use esp_hal::i2c::I2C;
use esp_hal::mcpwm::operator::PwmPin;
use esp_hal::peripherals::{I2C1, MCPWM0, SPI2, SPI3};
use esp_hal::spi::FullDuplexMode;
use esp_hal::spi::master::Spi;
use esp_hal::systimer::SystemTimer;
use esp_println::println;
use log::log;
use mipidsi::Display;
use mipidsi::models::ST7789;
use pcf8563::{DateTime as Datetime, PCF8563};
use slint::{Image, LogicalPosition, Rgba8Pixel, SharedPixelBuffer, Weak};
use slint::platform::{PointerEventButton, WindowEvent};
use slint::platform::software_renderer::{
    LineBufferProvider,
    MinimalSoftwareWindow,
    RepaintBufferType,
    Rgb565Pixel,
};
use spin::Mutex;

use crate::storage::SdMmcClock;

slint::include_modules!();

static mut TOUCH_RELEASED: bool = true;
static mut TOUCH_RELEASED_TIMES: u32 = 0;
static mut LAST_TOUCH_POSITION: Option<LogicalPosition> = None;
static mut LAST_TOUCH_BUTTON: Option<PointerEventButton> = None;

struct EspBackend {
    window: Rc<MinimalSoftwareWindow>,
}

impl slint::platform::Platform for EspBackend {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> core::time::Duration {
        core::time::Duration::from_millis(
            SystemTimer::now() / (SystemTimer::TICKS_PER_SECOND / 1000),
        )
    }

    fn debug_log(&self, arguments: core::fmt::Arguments) {
        esp_println::println!("{}", arguments);
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

#[embassy_executor::task]
pub async fn run(
    display: Display<SPIInterface<ExclusiveDevice<Spi<'static, SPI3, FullDuplexMode>, GpioPin<Output<PushPull>, 16>, Delay>, GpioPin<Output<PushPull>, 17>>, ST7789, GpioPin<Output<PushPull>, 13>>,
    mut touch: CST816S<RefCellDevice<'static, I2C<'static, I2C1, Blocking>>, GpioPin<Input<PullUp>, 9>, GpioPin<Output<PushPull>, 10>>,
    mut axp2101: Axp2101<I2CInterface<RefCellDevice<'static, I2C<'static, I2C1, Blocking>>>>,
    rtc: PCF8563<RefCellDevice<'static, I2C<'static, I2C1, Blocking>>>,
    bl_pwm_pin: PwmPin<'static, GpioPin<Output<PushPull>, 18>, MCPWM0, 0, true>,
    volume_manager: VolumeManager<SdCard<ExclusiveDevice<Spi<'static, SPI2, FullDuplexMode>, DummyCsPin, NoDelay>, GpioPin<Output<PushPull>, 35>, Delay>, SdMmcClock>,
) {
    let mut buffer_provider = DrawBuffer {
        display,
        buffer: &mut [Rgb565Pixel(0); 240],
    };

    let size = slint::PhysicalSize::new(240, 280);
    let window = MinimalSoftwareWindow::new(RepaintBufferType::ReusedBuffer);
    window.set_size(size);
    slint::platform::set_platform(Box::new(EspBackend { window: window.clone() }))
        .expect("backend already initialized");

    let ui = UI::new().unwrap();

    // 定时更新电池状态
    let battery_timer = slint::Timer::default();
    update_battery(&mut axp2101, ui.as_weak());
    let ui_week = ui.as_weak();
    battery_timer.start(slint::TimerMode::Repeated, core::time::Duration::from_secs(1), move || {
        update_battery(&mut axp2101, ui_week.clone());
    });

    // 定时更新UI日期时间
    let rtc = Arc::new(Mutex::new(rtc));
    let rtc_cloned = rtc.clone();
    let datetime_timer = slint::Timer::default();
    update_datetime(rtc_cloned.lock().deref_mut(), ui.as_weak());
    let ui_weak = ui.as_weak();
    let rtc_cloned = rtc.clone();
    datetime_timer.start(slint::TimerMode::Repeated, core::time::Duration::from_secs(1), move || {
        update_datetime(rtc_cloned.lock().deref_mut(), ui_weak.clone());
    });

    // 修改时间
    let rtc_cloned = rtc.clone();
    ui.global::<System>().on_set_datetime(move |year, month, weekday, day, hours, minutes, seconds| {
        let new_datetime = Datetime {
            year: year as u8,
            month: month as u8,
            weekday: weekday as u8,
            day: day as u8,
            hours: hours as u8,
            minutes: minutes as u8,
            seconds: seconds as u8,
        };
        println!("{:#?}", new_datetime);
        rtc_cloned.lock().deref_mut().set_datetime(&new_datetime).expect("Set datetime error");
    });

    let volume_manager = Arc::new(Mutex::new(volume_manager));
    let volume_manager_cloned = volume_manager.clone();
    ui.global::<ImageLoader>().on_load(move |file_name| {
        log::info!("ImageLoader load ==> {:#?}", file_name);
        let image = crate::storage::sdcard_read(volume_manager_cloned.lock().deref_mut(), &file_name).unwrap();
        let image = SerializableImage::deserialize(&image).unwrap();
        log::info!("Deserialize image from bytes");
        Image::from_rgba8(image.into())
    });

    // 延迟亮屏过滤花屏
    let bl = Arc::new(Mutex::new(bl_pwm_pin));
    let bl_timer = slint::Timer::default();
    let bl_cloned = bl.clone();
    bl_timer.start(slint::TimerMode::SingleShot, core::time::Duration::from_secs(1), move || {
        bl_cloned.lock().deref_mut().set_timestamp(50);
    });

    // 控制屏幕亮度
    let bl_cloned = bl.clone();
    ui.global::<System>().on_brightness_change(move |value| {
        bl_cloned.lock().deref_mut().set_timestamp((value as f32 * 0.9 + 10.0) as u16);
    });

    // 处理触摸屏问题
    let touch_timer = slint::Timer::default();
    let window_copy = window.clone();
    touch_timer.start(slint::TimerMode::Repeated, core::time::Duration::from_millis(1), move || {
        let button = PointerEventButton::Left;
        if let Some(event) = touch.read_one_touch_event(true).map(|record| {
            let position = slint::PhysicalPosition::new(record.x as _, record.y as _).to_logical(window_copy.scale_factor());
            unsafe {
                LAST_TOUCH_POSITION = Some(position);
                LAST_TOUCH_BUTTON = Some(button);
                TOUCH_RELEASED = false;
                TOUCH_RELEASED_TIMES = 0;
            }
            match record.action {
                0 => WindowEvent::PointerPressed { position, button },
                1 => {
                    unsafe { TOUCH_RELEASED = true };
                    WindowEvent::PointerReleased { position, button }
                }
                2 => WindowEvent::PointerMoved { position },
                _ => WindowEvent::PointerExited,
            }
        }) {
            // esp_println::println!("A ==> {:?}", event);
            window_copy.dispatch_event(event);
        } else {
            if unsafe { !TOUCH_RELEASED } {
                if unsafe { TOUCH_RELEASED_TIMES > 10 } {
                    let event = WindowEvent::PointerReleased {
                        position: unsafe { LAST_TOUCH_POSITION.unwrap() },
                        button: unsafe { LAST_TOUCH_BUTTON.unwrap() },
                    };
                    // esp_println::println!("B ==> {:?}", event);
                    window_copy.dispatch_event(event);
                    unsafe { TOUCH_RELEASED = true };
                    unsafe { TOUCH_RELEASED_TIMES = 0 };
                }
                unsafe { TOUCH_RELEASED_TIMES += 1 };
            }
        }
    });

    loop {
        slint::platform::update_timers_and_animations();
        window.draw_if_needed(|renderer| {
            renderer.render_by_line(&mut buffer_provider);
        });

        if !window.has_active_animations() {
            if let Some(duration) = slint::platform::duration_until_next_timer_update() {
                Timer::after(Duration::from_millis(duration.as_millis() as u64)).await;
                continue;
            }
        }
        Timer::after(Duration::from_millis(10)).await;
    }
}

fn update_battery(power: &mut Axp2101<I2CInterface<RefCellDevice<I2C<'_, I2C1, Blocking>>>>, ui_weak: Weak<UI>) {
    match ui_weak.upgrade() {
        Some(ui) => {
            match power.is_charging() {
                Ok(charging) => {
                    ui.global::<Battery>().set_charging(charging);
                }
                Err(_) => {}
            };
            match power.get_battery_persentage() {
                Ok(battery_persent) => {
                    ui.global::<Battery>().set_percent(battery_persent.into());
                }
                Err(_) => {}
            };
        }
        None => {}
    }
}

fn update_datetime(rtc: &mut PCF8563<RefCellDevice<I2C<'_, I2C1, Blocking>>>, ui_weak: Weak<UI>) {
    match ui_weak.upgrade() {
        Some(ui) => {
            match rtc.get_datetime() {
                Ok(date_time) => {
                    // println!("Current datetime ==> {:#?}", date_time);
                    ui.global::<DateTime>().set_year(if date_time.year > 99 { 0 } else { date_time.year.into() });
                    ui.global::<DateTime>().set_month(if date_time.month < 1 || date_time.month > 12 { 1 } else { date_time.month.into() });
                    ui.global::<DateTime>().set_weekday(if date_time.weekday > 6 { 0 } else { date_time.weekday.into() });
                    ui.global::<DateTime>().set_day(if date_time.day < 1 || date_time.day > 31 { 1 } else { date_time.day.into() });
                    ui.global::<DateTime>().set_hours(if date_time.hours > 23 { 0 } else { date_time.hours.into() });
                    ui.global::<DateTime>().set_minutes(if date_time.minutes > 59 { 0 } else { date_time.minutes.into() });
                    ui.global::<DateTime>().set_seconds(if date_time.seconds > 59 { 0 } else { date_time.seconds.into() });
                }
                Err(_) => {}
            };
        }
        None => {}
    }
}

pub struct SerializableImage {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl SerializableImage {
    pub fn deserialize(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 8 {
            return Err("Data too short to contain width and height.");
        }
        let width = u32::from_be_bytes(data[0..4].try_into().unwrap());
        let height = u32::from_be_bytes(data[4..8].try_into().unwrap());
        let image_data = &data[8..];
        Ok(Self {
            width,
            height,
            data: image_data.to_vec(),
        })
    }
}

impl Into<SharedPixelBuffer<Rgba8Pixel>> for SerializableImage {
    fn into(self) -> SharedPixelBuffer<Rgba8Pixel> {
        SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&self.data, self.width, self.height)
    }
}