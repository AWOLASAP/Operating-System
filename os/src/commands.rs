#![allow(unused_variables)]
#![feature(in_band_lifetimes)]

use crate::println;
use lazy_static::lazy_static;
use alloc::string::String;
use spin::Mutex;
use crate::vga_buffer::MODE;
use x86_64::instructions::interrupts;
use alloc::vec::Vec;
use crate::tetris::TETRIS;

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

    pub fn remove_from_buffer(&mut self) {
        self.command_buffer.pop();
    }
  
    pub fn echo(&self, string: &str) {
        println!("\n{}", string);
    }

    pub fn print_buffer(&self) {
        println!("\nThe command buffer includes: {}", self.command_buffer);
    }
     
    pub fn eval_buffer(&mut self) {
        let mut index = 0;
        let (commands, args_list) = self.split_buffer();

        for command in commands {
            let args = args_list[index];
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
            else if "tetris" == command {
                if MODE.lock().text == true {
                    println!("\nYou need to be in graphical mode for that!  Try 'gterm'");
                } else {
                    TETRIS.lock().init();
                }
            } else if "help" == command {
                println!("\nList of available commands:");
                println!("print");
                println!("echo");
                println!("gterm");
                println!("tterm");
                println!("tetris");
            }
            else {
                println!("\nInvalid Command!");
            }
            index += 1;
        }
        self.command_buffer = String::from("");
    }

    pub fn split_buffer(&self) -> (Vec<&str>, Vec<&str>) {
        let mut commands = Vec::new();
        let mut args_list = Vec::new();
        let mut command_len: i32;
        
        for command in self.command_buffer.split(";"){
            let mut found_args = false;
            for index in 0..command.len() {
                if &command[index..index+1] == String::from(' ').as_str() {
                    commands.push(&command[0..index]);
                    args_list.push(&command[index + 1..command.len()]);
                    found_args = true;
                    break;
                }
            }    

            if found_args == false {
                commands.push(command);
                args_list.push("");
            }
        }

        

        /*
        for index in 0..self.command_buffer.len() {
            if &self.command_buffer.as_str()[index..index+1] == String::from(' ').as_str() {
                commands.push(&self.command_buffer.as_str()[0..index]);
                arguments.push(&self.command_buffer.as_str()[index + 1..self.command_buffer.len()]);
            }
        }
        */

        (commands, args_list)
    }
}

pub fn add_command_buffer_fn(c: char) {
        COMMANDRUNNER.lock().add_to_buffer(c);
}
pub fn remove_command_buffer_fn() { 
    COMMANDRUNNER.lock().remove_from_buffer(); 
}

#[macro_export]
macro_rules! add_command_buffer {
    ($c: expr) => {crate::commands::add_command_buffer_fn($c)};
}

pub fn print_command_buffer_fn() {
    COMMANDRUNNER.lock().print_buffer();
}
