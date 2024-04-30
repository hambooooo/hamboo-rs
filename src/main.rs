#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;

use esp_hal::entry;

use hambooo::esp32s3::init_heap;

slint::include_modules!();

#[entry]
fn main() -> ! {
    init_heap();

    // Configure platform for Slint
    slint::platform::set_platform(Box::new(hambooo::esp32s3::SlintPlatform::default())).expect("backend already initialized");

    create_app().run().unwrap();
    panic!("The MCU demo should not quit");
}

fn create_app() -> App {
    let app = App::new().expect("Failed to load UI");

    // let ui_handle = ui.as_weak();
    // // ui.on_request_increase_value(move || {
    // //     let ui = ui_handle.unwrap();
    // //     ui.set_counter(ui.get_counter() + 1);
    // // });
    app
}