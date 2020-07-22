#![no_std]
#![no_main]
use crate::vga_buffer::MODE;
use crate::vga_buffer::WRITER;
use crate::vga_buffer::ADVANCED_WRITER;
use vga::colors::Color16;

extern crate rlibc;

mod vga_buffer;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    loop {}
}

static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    MODE.lock().init();
    MODE.lock().graphics_init();

    //for i in 0..60 {
    //    println!("{}", i);
    //};
    let test_string = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzab";
    for i in 0..60 {
        println!("abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzab");
    }

    //println!("Hello, World!");
    //MODE.lock().graphics_init();
    //println!("Hello, World!");
    //println!("Hello, World!");
    //println!("Hello, World!");
    //println!("Hello, World!");
    //MODE.lock().text_init();
    //WRITER.lock().set_back_color(Color16::White);
    //WRITER.lock().set_front_color(Color16::Black);
    panic!("Some panic message!");
}
//2.5
//33.5

//55.5
