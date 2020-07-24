use lazy_static::lazy_static;
use crate::vga_buffer::{MODE, WRITER, ADVANCED_WRITER, PrintWriter};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyCode};
use spin::Mutex;
use crate::add_command_buffer;
use crate::tetris::TETRIS;

/* MODES
0 - Terminal + sends weird stuff to screenbuffer
1 - Screenbuffer only
2 - Tetris - only sends tetris keybinds
*/
pub struct KeyboardRouter {
    pub mode: usize,
}

impl KeyboardRouter {
    fn new() -> KeyboardRouter {
        KeyboardRouter { mode: 0 }
    }

    pub fn handle_scancode(&mut self, scancode: u8) {
        let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1,
            HandleControl::Ignore);
        match scancode{
            // We need to find the right scancode for this (escape)
            27=>self.esc(),
            // Arrow keys
            72=>self.up(),
            75=>self.move_cursor(-1),
            77=>self.move_cursor(1),
            80=>self.down(),
            _=>if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                if let Some(key) = keyboard.process_keyevent(key_event) {
                    match key {
                        DecodedKey::Unicode(character) => self.unicode(character),
                        DecodedKey::RawKey(key) => self.raw_key(key),
                    }
                }
            }
        }
    }

    fn unicode(&self, character: char) {
        if self.mode == 0 {
            add_command_buffer!(character);
            print!("{}", character);
        }
        else if self.mode == 1 {
            print!("{}", character);
        }
        else if self.mode == 2 {
            // These are the tetris control keys, it's easier to pass them as integers though this solution isn't really efficent or extensible.
            if character == 'a' {
                TETRIS.lock().set(7)
            }
            else if character == 'c' {
                TETRIS.lock().set(8)
            }
            else if character == 'z' {
                TETRIS.lock().set(5)
            }
            else if character == ' ' {
                TETRIS.lock().set(4)                
            }
            else if character == 'p' {
                TETRIS.lock().set(9)
            }
        }
    }

    fn raw_key(&self, code: KeyCode) {
        if self.mode == 1 || self.mode == 0 {
            print!("{:?}", code);
        }
    }

    // You though it was move_cursor, but it was I, arrow key
    fn move_cursor(&self, dist: isize) {
        if self.mode == 0 || self.mode == 1 {
            if dist > 0 {
                right(dist as usize);
            }
            else {
                left(dist as usize);
            }
        }
        else if self.mode == 2 {
            if dist > 0 {
                TETRIS.lock().set(2)
            }
            else {
                TETRIS.lock().set(1)
            }
        }
    }

    fn down(&self) {
        if self.mode == 2 {
            TETRIS.lock().set(3)
        }
    }

    fn up(&self) {
        if self.mode == 2 {
            TETRIS.lock().set(6)
        }
    } 

    fn esc(&mut self) {
        if self.mode == 2 {
            TETRIS.lock().set(9)
        }    
    }
}

lazy_static! {
    pub static ref KEYBOARD_ROUTER: Mutex<KeyboardRouter> = {
        Mutex::new(KeyboardRouter::new())
    };
}

 
pub fn left(_dist:usize){
    if MODE.lock().text {
        WRITER.lock().move_cursor_left(1);
    }
    else {
        ADVANCED_WRITER.lock().move_cursor_left(1);
    }
}

pub fn right(_dist:usize){
    if MODE.lock().text {
        WRITER.lock().move_cursor_right(1);
    }
    else {
        ADVANCED_WRITER.lock().move_cursor_right(1);
    }
}