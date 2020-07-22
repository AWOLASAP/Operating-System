use crate::println;
use lazy_static::lazy_static;
use alloc::string::String;


lazy_static! {
    pub static ref COMMANDRUNNER: CommandRunner = CommandRunner::new(String::from(" "));
}

pub struct CommandRunner{
    command_buffer: String,
}

impl CommandRunner {
    fn new(string: String) -> CommandRunner {
        CommandRunner{
            command_buffer: String::new(),
        }

    }

    pub fn addToBuffer(&mut self, c: char) {
        self.command_buffer.push(c)
    }

    pub fn echo(&mut self, string: &str) {
        println!("\n{}", string);
    }
}

#[macro_export]
macro_rules! addCommandBuffer {
    ($c: expr) => {(COMMANDRUNNER.addToBuffer(c))};
}
