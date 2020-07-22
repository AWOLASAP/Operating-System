use crate::println;
use lazy_static::lazy_static;
use alloc::string::String;
use spin::Mutex;


lazy_static! {
    pub static ref COMMANDRUNNER: Mutex<CommandRunner> = Mutex::new(CommandRunner::new(String::from(" ")));
}

pub struct CommandRunner{
    command_buffer: String,
}

impl CommandRunner {
    pub fn new(string: String) -> CommandRunner {
        CommandRunner{
            command_buffer: String::new(),
        }

    }

    pub fn addToBuffer(&mut self, c: char) {
        self.command_buffer.push(c);
        if (c == 'p') {
            self.printBuffer();
        }
    }

    pub fn echo(&mut self, string: &str) {
        println!("\n{}", string);
    }

    pub fn printBuffer(&mut self) {
        println!("The command buffer includes: {}", self.command_buffer);
    }
}

pub fn addCommandBufferFN(c: char) {
    COMMANDRUNNER.lock().addToBuffer(c);
}

#[macro_export]
macro_rules! addCommandBuffer {
    ($c: expr) => {commands::addCommandBufferFN($c)};
}

pub fn printCommandBuffer() {
    COMMANDRUNNER.lock().printBuffer();
}
