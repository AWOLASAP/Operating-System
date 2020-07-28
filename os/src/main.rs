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
use os::println;
use os::memory::{self, BootInfoFrameAllocator};
use os::allocator;
use bootloader::{BootInfo, entry_point};
use x86_64::{VirtAddr};
use core::panic::PanicInfo;
use os::task::{Task};
use os::task::executor::Executor;
use os::task::keyboard;
use x86_64::instructions::interrupts;
use os::ata_block_driver;

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

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
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
    ata_block_driver::AtaPio::try_new();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    #[cfg(test)]
    test_main();
    
    
    interrupts::without_interrupts(|| {

        MODE.lock().graphics_init();
        //ADVANCED_WRITER.lock().enable_border();
        ADVANCED_WRITER.lock().clear_buffer();

        ADVANCED_WRITER.lock().draw_rect((0, 0), (640, 480), Color16::Blue);
        ADVANCED_WRITER.lock().draw_logo(320, 240, 30);
        for _i in 0..30 {
            ADVANCED_WRITER.lock().draw_rect((0, 0), (75, 480), Color16::Blue);
        }
        //ADVANCED_WRITER.lock().clear_buffer();
        MODE.lock().text_init();
        println!("");
    });
    

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();


    //let mut executor = Executor::new();
    //executor.spawn(Task::new(example_task()));
    //executor.spawn(Task::new(keyboard::print_keypresses()));
    //executor.run();
    //os::hlt_loop();

    //for i in 0..60 {
    //    println!("{}", i);
    //};
    // This is an example on how to reactivate text mode and deactivate graphics mode.
    // This then changes the background and foreground color.
    // MODE.lock().text_init();
    //WRITER.lock().set_back_color(Color16::White);
    //WRITER.lock().set_front_color(Color16::Black);
}
