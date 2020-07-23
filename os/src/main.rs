#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
extern crate rlibc;

mod serial;

// Use these for things like buffer access
use os::vga_buffer::{MODE, WRITER, ADVANCED_WRITER, PrintWriter};
use vga::colors::Color16;
use os::print;
use os::println;
use os::memory::{self, BootInfoFrameAllocator};
use os::allocator;
use bootloader::{BootInfo, entry_point};
use x86_64::{VirtAddr, structures::paging::MapperAllSizes, structures::paging::Page};
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
use core::panic::PanicInfo;
use x86_64::instructions::interrupts;

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

entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    MODE.lock().init();
    // Use this to activate graphics mode - graphics mode implements all of the APIs that text mode implements, 
    // but it is  slower than text mode because it doesn't operate off of direct memory access. 
    // Activating graphics mode also enables graphics things like line drawing
    //MODE.lock().graphics_init();
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

    interrupts::without_interrupts(|| {
        
        MODE.lock().graphics_init();
        ADVANCED_WRITER.lock().enable_border();
        ADVANCED_WRITER.lock().clear_buffer();

        ADVANCED_WRITER.lock().draw_rect((0, 0), (640, 480), Color16::Blue);
        ADVANCED_WRITER.lock().draw_logo(320, 240, 30);
        for _i in 0..30 {
            ADVANCED_WRITER.lock().draw_rect((0, 0), (75, 480), Color16::Blue);
        }
        ADVANCED_WRITER.lock().clear_buffer();
        MODE.lock().text_init();
        println!("");
    });

    os::hlt_loop();

    //for i in 0..60 {
    //    println!("{}", i);
    //};
    // This is an example on how to reactivate text mode and deactivate graphics mode.
    // This then changes the background and foreground color.
    //WRITER.lock().set_back_color(Color16::White);
    //WRITER.lock().set_front_color(Color16::Black);
}
//2.5
//33.5

//55.5
