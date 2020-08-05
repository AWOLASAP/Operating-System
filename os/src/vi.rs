use lazy_static::lazy_static;
use spin::{Mutex};
use crate::vga_buffer::ADVANCED_WRITER;
use vga::colors::Color16;
use alloc::vec::Vec;
use alloc::string::String;
use crate::ustar::USTARFS;
use crate::println;
use crate::keyboard_routing::KEYBOARD_ROUTER;
use crate::timer_routing::TIME_ROUTER;
use x86_64::instructions::interrupts;
use crate::vga_buffer::PrintWriter;
use crate::alloc::string::ToString;
use crate::commands::COMMANDRUNNER;

// This is probably not going to be "real" vi, but enough of a clone to do basic stuff.

const SCREEN_WIDTH: usize = 76;
const SCREEN_HEIGHT: usize = 56;

pub struct FakeVim {
    data: Vec<u8>,
    data_screen_index: usize,
    colon_pressed: bool,
    is_active: bool,
    char_buffer:  [[char; SCREEN_WIDTH]; SCREEN_HEIGHT],
    cursor_x: isize,
    cursor_y: isize,
    cursor_blink_timer: isize,
    blink_on: bool,
    command_mode: bool,
    input_mode: bool,
    command_buffer: String,
    filename: String,
    id: Option<u64>,
}

impl Default for FakeVim {
    fn default() -> FakeVim {
        FakeVim::new()
    }
}

impl FakeVim {
    pub fn new() -> Self {
        FakeVim{
            data: Vec::new(), 
            data_screen_index: 0,
            colon_pressed: false,
            is_active: false,
            char_buffer: [['\0'; SCREEN_WIDTH]; SCREEN_HEIGHT],
            cursor_x: 0,
            cursor_y: 0,
            cursor_blink_timer: 5,
            blink_on: false,
            command_mode: false,
            input_mode: false,
            command_buffer: String::new(),
            filename: String::new(),
            id: None,
        }
    }

    pub fn handle_scancode(&mut self, scancode: char) {
        if self.input_mode {
            self.insert_char(scancode);
        }
        else if self.command_mode {
            self.handle_command(scancode);
        }
        else {
            match scancode as char {
                'i' => {
                    self.command_mode = false;
                    self.input_mode = true;
                },
                ':' => {
                    self.command_mode = true;
                    self.command_buffer = String::new();
                    self.handle_command(scancode);
                }
                _ => {

                },
            }
        }
        //Delete 
        // Arrow Keys + vim keys
        // Command mode
        // Keyboard input
    }

    pub fn handle_command(&mut self, command: char) {
        if command == '\n' {
            //Process command
            for i in self.command_buffer.as_bytes() {
                match *i as char {
                    'i' => {
                        ADVANCED_WRITER.lock().wipe_buffer();
                        self.command_mode = false;
                        self.input_mode = true;
                    },
                    'q' => {
                        interrupts::without_interrupts(|| {
                            ADVANCED_WRITER.lock().wipe_buffer();
                            unsafe {KEYBOARD_ROUTER.force_unlock()};
                            KEYBOARD_ROUTER.lock().mode.textedit = false;
                            KEYBOARD_ROUTER.lock().mode.terminal = true;
                            TIME_ROUTER.lock().mode.vim = false;
                            ADVANCED_WRITER.lock().enable_blink();
                            println!();
                            print!("[user@rust {}]# ", USTARFS.lock().cwd(COMMANDRUNNER.lock().dir_id));
                        });
                        return
                    },
                    'e' => {
                        if let Some(data) = USTARFS.lock().read_file(self.filename.to_string(), self.id) {
                            self.data = data;
                        }
                    }
                    'w' => {
                        USTARFS.lock().remove_file(self.filename.to_string(), self.id);
                        USTARFS.lock().write_file(self.filename.to_string(), self.data.clone(), self.id);
                    },
                    _ => {

                    },
                }
            }
            self.render_buffer();
            self.command_mode = false;
            self.command_buffer = String::new();
            return;
        }
        else if command == 0x08 as char {
            self.command_buffer.pop();
        }
        else {
            self.command_buffer.push(command);
        }
        for i in 0..10 {
            ADVANCED_WRITER.lock().draw_char(560 + i * 8, 472, ' ', Color16::Black, Color16::Black);
        }
        for (i, c) in self.command_buffer.as_bytes().iter().enumerate() {
            ADVANCED_WRITER.lock().draw_char(560 + i * 8, 472, *c as char, Color16::White, Color16::Black);
        }
    }

