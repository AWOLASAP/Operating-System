use crate::println;
use lazy_static::lazy_static;
use spin::Mutex;


lazy_static! {
    pub static ref COMMANDRUNNER: Mutex<CommandRunner> = {
        Mutex::new(CommandRunner::new())
    };
}


pub struct CommandRunner{
    command_buffer: &str,
}

impl CommandRunner {
    fn new() -> CommandRunner {
        CommandRunner{
            command_buffer: "",
        }

    }

    pub fn add_to_buffer(&mut self, c: char){
        self.command_buffer = self.command_buffer.to_owned();
        let borrowed_string: &str = c.to_string();

        self.command_buffer.push_str(borrowed_string);
    }

    pub fn echo(&mut self, string: &str) {
        println!("\n{}", string);
    }
}

#[macro_export]
macro_rules! addCommandBuffer {
    ($e:expr) => (COMMANDRUNNER.addToBuffer(e));
}
