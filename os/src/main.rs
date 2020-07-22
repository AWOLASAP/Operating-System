#![no_std]
#![no_main]
use crate::vga_buffer::MODE;

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
    //for i in 0..60 {
    //    println!("{}", i);
    //};
    //for i in 0..60 {
    //    println!("abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzab");
    //}
    MODE.lock().init();
    println!("Hello, Workd!");
    MODE.lock().graphics_init();
    println!("Hello, Workd!");
    println!("Hello, Workd!");
    println!("Hello, Workd!");
    println!("Hello, Workd!");
    MODE.lock().text_init();

    panic!("Some panic message!");
}
//2.5
//33.5

//55.5
