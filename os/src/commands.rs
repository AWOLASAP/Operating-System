use crate::println;
use lazy_static::lazy_static;
use alloc::string::String;
use spin::Mutex;
use crate::vga_buffer::MODE;
use x86_64::instructions::interrupts;

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

    pub fn add_to_buffer(&mut self, c: char) {
        let delete_char = char::from(8);
        if c == '\n' {
            self.eval_buffer();
        } else if c == delete_char {
            self.remove_from_buffer();
                 } else {
            self.command_buffer.push(c);
        }

    }

    pub fn addToBuffer(&mut self, c: char) {
        if (c == '\n'){
            self.eval_buffer();
        } else if (c == char::from(8)) {
            self.deleteLastInBuffer();
        } else {
            self.command_buffer.push(c);
        }

    }

    pub fn remove_from_buffer(&mut self) {
        self.command_buffer.pop();
    }
    pub fn deleteLastInBuffer(&mut self) {
        self.command_buffer.pop();
    }
    pub fn echo(&self, string: &str) {
        println!("\n{}", string);
    }

    pub fn print_buffer(&mut self) {
        println!("\nThe command buffer includes: {}", self.command_buffer);
    }

    pub fn eval_buffer(&mut self) {
        let (command, args) = self.split_buffer();
        if "print" == command {
            self.print_buffer();
        }
        else if "echo" == command {
            self.echo(args);
        }
        else if "gterm" == command {
            interrupts::without_interrupts(|| {
                MODE.lock().graphics_init();
            });
            println!("Graphical mode activated");
        }
        else if "tterm" == command {
            interrupts::without_interrupts(|| {
                MODE.lock().text_init();
            });
            println!("Text mode activated");
        }
        else {
            println!("\nInvalid Command!");
        }
        self.command_buffer = String::from("");
    }

    pub fn split_buffer(&self) -> (&str, &str) {
        for index in 0..self.command_buffer.len() {
            if &self.command_buffer.as_str()[index..index+1] == String::from(' ').as_str() {
                return (&self.command_buffer.as_str()[0..index], &self.command_buffer.as_str()[index + 1..self.command_buffer.len()])
            }
        }

        (&self.command_buffer.as_str(), "")
    }
}

pub fn add_command_buffer_FN(c: char) {
    interrupts::without_interrupts(|| {
        COMMANDRUNNER.lock().add_to_buffer(c);
    });
}
pub fn remove_command_buffer_FN() {     interrupts::without_interrupts(|| {
    COMMANDRUNNER.lock().remove_from_buffer();}); }

#[macro_export]
macro_rules! add_command_buffer {
    ($c: expr) => {crate::commands::add_command_buffer_FN($c)};
}

pub fn print_command_buffer_FN() {
    interrupts::without_interrupts(|| {
    COMMANDRUNNER.lock().print_buffer();
    });
}