    pub fn up(&mut self) {
        self.un_blink();
        self.cursor_y -= 1;
        if self.cursor_y < 0 {
            self.cursor_y = 0;
            if self.data_screen_index > SCREEN_WIDTH {
                self.data_screen_index -= SCREEN_WIDTH;
                for i in (0..(SCREEN_WIDTH - 1)).rev() {
                    if let Some(chr) = self.data.get(self.data_screen_index + i) {
                        if *chr as char == '\n' {
                            self.data_screen_index += (i as isize - 1) as usize;
                            break;
                        }
                    }
                }
            }
            else {
                self.data_screen_index = 0;
            }
        }
        while self.char_buffer[self.cursor_y as usize][self.cursor_x as usize] == '\0' {
            self.cursor_x -= 1;
        }
        self.render_buffer();
    }

    pub fn down(&mut self) {
        self.un_blink();
        self.cursor_y += 1;
        if self.cursor_y == SCREEN_HEIGHT as isize {
            self.cursor_y = SCREEN_HEIGHT as isize - 1;
            let mut broke = false;
            for i in 0..SCREEN_WIDTH {
                if let Some(chr) = self.data.get(self.data_screen_index + i) {
                    if *chr as char == '\n' {
                        self.data_screen_index += (i as isize + 1) as usize;
                        broke = true;
                        break;
                    }
                }
            }
            if !broke {
                self.data_screen_index += SCREEN_WIDTH;
            }
        }
        while self.char_buffer[self.cursor_y as usize][self.cursor_x as usize] == '\0' {
            self.cursor_x -= 1;
        }
        self.render_buffer();
    }

    pub fn left(&mut self) {
        let mut top = false;
        if self.data_screen_index == 0 && self.cursor_y == 0 {
            top = true;
        }
        self.un_blink();
        self.cursor_x -= 1;
        if self.cursor_x < 0 {
            self.cursor_x = SCREEN_WIDTH as isize - 1;
            if top {
                self.cursor_x = 0
            }
            self.up();
        }

        self.render_buffer();
    }

    pub fn right(&mut self) {
        self.un_blink();
        self.cursor_x += 1;
        if self.cursor_x == SCREEN_WIDTH as isize {
            self.cursor_x = 0;
            self.down();
        }
        if self.char_buffer[self.cursor_y as usize][self.cursor_x as usize] == '\0' {
            self.cursor_x -= 1;
        }
        self.render_buffer();
    }

    pub fn handle_esc(&mut self) {
        self.input_mode = false;
        self.command_mode = false;
        ADVANCED_WRITER.lock().wipe_buffer();
        self.render_buffer();
    }

    pub fn init(&mut self, file: String, id: Option<u64>) {
        if let Some(data) = USTARFS.lock().read_file(file.to_string(), id) {
            self.data = data;
            // Init the keyboard stuff
            interrupts::without_interrupts(|| {
                ADVANCED_WRITER.lock().wipe_buffer();
                unsafe {KEYBOARD_ROUTER.force_unlock()};
                KEYBOARD_ROUTER.lock().mode.textedit = true;
                KEYBOARD_ROUTER.lock().mode.terminal = false;
                KEYBOARD_ROUTER.lock().mode.screenbuffer = false;
                TIME_ROUTER.lock().mode.vim = true;
                //TIME_ROUTER.lock().mode.terminal = false;
                ADVANCED_WRITER.lock().disable_blink();
            });
            self.cursor_x = 0;
            self.cursor_y = 0;
            self.data_screen_index = 0;
            self.colon_pressed = false;
            self.is_active = false;
            self.char_buffer = [[' '; SCREEN_WIDTH]; SCREEN_HEIGHT];
            self.cursor_blink_timer = 5;
            self.blink_on = false;
            self.filename = file;
            self.id = id;
            self.command_buffer = String::new();
            self.command_mode = false;
            self.input_mode = false;
            self.render_buffer();
        }
        else {
            println!("File doesn't exist");

        }
    }
    
