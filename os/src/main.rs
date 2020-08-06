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
use os::{println,allocator,print};
use os::memory::{self, BootInfoFrameAllocator};
use bootloader::{BootInfo, entry_point};
use x86_64::{VirtAddr};
use core::panic::PanicInfo;
use os::task::{Task,keyboard,executor::Executor};
use x86_64::instructions::interrupts;
use os::ustar::USTARFS;
use os::commands::COMMANDRUNNER;
use lazy_static::lazy_static;
use spin::Mutex;

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

// creates the entry point for the application
entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    MODE.lock().init();
    // Use this to activate graphics mode - graphics mode implements all of the APIs that text mode implements,
    // but it is  slower than text mode because it doesn't operate off of direct memory access.
    // Activating graphics mode also enables graphics things like line drawing
    //MODE.lock().graphics_init();
    //println!("Hello World!");

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


    interrupts::without_interrupts(|| {

        MODE.lock().graphics_init();
        //ADVANCED_WRITER.lock().enable_border();
        ADVANCED_WRITER.lock().clear_buffer();

        ADVANCED_WRITER.lock().draw_rect((0, 0), (640, 480), Color16::Blue);
        ADVANCED_WRITER.lock().draw_logo(320, 240, 30);
        for _i in 0..30 {
            ADVANCED_WRITER.lock().draw_rect((0, 0), (75, 480), Color16::Blue);
        }
        ADVANCED_WRITER.lock().clear_buffer();
        MODE.lock().text_init();
        println!();
    });
    USTARFS.lock().init();
    //USTARFS.lock().set_all_files_to_write();
    //USTARFS.lock().write();
    //USTARFS.lock().print_root();
    COMMANDRUNNER.lock().init();

    print!("[user@rust /]# ");
    COMMANDRUNNER.lock().prompt_length = 15;
    EXECUTOR.lock().spawn(Task::new(keyboard::print_keypresses()));
    EXECUTOR.lock().run();
}

    // This is an example on how to reactivate text mode and deactivate graphics mode.
    // This then changes the background and foreground color.
    // MODE.lock().text_init();
    //WRITER.lock().set_back_color(Color16::White);
    //WRITER.lock().set_front_color(Color16::Black);

lazy_static! {
    pub static ref EXECUTOR: Mutex<Executor> = {
        Mutex::new(Executor::new())
    };
}
