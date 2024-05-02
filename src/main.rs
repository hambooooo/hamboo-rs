#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use core::time::Duration;

use esp_hal::entry;
use esp_println::println;
use slint::{Timer, TimerMode};
use hambooo::esp32s3;



slint::include_modules!();

#[entry]
fn main() -> ! {
    esp32s3::init_heap();

    // Configure platform for Slint
    let esp_platform = hambooo::esp32s3::EspPlatform::default();
    slint::platform::set_platform(Box::new(esp_platform)).expect("backend already initialized");

    create_app().run().unwrap();
    panic!("The MCU demo should not quit");
}

fn create_app() -> App {
    let app = App::new().expect("Failed to load UI");
    //
    // slint::run_event_loop().expect("Slint run event loop panic!");
    // let timer = Timer::default();
    // timer.start(TimerMode::Repeated, Duration::from_millis(200), move || {
    //     let datetime = esp32s3::get_datetime();
    //     // app_weak.unwrap().set_time_text(datetime.into());
    //     println!("Current time ==> {}", datetime);
    // });
    // slint::run_event_loop().expect("Slint run event loop panic!");
    app
}