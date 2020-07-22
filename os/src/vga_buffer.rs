#[allow(dead_code)]

use core::fmt;
use volatile::Volatile;
use spin::Mutex;
use lazy_static::lazy_static;
use vga::colors::Color16;
use vga::writers::{Graphics640x480x16, GraphicsWriter};
use vga::drawing::Point;
use num_enum::TryFromPrimitive;
use core::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

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

const BUFFER_HEIGHT: usize = 60;
const BUFFER_WIDTH: usize = 80;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl Buffer {
    fn new() -> Buffer {
        Buffer{ chars: [[ScreenChar::new(); BUFFER_WIDTH]; BUFFER_HEIGHT] }
    }
}

pub struct AdvancedWriter {
    mode: Graphics640x480x16,
}

impl AdvancedWriter {
    fn new() -> AdvancedWriter {
        let mode = Graphics640x480x16::new();
        mode.set_mode();
        mode.clear_screen(Color16::Black);
        AdvancedWriter{ mode: mode }
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

    pub fn draw_buffer(&self, buffer: Buffer) {
        // This also sets write mode 2
        //self.clear_screen(Color16::Black);
        for (index1, row) in (buffer).chars.iter().enumerate() {
            for (index2, character) in row.iter().enumerate() {
                if (character.ascii_character != 0) {
                    self.draw_character(index2 * 8, index1 * 8, *character)
                }
            }
        }
    }
}


lazy_static! {
    pub static ref ADVANCED_WRITER: Mutex<AdvancedWriter> = Mutex::new(AdvancedWriter::new());
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    pub buffer: Buffer,
}

impl Writer {
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
                self.buffer.chars[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code: color_code,
                };
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col];
                self.buffer.chars[row - 1][col] = character;
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


impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = {
        Mutex::new(Writer {
            column_position: 0,
            color_code: ColorCode::new(Color16::Yellow, Color16::Black),
            buffer: Buffer::new(),
        })
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
    WRITER.lock().write_fmt(args).unwrap();
    //ADVANCED_WRITER.lock().draw_buffer(WRITER.lock().buffer);
}
