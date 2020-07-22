#[allow(dead_code)]

use core::fmt;
use volatile::Volatile;
use spin::Mutex;
use lazy_static::lazy_static;
use vga::colors::Color16;
use vga::writers::{Graphics640x480x16, GraphicsWriter, Text80x25, TextWriter};
use vga::drawing::Point;
use core::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color16, background: Color16) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

impl ScreenChar {
    fn new() -> ScreenChar {
        ScreenChar {
            ascii_character: 0,
            color_code: ColorCode::new(Color16::Black, Color16::Black),
        }
    }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT_ADVANCED: usize = 60;
const BUFFER_WIDTH_ADVANCED: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],}

#[derive(Copy, Clone)]
#[repr(transparent)]
struct AdvancedBuffer {
    chars: [[ScreenChar; BUFFER_WIDTH_ADVANCED]; BUFFER_HEIGHT_ADVANCED],
}

impl AdvancedBuffer {
    fn new() -> AdvancedBuffer {
        AdvancedBuffer{ chars: [[ScreenChar::new(); BUFFER_WIDTH_ADVANCED]; BUFFER_HEIGHT_ADVANCED] }
    }
}

pub struct AdvancedWriter {
    column_position: usize,
    color_code: ColorCode,
    buffer: AdvancedBuffer,
    old_buffer: AdvancedBuffer,
    mode: Graphics640x480x16,
    back_color: Color16, 
    front_color: Color16,
}

impl AdvancedWriter {
    fn new() -> AdvancedWriter {
        let mode = Graphics640x480x16::new();
        AdvancedWriter { 
            mode: mode,             
            column_position: 0,
            color_code: ColorCode::new(Color16::Yellow, Color16::Black),
            buffer: AdvancedBuffer::new(),
            old_buffer: AdvancedBuffer::new(),
            back_color: Color16::Black,
            front_color: Color16::Yellow,
        }
    }

    pub fn init(&mut self) {
        self.mode.set_mode();
        self.mode.clear_screen(Color16::Black);
    }

    pub fn set_color_code(&mut self, color_code: ColorCode) {
        self.color_code = color_code;
        let color = color_code.0;
        let front_color = Color16::try_from((color << 4) >> 4);
        let back_color = Color16::try_from(color >> 4);

        let front_color = match front_color {
            Ok(front_color) => front_color,
            Err(why) => panic!("{:?}", why),
        };

        let back_color = match back_color {
            Ok(back_color) => back_color,
            Err(why) => panic!("{:?}", why),
        };

        self.front_color = front_color;
        self.back_color = back_color;
    }

    pub fn set_front_color(&mut self, color: Color16) {
        self.set_color_code(ColorCode::new(color, self.back_color));
    }

    pub fn set_back_color(&mut self, color: Color16) {
        self.set_color_code(ColorCode::new(self.front_color, color));
    }

    

    // For use with the writer - draws characters
    // If this stops working, try replacing &self with &mut self
    fn draw_character(&self, x: usize, y: usize, character: ScreenChar) {
        let color = character.color_code.0;
        let ascii_character = character.ascii_character;
        let front_color = Color16::try_from((color << 4) >> 4);
        let back_color = Color16::try_from(color >> 4);

        let front_color = match front_color {
            Ok(front_color) => front_color,
            Err(why) => panic!("{:?}", why),
        };

        let back_color = match back_color {
            Ok(back_color) => back_color,
            Err(why) => panic!("{:?}", why),
        };


        self.mode.draw_character(x, y, ascii_character as char, front_color, back_color);
    }

    pub fn draw_char(&self, x: usize, y: usize, character: char, front_color: Color16, back_color: Color16) {
        self.mode.set_write_mode_2();
        self.mode.draw_character(x, y, character, front_color, back_color);
    }

    pub fn clear_screen(&self, color: Color16) {
        self.mode.clear_screen(color);
    }

    pub fn draw_line(&self, start: Point<isize>, end: Point<isize>, color: Color16) {
        self.mode.draw_line(start, end, color);
    }

