#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(wake_trait)]

extern crate rlibc;
extern crate alloc;

use core::panic::PanicInfo;

// defines the modules in the project
pub mod serial;
pub mod vga_buffer;
pub mod interrupts;
pub mod gdt;
pub mod memory;
pub mod allocator;
pub mod task;
pub mod commands;
pub mod rng;
pub mod keyboard_routing;
pub mod timer_routing;
pub mod tetris;
pub mod ata_block_driver;
pub mod ustar;
pub mod speaker;
pub mod vi;

// defines the Testable trait
pub trait Testable {
    fn run(&self);
}

// implementation of the Testable trait
impl<T> Testable for T
    where
        T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

// creates a test runner for running test functions
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

// defines a special panic handler for when running tests
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

// enum that defines the Qemu Exit codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

// exits qemu with the corresponding exit code
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

//defines the entry point when testing
#[cfg(test)]
entry_point!(test_kernel_main);

// entry point when testing
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

// panic function for when testing
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

// initializes everything in the os
pub fn init() {
    interrupts::init_idt();
    gdt::init();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

// halts the cpu when not working
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

// handles errors with heap allocation
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
