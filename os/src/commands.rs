#![allow(unused_variables)]
use crate::println;
use lazy_static::lazy_static;
use alloc::string::String;
use spin::Mutex;
use crate::vga_buffer::{MODE, BUFFER_HEIGHT, BUFFER_HEIGHT_ADVANCED, ADVANCED_WRITER,WRITER,PrintWriter};
use vga::colors::Color16;
use x86_64::instructions::interrupts;
use alloc::vec::Vec;
use crate::tetris::TETRIS;
use crate::ustar::USTARFS;
use crate::alloc::string::ToString;
use crate::play_beep;
use crate::play_tet_ost;
use crate::vi::FAKE_VIM;
use x86::io::outw;
use crate::brainf::BRAINF;

pub fn from_str(input: &str) -> Result<Color16, &str> {
    match input {
        "black"=>Ok(Color16::Black),
        "blue"=>Ok(Color16::Blue),
        "green"=>Ok(Color16::Green),
        "cyan"=>Ok(Color16::Cyan),
        "red"=>Ok(Color16::Red),
        "magenta"=>Ok(Color16::Magenta),
        "brown"=>Ok(Color16::Brown),
        "lightgrey"=>Ok(Color16::LightGrey),
        "darkgrey"=>Ok(Color16::DarkGrey),
        "lightblue"=>Ok(Color16::LightBlue),
        "lightgreen"=>Ok(Color16::LightGreen),
        "lightcyan"=>Ok(Color16::LightCyan),
        "lightred"=>Ok(Color16::LightRed),
        "pink"=>Ok(Color16::Pink),
        "yellow"=>Ok(Color16::Yellow),
        "white"=>Ok(Color16::White),
        _ => Err("Not a valid color."),
    }
}


// Init a CommandRunner class to run commands for the user
lazy_static! {
    pub static ref COMMANDRUNNER: Mutex<CommandRunner> = Mutex::new(CommandRunner::new(String::from(" ")));
}

