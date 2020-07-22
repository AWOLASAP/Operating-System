#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]
use crate::vga_buffer::MODE;
use crate::vga_buffer::WRITER;
use crate::vga_buffer::ADVANCED_WRITER;
use vga::colors::Color16;

extern crate rlibc;

mod vga_buffer;

extern crate alloc;

use core::panic::PanicInfo;
use os::println;
use os::memory::{self, BootInfoFrameAllocator};
use os::allocator;
use bootloader::{BootInfo, entry_point};
use x86_64::{VirtAddr, structures::paging::MapperAllSizes, structures::paging::Page};
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};

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
}

entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello World!");

    os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    os::hlt_loop();

    //MODE.lock().graphics_init();

    //for i in 0..60 {
    //    println!("{}", i);
    //};
    //let test_string = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzab";
    //for i in 0..60 {
    //    println!("abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzab");
    //}

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
