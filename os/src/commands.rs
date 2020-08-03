#![allow(unused_variables)]
use crate::println;
use lazy_static::lazy_static;
use alloc::string::String;
use spin::Mutex;
use crate::vga_buffer::{MODE, BUFFER_HEIGHT, BUFFER_HEIGHT_ADVANCED, ADVANCED_WRITER};
use vga::colors::Color16;
use x86_64::instructions::interrupts;
use alloc::vec::Vec;
use crate::tetris::TETRIS;
use crate::play_beep;
use crate::play_tet_ost;
use x86::io::outw;


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

    // mode command
    // prints out the current mode,
    // either text mode or graphical
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
        if MODE.lock().text {
            println!("\nYou need to be in graphical mode for that!  Try 'gterm'");
        } else {
            TETRIS.lock().init();
        }
    }

    pub fn logo(&self) {
        if MODE.lock().text {
            println!("\nYou need to be in graphical mode for that!  Try 'gterm'");
        } else {
            ADVANCED_WRITER.lock().clear_buffer();

            ADVANCED_WRITER.lock().draw_rect((0, 0), (640, 480), Color16::Blue);
            ADVANCED_WRITER.lock().draw_logo(320, 240, 30);
        }
    }

    // help command.
    // Lists all available commands 
    pub fn help(&self, args: &str) {
        if args == String::from("").as_str() {
            self.basic_help();
        } else if args == String::from("print-buffer").as_str() {
            self.print_buffer_help();
        } else if args == String::from("echo").as_str() {
            self.echo_help();
        } else if args == String::from("gterm").as_str() {
            self.gterm_help();
        } else if args == String::from("tterm").as_str() {
            self.tterm_help()
        } else if args == String::from("mode").as_str() {
            self.mode_help();
        } else if args == String::from("tetris").as_str() {
            self.tetris_help();
        } else if args == String::from("beep").as_str() {
            self.beep_help();
        } else if args == String::from("tet-ost").as_str() {
            self.tet_ost_help();
        } else if args == String::from("clear").as_str() {
            self.clear_help();
        } else if args == String::from("logo").as_str() {
            self.logo_help();
        } else if args == String::from("help").as_str() {
            self.help_help();
        }
    }

    // Prints all the possible commands and prompts user how to learn more about each command
    fn basic_help(&self) {
        println!("\nList of available commands:");
        println!("print-buffer");
        println!("echo");
        println!("gterm");
        println!("tterm");
        println!("mode");
        println!("tetris");
        println!("beep");
        println!("tet-ost");
        println!("clear");
        println!("logo");
        println!("\nFor specific options try 'help <command name>'\n");
        println!("You can also run multiple commands at the same time by separating them with a semi-colon ';'\n");
    }

    // Describes and displays options for the print_buffer command
    fn print_buffer_help(&self) {
        println!("\nCommand: print-buffer");
        println!("Prints the contents of the command buffer to the terminal.");
        println!("No defined arguments, everything after print-buffer will be added to command buffer and printed out.");
    }

    // Describes and displays options for the echo command
    fn echo_help(&self) {
        println!("\nCommand: echo");
        println!("Prints whatever comes after the command to the terminal.");
        println!("No defined arguments, everything after echo will be 'echoed' to the terminal");
    }

    // Describes and displays options for the gterm command
    fn gterm_help(&self) {
        println!("\nCommand: gterm");
        println!("Changes display into Graphics Mode.");
        println!("No defined arguments, everything after gterm will be ignored");
    }

    // Describes and displays options for the tterm command
    fn tterm_help(&self) {
        println!("\nCommand: tterm");
        println!("Changes display into Text Mode.");
        println!("No defined arguments, everything after tterm will be ignored.");
    }

    // Describes and displays options for the mode command
    fn mode_help(&self) {
        println!("\nCommand: mode");
        println!("Prints the current display mode (gterm vs tterm).");
        println!("No defined arguments, everything after mode will be ignored.");
    }

    // Describes and displays options for the tetris command
    fn tetris_help(&self) {
        println!("\nCommand: tetris");
        println!("Starts a game of tetris, REQUIRES Graphics Mode.");
        println!("No defined arguments, everything after tetris will be ignored.");
        println!("To exit, press 'p'");
    }

    // Describes and displays options for the beep command
    fn beep_help(&self) {
        println!("\nCommand: beep");
        println!("Plays a sound on the PC speaker.");
        println!("One defined argument: the frequency of the sound - defaults to 0");
    }

    // Describes and displays options for the tet-ost command
    fn tet_ost_help(&self) {
        println!("\nCommand: tet-ost");
        println!("Plays the tetris theme song.");
        println!("One defined argument: number of times to repeat (for infinitely repeating enter 0) - defaults to 1");
    }

    // Describes and displays options for the clear command
    fn clear_help(&self) {
        println!("\nCommand: clear");
        println!("Clears the terminal.");
        println!("No defined arguments, everything after clear will be ignored.");
    }

    // Describes and displays options for the logo command
    fn logo_help(&self) {
        println!("\nCommand: logo");
        println!("Prints boot logo to terminal, REQUIRES Graphics Mode.");
        println!("No defined arguments, everything after logo will be ignored.");
    }

    // Describes and displays options for the help command
    fn help_help(&self) {
        println!("\nCommand: help");
        println!("Displays information about terminal commands.");
        println!("One defined argument: optional command.");
    }

    // beep command
    // Calls the pcspeaker and plays a beep for 2 cycles
    pub fn beep(&self, args: &str) {
        if args == " "{
            println!("\nWhat frequency do you want the beep?");
        } else {
            let freq: i32 = args.parse().unwrap_or(0);
            play_beep!(freq, 2);
        }
    }

    // tet-ost command
    // Plays the Tetris sound track through the pcspeaker
    pub fn tet_ost(&self, args: &str) {
        let num: i32 = args.parse().unwrap_or(1);
        play_tet_ost!(num);
    }
    
    // clear command
    // Clears the screen by writing a bunch on new lines
    pub fn clear(&self) {
        if MODE.lock().text {
            for line in 0..BUFFER_HEIGHT {
                println!();
            }
        } else {
            for line in 0..BUFFER_HEIGHT_ADVANCED {
                println!();
            }
        }
    }
   
    // yes command
    // Continuously prints y to get rid of those pesky 
    // "Would you like to do X [y/N]" messages
    pub fn yes(&self) {
        loop {
            println!("y");
        }
    }

    // shutdown command
    // shuts down the operating system
    // ONLY WORKS ON QEMU NOT ON REAL HARDWARE!
    pub fn shut_down(&self) {
        unsafe { outw(0x604, 0x2000); }
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
                "help" => self.help(args),
                "beep" => self.beep(args),
                "tet-ost" => self.tet_ost(args),
                "clear" => self.clear(),
                "logo" => self.logo(),
                "yes" => self.yes(),
                "exit" => self.shut_down(),
                _ => println!("\nInvalid Command: {}", command),
            }
            
            // Index increases as we move onto the next command
            index += 1;
        }
        
        // Clear the command buffer after an evaluation
        self.command_buffer = String::from("");
        self.index = 0;
    }

    // Split the command buffer into its various parts
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

// Macro for moving the command cursor
#[macro_export]
macro_rules! move_command_cursor {
    ($c: expr) => {crate::commands::move_command_cursor_fn($c)};
}
