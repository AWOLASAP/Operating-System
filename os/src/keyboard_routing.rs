use lazy_static::lazy_static;
use crate::vga_buffer::{MODE, WRITER, ADVANCED_WRITER, PrintWriter};
use pc_keyboard::{layouts, DecodedKey, Keyboard, ScancodeSet1, KeyCode};
use spin::Mutex;
use crate::{move_command_cursor, end_tet_ost};
use crate::tetris::TETRIS;
use crate::vi::FAKE_VIM;
use crate::brainf::BRAINF;
use crate::commands::COMMANDRUNNER;

/* MODES
0 - Terminal + sends weird stuff to screenbuffer
1 - Screenbuffer only
2 - Tetris - only sends tetris keybinds
3 - Song - for quiting early
*/

pub struct Modes {
    pub terminal: bool,
    pub screenbuffer: bool,
    pub tetris: bool,
    pub tetris_score: bool,
    pub song: bool,
    pub textedit: bool,
    pub brainf: bool,
    pub bfesc: bool,
}

impl Modes {
    fn new() -> Modes {
        Modes {
            terminal: true,
            screenbuffer: false,
            tetris: false,
            tetris_score: false,
            song: false,
            textedit: false,
            brainf: false,
            bfesc: false,
        }
    }
}

pub struct KeyboardRouter {
    pub mode: Modes,
}

impl KeyboardRouter {
    fn new() -> KeyboardRouter {
        KeyboardRouter { mode: Modes::new() }
    }

    pub fn handle_scancode(&mut self, scancode: u8, keyboard: &mut Keyboard<layouts::Us104Key,ScancodeSet1>) {
        match scancode{
            // We need to find the right scancode for this (escape)
            129=>self.esc(),
            // Do nothing here - this is escape key down
            1=>(),
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
        if self.mode.terminal {
            print!("{}", character);
            if !COMMANDRUNNER.lock().add_to_buffer(character) {
                return;
            }
        }
        else if self.mode.screenbuffer {
            print!("{}", character);
        }
        if self.mode.tetris {
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
        if self.mode.song && character == 'q' {
            end_tet_ost!();
        }
        if self.mode.textedit {
            FAKE_VIM.lock().handle_scancode(character);
        }
        if self.mode.brainf {
            BRAINF.lock().handle_scancode(character);
        }
        if self.mode.tetris_score {
            TETRIS.lock().handle_scancode(character);
        }
    }

    fn raw_key(&self, code: KeyCode) {
        if self.mode.terminal || self.mode.screenbuffer {
            print!("{:?}", code);
        }
    }

    // You though it was move_cursor, but it was I, arrow key
    fn move_cursor(&self, dist: i8) {
        if self.mode.terminal || self.mode.screenbuffer {
            if dist > 0 {
                right();
            }
            else {
                left();
            }
        }
        else if self.mode.tetris {
            if dist > 0 {
                TETRIS.lock().set(2)
            }
            else {
                TETRIS.lock().set(1)
            }
        }
        if self.mode.textedit {
            if dist > 0 {
                FAKE_VIM.lock().right();
            }
            else {
                FAKE_VIM.lock().left();
            }
        }
    }

    fn down(&self) {
        if self.mode.tetris {
            TETRIS.lock().set(3)
        }
        if self.mode.textedit {
            FAKE_VIM.lock().down();
        }
    }

    fn up(&self) {
        if self.mode.tetris {
            TETRIS.lock().set(6)
        }
        if self.mode.textedit {
            FAKE_VIM.lock().up();
        }
    }

    fn esc(&mut self) {
        if self.mode.tetris || self.mode.tetris_score {
            TETRIS.lock().set(9)
        }
        if self.mode.textedit {
            FAKE_VIM.lock().handle_esc();
        }
        if self.mode.bfesc {
            BRAINF.lock().handle_esc();
        }
    }
}

lazy_static! {
    pub static ref KEYBOARD_ROUTER: Mutex<KeyboardRouter> = {
        Mutex::new(KeyboardRouter::new())
    };
}


pub fn left(){
    if MODE.lock().text {
        WRITER.lock().move_cursor_left(1);
    }
    else {
        ADVANCED_WRITER.lock().move_cursor_left(1);
    }
    move_command_cursor!(-1);
}

pub fn right(){
    if MODE.lock().text {
        WRITER.lock().move_cursor_right(1);
    }
    else {
        ADVANCED_WRITER.lock().move_cursor_right(1);
    }
    move_command_cursor!(1)
}