// CommandRunner really only needs a place to store the commands
pub struct CommandRunner{
    command_buffer: String,
    pub dir_id: u64,
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
            dir_id: 0,
            index: 0,
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
            print!("[user@rust {}]# ", USTARFS.lock().cwd(self.dir_id));
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
        println!("\nGraphical mode activated");
    }

    // tterm command.
    // Switches to text mode
    pub fn tterm(&self) {
        // Deadlock prevention
        interrupts::without_interrupts(|| {
            MODE.lock().text_init();
        });
        println!("\nText mode activated");
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
        match args{
            ""=>self.basic_help(),
            "print-buffer"=>self.print_buffer_help(),
            "echo"=>self.echo_help(),
            "gterm"=>self.gterm_help(),
            "tterm"=>self.tterm_help(),
            "mode"=>self.mode_help(),
            "tetris"=>self.tetris_help(),
            "beep"=>self.beep_help(),
            "tet-ost"=>self.tet_ost_help(),
            "clear"=>self.clear_help(),
            "logo"=>self.logo_help(),
            "ls"=>self.ls_help(),
            "dir"=>self.dir_help(),
            "cd"=>self.cd_help(),
            "cat"=>self.cat_help(),
            "mkdir"=>self.mkdir_help(),
            "rmdir"=>self.rmdir_help(),
            "defrag"=>self.defrag_help(),
            "write"=>self.write_help(),
            "touch"=>self.touch_help(),
            "rm"=>self.rm_help(),
            "touchhello"=>self.touchhello_help(),
            "help"=>self.help_help(),
            "set_text_color"=>self.set_text_color_help(),
            "set_background_color"=>self.set_background_color_help(),
            "exit"=>self.shut_down_help(),
            "vim"=>self.vim_help(),
            "proot"=>self.proot_help(),
            _=>print!("\nThat command doesn't exist."),
        }
    }

    // Prints all the possible commands and prompts user how to learn more about each command
    fn basic_help(&self) {
        println!("\nList of available commands:");
        print!("print-buffer, ");
        print!("echo, ");
        print!("gterm, ");
        println!("tterm");
        print!("mode, ");
        print!("tetris, ");
        print!("beep, ");
        println!("tet-ost");
        print!("clear, ");
        print!("logo, ");
        print!("ls, ");
        println!("dir");
        print!("cd, ");
        print!("cat, ");
        print!("mkdir, ");
        println!("rmdir");
        print!("defrag, ");
        print!("write, ");
        print!("touch, ");
        println!("rm");
        print!("touchhello, ");
        println!("set_text_color");
        print!("set_background_color, ");
        print!("proot, ");
        print!("vim, ");
        println!("exit");
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

    // Describes and displays options for the ls command
    fn ls_help(&self) {
        println!("\nCommand: ls");
        println!("Lists all of the files and directories in the current directory.");
        println!("No defined arguments, everything after ls will be ignored.");
    }

    // Describes and displays options for the dir command
    fn dir_help(&self) {
        println!("\nCommand: dir");
        println!("Lists all of the files and directories in the current directory.");
        println!("No defined arguments, everything after dir will be ignored.");
    }

    // Describes and displays options for the cd command
    fn cd_help(&self) {
        println!("\nCommand: cd");
        println!("Changes into the specified directory.");
        println!("One defined argument, the path to the target directory.");
        println!("To move up in the tree, use '..'");
    }

    // Describes and displays options for the mkdir command
    fn mkdir_help(&self) {
        println!("\nCommand: mkdir");
        println!("Creates a directory of the given name.");
        println!("One defined argument, the name of the new directory.");
    }

    // Describes and displays options for the rmdir command
    fn rmdir_help(&self) {
        println!("\nCommand: rmdir");
        println!("Deletes the specified directory.");
        println!("One defined argument, the name of the directory to be deleted.");
    }

    // Describes and displays options for the defrag command
    fn defrag_help(&self) {
        println!("\nCommand: defrag");
        println!("Runs a defragmentation process on system memory, cleaning up memory.");
        println!("No defined arguments, everything after defrag will be ignored.");
    }

    // Describes and displays options for the rm command
    fn rm_help(&self) {
        println!("\nCommand: rm");
        println!("Deletes the specified file.");
        println!("One defined argument, the name of the file to be deleted.");
    }

    // Describes and displays options for the touchhello command
    fn touchhello_help(&self) {
        println!("\nCommand: touchhello");
        println!("Creates a file of the specified name containing 'Hello World!'");
        println!("One defined argument, the name of the new file.");
    }

    // Describes and displays options for the cat command
    fn cat_help(&self) {
        println!("\nCommand: cat");
        println!("Prints the contents of the specified file to the terminal.");
        println!("One defined argument, the path to the target file.");
    }

    // Describes and displays options for the write command
    fn write_help(&self) {
        println!("\nCommand: write");
        println!("Writes the current changes.");
        println!("No defined arguments, everything after write will be ignored.");
    }

    // Describes and displays options for the write command
    fn touch_help(&self) {
        println!("\nCommand: touch");
        println!("Creates an empty file of the given name.");
        println!("One defined argument, the name of the target file.");
    }

    // Describes and displays options for the help command
    fn help_help(&self) {
        println!("\nCommand: help");
        println!("Displays information about terminal commands.");
        println!("One defined argument: optional command.");
    }

    // Describes and displays options for the exit command
    fn shut_down_help(&self) {
        println!("\nCommand: exit");
        println!("Shuts down the system.");
        println!("No defined arguments, everything after exit will be ignored.");
        println!("ONLY WORKS FOR QEMU, NOT REAL HARDWARE");
    }

    fn set_text_color_help(&self){
        println!("\nCommand: set_text_color");
        println!("Changes the text color.");
        println!("One defined argument: required color from list: Blue, Black, Green, Cyan, Red, Magenta, Brown, LightGrey, DarkGrey, LightBlue, LightGreen, LightCyan, LightRed, Pink, Yellow, or White (non-case sensitive).");
    }

    fn set_background_color_help(&self){
        println!("\nCommand: set_background_color");
        println!("Changes the background color.");
        println!("One defined argument: required color from list: Blue, Black, Green, Cyan, Red, Magenta, Brown, LightGrey, DarkGrey, LightBlue, LightGreen, LightCyan, LightRed, Pink, Yellow, or White (non-case sensitive).");
    }

    fn vim_help(&self){
        println!("\nCommand: vim");
        println!("Opens a text file for editng.");
        println!("One defined argument: File to edit");
    }

    fn proot_help(&self){
        println!("\nCommand: proot");
        println!("Prints the directory tree");
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

    pub fn set_text_color(&self, args: &str){
        let args = args.to_lowercase();
        let color = from_str(&args);
        let color = match color {
            Ok(color) => color,
            Err(why) => {println!("\n{}",why);return},
        };
        if MODE.lock().text {
            WRITER.lock().set_front_color(color);
            WRITER.lock().rerender_screen();
        } else {
            ADVANCED_WRITER.lock().set_front_color(color);
            ADVANCED_WRITER.lock().rerender_screen();
        }
    }

    pub fn set_background_color(&self, args: &str){
        let args = args.to_lowercase();
        let color = from_str(&args);
        let color = match color {
            Ok(color) => color,
            Err(why) => {println!("\n{}",why);return},
        };
        if MODE.lock().text {
            WRITER.lock().set_back_color(color);
            WRITER.lock().rerender_screen();
        } else {
            ADVANCED_WRITER.lock().set_back_color(color);
            ADVANCED_WRITER.lock().rerender_screen();
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

    pub fn ls(&self) {
        println!();
        for i in USTARFS.lock().list_files(self.dir_id) {
            println!("{}", i);
        }
        for i in USTARFS.lock().list_subdirectories(self.dir_id) {
            println!("{}", i);
        }
    }

    pub fn cd(&self, args: &str) {
        USTARFS.lock().change_directory(args.to_string(), self.dir_id);
    }

    pub fn mkdir(&self, args: &str) {
        USTARFS.lock().create_directory(args.to_string(), self.dir_id);
    }

    pub fn rmdir(&self, args: &str) {
        USTARFS.lock().remove_directory(args.to_string(), Some(self.dir_id));
    }

    pub fn defrag(&self) {
        USTARFS.lock().defragment();
    }

    pub fn rm(&self, args: &str) {
        USTARFS.lock().remove_file(args.to_string(), Some(self.dir_id));
    }

    pub fn touchhello(&self, args: &str) {
        let data = String::from("Hello World!");
        let data = data.into_bytes();
        USTARFS.lock().write_file(args.to_string(), data, Some(self.dir_id));    
    }

    pub fn cat(&self, args: &str) {
        let data = match USTARFS.lock().read_file(args.to_string(), Some(self.dir_id)) {
            Some(data) => data,
            None => Vec::new(),
        };
        println!();
        for i in data.iter() {
            print!("{}", *i as char);
        }
        println!();
    }

    pub fn write(&self) {
        USTARFS.lock().write();
    }

    pub fn touch(&self, args: &str) {
        let data = String::from(" ");
        let data = data.into_bytes();
        USTARFS.lock().write_file(args.to_string(), data, Some(self.dir_id));
    }

    pub fn proot(&self) {
        USTARFS.lock().print_root();
    }

    pub fn vim(&self, args: &str) {
        if MODE.lock().text {
            println!("\nYou need to be in graphical mode for that!  Try 'gterm'");
        } else {
            FAKE_VIM.lock().init(args.to_string(), Some(self.dir_id));
        }
    }

    pub fn brainf(&self, args: &str) {
        BRAINF.lock().init_keyboard(args.to_string(), Some(self.dir_id));
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

        #[allow(clippy::all)]
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
                "ls" => self.ls(),
                "dir" => self.ls(),
                "cd" => self.cd(args),
                "cat" => self.cat(args),
                "mkdir" => self.mkdir(args),
                "rmdir" => self.rmdir(args),
                "defrag" => self.defrag(),
                "write" => self.write(),
                "touch" => self.touch(args),
                "vim" => self.vim(args),
                "rm" => self.rm(args),
                "touchhello" => self.touchhello(args),
                "set_text_color"=>self.set_text_color(args),
                "set_background_color"=>self.set_background_color(args),
                "exit" => self.shut_down(),
                "proot" => self.proot(),
                "brainf" => self.brainf(args),
                _ => println!("Invalid Command: {}", command),
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

        // Go through the seperate commands in the buffer, each separated by a `;`
        for command in self.command_buffer.split(';'){

            let mut found_args = false;

            // Go through the individual command to see if args were provided
            for index in 0..command.len() {
                // ` ` indicates separation of command and args.
                // Add command to commands and args to args_list.
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
            if !found_args {
                commands.push(command);
                args_list.push("");
            }
        }

        // Return the list of commands and corresponding arguments
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
