#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
extern crate rlibc;

mod serial;

// Use these for things like buffer access
use os::vga_buffer::{MODE, ADVANCED_WRITER};
use vga::colors::Color16;
use os::{println,allocator};
use os::memory::{self, BootInfoFrameAllocator};
use bootloader::{BootInfo, entry_point};
use x86_64::{VirtAddr};
use core::panic::PanicInfo;
use os::task::{Task,keyboard,executor::EXECUTOR};
use x86_64::instructions::interrupts;

// defines a panic function for when not testing
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    os::hlt_loop();
}

// defines a panic function for when testing
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

// creates the entry point for the application
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

    #[cfg(test)]
    test_main();

    // interrupts::without_interrupts(|| {
    //
    //     MODE.lock().graphics_init();
    //     //ADVANCED_WRITER.lock().enable_border();
    //     ADVANCED_WRITER.lock().clear_buffer();
    //
    //     ADVANCED_WRITER.lock().draw_rect((0, 0), (640, 480), Color16::Blue);
    //     ADVANCED_WRITER.lock().draw_logo(320, 240, 30);
    //     for _i in 0..30 {
    //         ADVANCED_WRITER.lock().draw_rect((0, 0), (75, 480), Color16::Blue);
    //     }
    //     //ADVANCED_WRITER.lock().clear_buffer();
    //     MODE.lock().text_init();
    //     println!("");
    // });

    EXECUTOR.write().spawn(Task::new(example_task()));
    EXECUTOR.write().spawn(Task::new(keyboard::print_keypresses()));
    EXECUTOR.write().run();
}
