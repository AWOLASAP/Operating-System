#![no_std]
#![no_main]
use crate::vga_buffer::ADVANCED_WRITER;
use crate::vga_buffer::WRITER;

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
    for i in 0..60 {
        println!("abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzab");
    }
    ADVANCED_WRITER.lock().draw_buffer(WRITER.lock().buffer);


    panic!("Some panic message!");

    loop{}
}
//2.5
//33.5

//55.5