    pub fn set_pixel(&self, x: usize, y: usize, color: Color16) {
        self.mode.set_write_mode_2();
        self.mode.set_pixel(x, y, color);
    }

    pub fn draw_buffer(&mut self) {
        // This also sets write mode 2
        //self.clear_screen(Color16::Black);
        for (index1, (row_new, row_old)) in self.buffer.chars.iter().zip(self.old_buffer.chars.iter()).enumerate() {
            for (index2, (character_new, character_old)) in row_new.iter().zip(row_old.iter()).enumerate() {
                if character_new.ascii_character != 0 && character_new.ascii_character != character_old.ascii_character{
                    self.draw_character(index2 * 8, index1 * 8, *character_new)
                }
            }
        }
        self.old_buffer = self.buffer;
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH_ADVANCED {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT_ADVANCED - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code: color_code,
                };
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT_ADVANCED {
            for col in 0..BUFFER_WIDTH_ADVANCED {
                let character = self.buffer.chars[row][col];
                self.buffer.chars[row - 1][col] = character;
            }
        }
        self.clear_row(BUFFER_HEIGHT_ADVANCED - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH_ADVANCED {
            self.buffer.chars[row][col] = blank;
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }
}

impl fmt::Write for AdvancedWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
lazy_static! {
    pub static ref ADVANCED_WRITER: Mutex<AdvancedWriter> = Mutex::new(AdvancedWriter::new());
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
    mode: Text80x25,
    back_color: Color16, 
    front_color: Color16,
}

impl Writer {
    fn new() -> Writer {
        let mode = Text80x25::new();
        Writer {
            column_position: 0,
            color_code: ColorCode::new(Color16::Yellow, Color16::Black),
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
            mode: mode,
            back_color: Color16::Black,
            front_color: Color16::Yellow,
        }
    }

    pub fn init(&mut self) {
        self.mode.set_mode();
    }

    pub fn set_color_code(&mut self, color_code: ColorCode) {
        self.color_code = color_code;
        let color = color_code.0;
        let front_color = Color16::try_from((color << 4) >> 4);
        let back_color = Color16::try_from(color >> 4);

        let front_color = match front_color {
            Ok(front_color) => front_color,
            Err(why) => panic!("{:?}", why),
        };

        let back_color = match back_color {
            Ok(back_color) => back_color,
            Err(why) => panic!("{:?}", why),
        };

        self.front_color = front_color;
        self.back_color = back_color;
    }

    pub fn set_front_color(&mut self, color: Color16) {
        self.set_color_code(ColorCode::new(color, self.back_color));
    }

    pub fn set_back_color(&mut self, color: Color16) {
        self.set_color_code(ColorCode::new(self.front_color, color));
    }
    
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // backspace
                0x08 => self.backspace(),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn backspace(&mut self) {
        if self.column_position != 0 {
            self.column_position -= 1;
            self.write_byte(b' ');
            self.column_position -= 1;
        }
    }
}


impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = {
        Mutex::new(Writer::new())
    };
}

pub struct ModeController {
    text: bool,
}

impl ModeController {
    fn new() -> ModeController {
        ModeController { text: true }
    }

    pub fn init(&mut self) {
        if self.text {
            self.text_init();
        }
        else {
            self.graphics_init();
        }
    }

    pub fn text_init(&mut self) {
        self.text = true;
        WRITER.lock().init();
    }

    pub fn graphics_init(&mut self) {
        self.text = false;
        ADVANCED_WRITER.lock().init();
    }
}

lazy_static! {
    pub static ref MODE: Mutex<ModeController> = {
        Mutex::new(ModeController::new())
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    if MODE.lock().text == true {
        WRITER.lock().write_fmt(args).unwrap();
    }
    else {
        ADVANCED_WRITER.lock().write_fmt(args).unwrap();
    }
    //ADVANCED_WRITER.lock().draw_buffer(WRITER.lock().buffer);
}
