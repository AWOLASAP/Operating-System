#![allow(unused_variables)]
use crate::println;
use lazy_static::lazy_static;
use alloc::string::String;
use spin::Mutex;
use crate::vga_buffer::MODE;
use crate::vga_buffer::BUFFER_HEIGHT;
use x86_64::instructions::interrupts;
use alloc::vec::Vec;
use crate::tetris::TETRIS;
use crate::play_beep;
use crate::play_tet_ost;


// Init a CommandRunner class to run commands for the user
lazy_static! {
    pub static ref COMMANDRUNNER: Mutex<CommandRunner> = Mutex::new(CommandRunner::new(String::from(" ")));
}

// CommandRunner really only needs a place to store the commands
pub struct CommandRunner{
    command_buffer: String,
    index: usize,
}

// Implementation of CommandRunner. 
// Essentially it handles a command buffer, with
// commands inside that it can be executed upon
impl CommandRunner {

    // Create a new CommandRunner with an empty command_buffer string
    pub fn new(string: String) -> CommandRunner {
        CommandRunner{
            command_buffer: String::new(),
            index: 0,
        }

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
            if self.index < self.command_buffer.len() {
                self.command_buffer.remove(self.index);
                self.command_buffer.insert(self.index, c);
            } else {
                self.command_buffer.push(c);
            }
            self.index += 1;
        }

    }

    // Remove the last char from the command buffer
    pub fn remove_from_buffer(&mut self) {
        if self.index != 0 {
            self.command_buffer.remove(self.index - 1);
            self.index -= 1;
        }
    }

    // Moves the index in the command buffer when you move the cursor
    pub fn move_command_cursor(&mut self, i: i8) {
        if i > 0 && self.index < self.command_buffer.len() {
            self.index += 1;
        } else if i < 0 && self.index != 0 {
            self.index -= 1;
        }
    }
    
    // print-buffer command.
    // Prints out the command buffer to the screen before it get cleared
    pub fn print_buffer(&self) {
        println!("\nThe command buffer includes: {}", self.command_buffer);
    }

    // echo command.
    // Prints out the arguments given
    pub fn echo(&self, string: &str) {
        println!("\n{}", string);
    }

    // gterm command.
    // Switches to graphical mode
    pub fn gterm(&self) {
        // Deadlock prevention
        interrupts::without_interrupts(|| {
            MODE.lock().graphics_init();
        });
        println!("Graphical mode activated");
    }

    // tterm command.
    // Switches to text mode
    pub fn tterm(&self) {
        // Deadlock prevention
        interrupts::without_interrupts(|| {
            MODE.lock().text_init();
        });
        println!("Text mode activated");
    }

    pub fn mode(&self) {
        if MODE.lock().text {
            println!("\nText mode is active");
        } else {
            println!("\nGraphics mode is active");
        }
    }
    
    // tetris command
    // Plays the game Tetris
    pub fn tetris(&self) {
        // Run tetris if in the gterm mode
        if MODE.lock().text == true {
            println!("\nYou need to be in graphical mode for that!  Try 'gterm'");
        } else {
            TETRIS.lock().init();
        }
    }

    // help command.
    // Lists all available commands 
    pub fn help(&self) {
        println!("\nList of available commands:");
        println!("print-buffer");
        println!("echo");
        println!("gterm");
        println!("tterm");
        println!("tetris");
        println!("tet-ost");
        println!("clear");
    }

    pub fn beep(&self, args: &str) {
        if args == " "{
            println!("\nWhat frequency do you want the beep?");
        } else {
            let freq: i32 = args.parse().unwrap_or(0);
            play_beep!(freq, 2);
        }
    }
    
    pub fn tet_ost(&self) {
        play_tet_ost!();
    }
    
    pub fn clear(&self) {
        for line in 0..BUFFER_HEIGHT {
            println!();
        }
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

            match command {
                "print-buffer" => self.print_buffer(),
                "echo" => self.echo(args),
                "gterm" => self.gterm(),
                "tterm" => self.tterm(),
                "mode" => self.mode(),
                "tetris" => self.tetris(),
                "help" => self.help(),
                "beep" => self.beep(args),
                "tet-ost" => self.tet_ost(),
                "clear" => self.clear(),
                _ => println!("\nInvalid Command: {}", command),
            }
            
            // Index increases as we move onto the next command
            index += 1;
        }
        
        // Clear the command buffer after an evaluation
        self.command_buffer = String::from("");
        self.index = 0;
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

// Calls the CommandRunner class to move cursor in buffer
pub fn move_command_cursor_fn(i: i8) {
    COMMANDRUNNER.lock().move_command_cursor(i);
}

// Macro for adding to the command buffer from different files
#[macro_export]
macro_rules! add_command_buffer {
    ($c: expr) => {crate::commands::add_command_buffer_fn($c)};
}

#[macro_export]
macro_rules! move_command_cursor {
    ($c: expr) => {crate::commands::move_command_cursor_fn($c)};
}