    pub fn render_buffer(&mut self) {
        // Use the buffer for diffs, and render directly off the vector 
        let mut x = 0;
        let mut y = 0;
        //ADVANCED_WRITER.lock().clear_screen(Color16::Black);
        for (i, d) in self.data.iter().enumerate() {
            if i >= self.data_screen_index {
                // Handles newline
                if *d == 10 {
                    self.char_buffer[y][x] = '\n';
                    ADVANCED_WRITER.lock().draw_char(
                        (x + 2) * 8, 
                        (y + 2) * 8, 
                        0 as char, 
                        Color16::Black, 
                        Color16::Black,
                    );
                    for j in (x + 1)..SCREEN_WIDTH {
                        if self.char_buffer[y][j] != 0 as char {
                            ADVANCED_WRITER.lock().draw_char(
                                (j + 2) * 8, 
                                (y + 2) * 8, 
                                0 as char, 
                                Color16::Black, 
                                Color16::Black,
                            );
                            self.char_buffer[y][j] = 0 as char;
                        }
                    }
                    x = 0;
                    y += 1;
                }
                // Conditional rendering is mostly bugged, we don't care though
                else {
                    ADVANCED_WRITER.lock().draw_char(
                        (x + 2) * 8, 
                        (y + 2) * 8, 
                        *d as char, 
                        Color16::White, 
                        Color16::Black,
                    );
                    self.char_buffer[y][x] = *d as char;
                    x += 1;
                }
                if x >= SCREEN_WIDTH {
                    y += 1;
                    x = 0;
                }
                if y >= SCREEN_HEIGHT {
                    // I'm sorry Mr. Tinling
                    break;
                }
            }
        }
        if self.input_mode {
            ADVANCED_WRITER.lock().clear_buffer();
            println!("insert");
        }
        


    }

    pub fn insert_char(&mut self, chr: char) {
        // Include things like line changing logic
        // Insert the character at the cursor (before)

        // Use the buffer for diffs, and render directly off the vector 
        let mut x = 0;
        let mut y = 0;
        //ADVANCED_WRITER.lock().clear_screen(Color16::Black);
        for (i, d) in self.data.iter().enumerate() {
            if i >= self.data_screen_index {
                // Handles newline
                if self.cursor_x as usize == x && self.cursor_y as usize == y {
                    if chr as u8 == 0x08 {
                        if i != 0 {
                            self.data.remove(i - 1);
                        }
                        else {
                            self.data.remove(i);
                        }
                        self.left();
                    }
                    else {
                        self.data.insert(i, chr as u8);
                        self.right();

                    }
                    break;
                }

                if *d == 10 {
                    x = 0;
                    y += 1;
                }
                // Conditional rendering is mostly bugged, we don't care though
                else {
                    x += 1;
                }
                if x >= SCREEN_WIDTH {
                    y += 1;
                    x = 0;
                }
                if y >= SCREEN_HEIGHT {
                    // I'm sorry Mr. Tinling
                    break;
                }
            }
        }
        self.render_buffer();
    }

    pub fn un_blink(&mut self) {
        interrupts::without_interrupts(|| {
            self.blink_on = false;
            ADVANCED_WRITER.lock().draw_char(
                (self.cursor_x as usize + 2) * 8, 
                (self.cursor_y as usize + 2) * 8, 
                self.char_buffer[self.cursor_y as usize][self.cursor_x as usize], 
                Color16::White, 
                Color16::Black,
            );
        });
    }

    pub fn blink(&mut self) {
        interrupts::without_interrupts(|| {
            self.blink_on = true;
            ADVANCED_WRITER.lock().draw_char(
                (self.cursor_x as usize + 2) * 8, 
                (self.cursor_y as usize + 2) * 8, 
                self.char_buffer[self.cursor_y as usize][self.cursor_x as usize], 
                Color16::White, 
                Color16::White,
            );
        });
    }

    pub fn blink_cursor(&mut self) {
        if self.cursor_blink_timer < 0 {
            if self.blink_on {
                self.un_blink();
            }
            else {
                self.blink();
            }
            self.cursor_blink_timer = 4;
        }
        else {
            self.cursor_blink_timer -= 1;
        }

    }
}

lazy_static! {
    pub static ref FAKE_VIM: Mutex<FakeVim> = {
        Mutex::new(FakeVim::new())
    };
}
