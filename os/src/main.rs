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
use os::print;
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
use alloc::vec::Vec;
use os::ustar::Directory;
use os::ustar::USTARFS;
use os::commands::COMMANDRUNNER;

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
        //MODE.lock().text_init();
        println!("");
    });
    USTARFS.lock().init();
    //USTARFS.lock().set_all_files_to_write();
    //USTARFS.lock().write();
    //USTARFS.lock().print_root();
    COMMANDRUNNER.lock().init();
    /*
    let driv = ata_block_driver::AtaPio::try_new();
    let data = unsafe {driv.read_lba(0, 1)};
    
    for c in data.iter() {
        print!("{}", *c as char);
    }

    let mut file = Directory::from_block(data, 0);

    println!();

    let data = file.to_block();

    for c in data.iter() {
        print!("{}", *c as char);
    }
    println!();

    let mut dota = Vec::with_capacity(256);

    for i in 0..256 {
        dota.push(((data[2*i + 1] as u16) << 8) | data[2*i] as u16); 
    }

    unsafe {driv.write(0, 1, dota)};

    let data = unsafe {driv.read_lba(0, 1)};
    */
    /*let mut file = File::from_block(data, 0);
    let mut dota = Vec::with_capacity(256);
    let data = file.to_block();

    for i in 0..256 {
        dota.push(((data[2*i + 1] as u16) << 8) | data[2*i] as u16); 
    }

    unsafe {driv.write(0, 1, dota)};
    */
    /*

    let mut data = Vec::with_capacity(256);

    for i in 0..256 {
        data.push(0u16);
    }

    let data = unsafe {driv.write(0, 1, data)};

    let dota = unsafe {driv.read_lba(0, 1)};

    for c in dota.iter() {
        print!("{}", *c as char);
    }
    
*/
    let mut executor = Executor::new();
    //executor.spawn(Task::new(example_task()));
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
