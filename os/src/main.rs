#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]
// Use these for things like buffer access
use os::vga_buffer::MODE;
use os::vga_buffer::WRITER;
use os::vga_buffer::ADVANCED_WRITER;
use vga::colors::Color16;
use os::print;
use os::println;

extern crate rlibc;

mod serial;

use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    MODE.lock().init();
    // Use this to activate graphics mode - graphics mode implements all of the APIs that text mode implements, 
    // but it is  slower than text mode because it doesn't operate off of direct memory access. 
    // Activating graphics mode also enables graphics things like line drawing
    MODE.lock().graphics_init();
    println!("Hello World!");

    os::init();

    #[cfg(test)]
    test_main();

    for i in 0..60 {
        println!("abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzab");
    }
    ADVANCED_WRITER.lock().clear_buffer();
    print!("This is a test");
    println!("It did not crash!");
    x86_64::instructions::interrupts::int3();

    os::hlt_loop();


    //for i in 0..60 {
    //    println!("{}", i);
    //};
    // This is an example on how to reactivate text mode and deactivate graphics mode.
    // This then changes the background and foreground color.
    //MODE.lock().text_init();
    //WRITER.lock().set_back_color(Color16::White);
    //WRITER.lock().set_front_color(Color16::Black);
}
//2.5
//33.5

//55.5
