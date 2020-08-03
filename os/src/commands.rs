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
use crate::ustar::USTARFS;
use crate::alloc::string::ToString;

// Init a CommandRunner class to run commands for the user
lazy_static! {
    pub static ref COMMANDRUNNER: Mutex<CommandRunner> = Mutex::new(CommandRunner::new(String::from(" ")));
}

// CommandRunner really only needs a place to store the commands
pub struct CommandRunner{
    command_buffer: String,
    dir_id: u64,
}

// Implementation of CommandRunner. 
// Essentially it handles a command buffer, which commands inside 
// it can be executed by CommandRunner
impl CommandRunner {

    // Create a new CommandRunner with an empty command_buffer string
    pub fn new(string: String) -> CommandRunner {
        CommandRunner{
            command_buffer: String::new(),
            dir_id: 0,
        }

    }

    pub fn init(&mut self) {
        self.dir_id = USTARFS.lock().get_id();
    }

    // Add a character to the command buffer. 
    // This was used instead of reading what was on the screen
    // due to it being easier and more reliable.
    pub fn add_to_buffer(&mut self, c: char) {
        let backspace_char = char::from(8);
        if c == '\n' {
            // If the char is a newline, evaluate the buffer
            self.eval_buffer();
        } else if c == backspace_char {
            // If the char is a backspace, remove the last character from the buffer
            self.remove_from_buffer();
        } else {
            // If not a special case, just add the char to the buffer
            self.command_buffer.push(c);
        }

    }

    // Remove the last char from the command buffer
    pub fn remove_from_buffer(&mut self) {
        self.command_buffer.pop();
    }

    // echo command.
    // Prints out the arguments given
    pub fn echo(&self, string: &str) {
        println!("\n{}", string);
    }

    // print command.
    // Prints out the command buffer to the screen before it get cleared
    pub fn print_buffer(&self) {
        println!("\nThe command buffer includes: {}", self.command_buffer);
    }

    // Evaluate the command(s) in the buffer 
    pub fn eval_buffer(&mut self) {
        // Index to keep track of the command number for the argument number
        let mut index = 0;
        // Split up the command buffer into multiple commands,
        // each with a corresponding argument
        let (commands, args_list) = self.split_buffer();

        for command in commands {
            // Get the corresponding args for the current command
            let args = args_list[index];

            if "print" == command {
                self.print_buffer();
            }
            else if "echo" == command {
                self.echo(args);
            }
            else if "gterm" == command {
                // Deadlock prevention
                interrupts::without_interrupts(|| {
                    MODE.lock().graphics_init();
                });
                println!("Graphical mode activated");
            }
            else if "tterm" == command {
                // Deadlock prevention
                interrupts::without_interrupts(|| {
                    MODE.lock().text_init();
                });
                println!("Text mode activated");
            }
            else if "tetris" == command {
                // Run tetris if in the gterm mode
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
            else if "ls" == command {
                println!("");
                for i in USTARFS.lock().list_files(self.dir_id) {
                    println!("{}", i);
                }                
                for i in USTARFS.lock().list_subdirectories(self.dir_id) {
                    println!("{}", i);
                }
            }
            else if "cd" == command {
                USTARFS.lock().change_directory(args.to_string(), self.dir_id);
            }
            else if "cat" == command {
                let data = match USTARFS.lock().read_file(args.to_string(), Some(self.dir_id)) {
                    Some(data) => data,
                    None => Vec::new(),
                };
                println!("");
                for i in data.iter() {
                    print!("{}", *i as char);
                }
                println!("");               

            }
            else if "mkdir" == command {
                USTARFS.lock().create_directory(args.to_string(), self.dir_id);

            }
            else if "rmdir" == command {
                USTARFS.lock().remove_directory(args.to_string(), self.dir_id);
            }
            else if "defrag" == command {
                USTARFS.lock().defragment();
            }
            else if "write" == command {
                USTARFS.lock().write();
            }
            else {
                println!("\nInvalid Command!");
            }

            // Index increases as we move onto the next command
            index += 1;
        }
        
        // Clear the command buffer after an evaluation
        self.command_buffer = String::from("");
    }

    pub fn split_buffer(&self) -> (Vec<&str>, Vec<&str>) {
        // Variables for the various parts of the command_buffer
        let mut commands = Vec::new();
        let mut args_list = Vec::new();
        let mut command_len: i32;
        
        // Go through the seperate commands in the buffer, each seperated by a `;`
        for command in self.command_buffer.split(";"){

            let mut found_args = false;

            // Go through the indivual command to see if args were provided
            for index in 0..command.len() {
                // ` ` indicates seperation of command and args.
                // Add command to scommands and args to args_list.
                if &command[index..index+1] == String::from(' ').as_str() {
                    commands.push(&command[0..index]);
                    args_list.push(&command[index + 1..command.len()]);
                    found_args = true;
                    break;
                }
            }    

            // If no arguments were found,
            // make sure the command still gets added,
            // and the argument added is blank
            if found_args == false {
                commands.push(command);
                args_list.push("");
            }
        }

        // Return the list of commands and corresponding arguemnts
        (commands, args_list)
    }
}

// Calls the CommandRunner class to add a char to the buffer
pub fn add_command_buffer_fn(c: char) {
        COMMANDRUNNER.lock().add_to_buffer(c);
}

// Macro for adding to the command buffer from different files
#[macro_export]
macro_rules! add_command_buffer {
    ($c: expr) => {crate::commands::add_command_buffer_fn($c)};
}

