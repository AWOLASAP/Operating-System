#![macro_use]

use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use vga::colors::Color16;
use vga::writers::{Graphics640x480x16, GraphicsWriter, Text80x25, TextWriter};
use vga::drawing::Point;
use core::convert::TryFrom;
use core::cmp::{min, max};
use x86_64::instructions::interrupts;
use x86::io::outb;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color16, background: Color16) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }

    fn decompose(&self) -> (Color16, Color16) {
        let color = self.0;
        let foreground = Color16::try_from((color << 4) >> 4);
        let background = Color16::try_from(color >> 4);

        let foreground = match foreground {
            Ok(foreground) => foreground,
            Err(why) => panic!("{:?}", why),
        };

        let background = match background {
            Ok(background) => background,
            Err(why) => panic!("{:?}", why),
        };

        (foreground, background)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}
// Character in a buffer
impl ScreenChar {
    fn new() -> ScreenChar {
        ScreenChar {
            ascii_character: 0,
            color_code: ColorCode::new(Color16::Black, Color16::Black),
        }
    }
    fn set_color(&mut self,color:ColorCode){
        self.color_code = color;
    }
}

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub const BUFFER_HEIGHT_ADVANCED: usize = 60;
const BUFFER_WIDTH_ADVANCED: usize = 80;

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

// If we had five weeks or something, this could be useful in rendering custom terminals
pub trait PrintWriter {
    // Getters

    fn get_width(&self) -> usize;

    fn get_height(&self) -> usize;

    // Getters and setters

    fn get_front_color(&self) -> Color16;
    fn set_front_color_attr(&mut self, color: Color16);

    fn get_back_color(&self) -> Color16;
    fn set_back_color_attr(&mut self, color: Color16);

    fn get_color_code(&self) -> ColorCode;
    fn set_color_code_attr(&mut self, color_code: ColorCode);

    fn get_blinked(&self) -> bool;
    fn set_blinked(&mut self, blinked: bool);

    fn get_blink_on(&self) -> bool;
    fn set_blink_on(&mut self, blink_on: bool);

    fn get_blink_color(&self) -> Color16;
    fn set_blink_color(&mut self, blink_color: Color16);

    fn get_column_position(&self) -> usize;
    fn set_column_position(&mut self, column_position: usize);

    // Color functionality
    fn set_color_code(&mut self, color_code: ColorCode) {
        self.set_color_code_attr(color_code);

        let (front_color, back_color) = color_code.decompose();

        self.set_front_color_attr(front_color);
        self.set_back_color_attr(back_color);
    }

    fn set_front_color(&mut self, color: Color16) {
        self.set_color_code(ColorCode::new(color, self.get_back_color()));
    }

    fn set_back_color(&mut self, color: Color16) {
        self.set_color_code(ColorCode::new(self.get_front_color(), color));
    }

    // Blink stuff
    fn enable_blink(&mut self) {
        self.set_blink_on(true);
    }

    fn disable_blink(&mut self) {
        self.set_blink_on(false);
    }

    //rerenders all characters on screen
    fn rerender_screen(&mut self){
        for row in 0..self.get_height() {
            for col in 0..self.get_width() {
                let mut character = self.read_buffer(row, col);
                character.set_color(ColorCode::new(self.get_front_color(), self.get_back_color()));
                self.write_buffer(row, col, character);
            }
        }
    }

    // If cursor has blinked, then un-blink it, otherwise blink it
    fn blink(&mut self) {
        if self.get_blink_on() {
            let character = self.read_buffer(self.get_height() - 1, self.get_column_position());
            let ascii_character = character.ascii_character;

            let (front_color, back_color) = character.color_code.decompose();

            if self.get_blinked() {
                self.write_buffer(self.get_height() - 1,
                self.get_column_position(),
                ScreenChar {ascii_character, color_code: ColorCode::new(
                    front_color,
                    self.get_blink_color()
                )});
            }
            else {
                self.write_buffer(self.get_height() - 1,
                self.get_column_position(),
                ScreenChar {ascii_character, color_code: ColorCode::new(
                    front_color,
                    front_color
                )});
                self.set_blink_color(back_color);
            }
            self.set_blinked(!self.get_blinked());
        }
    }

    // Cursor stuff
    fn move_cursor_left(&mut self, dist: usize) {
        if self.get_blinked() {
            self.blink();
        }
        if self.get_column_position() < dist {
            self.set_column_position(0);
        }
        else {
            self.set_column_position(self.get_column_position() - dist)
        }
    }

    fn move_cursor_right(&mut self, dist: usize) {
        if self.get_blinked() {
            self.blink();
        }
        if self.get_column_position() + dist > self.get_width() - 1 {
           self.new_line();
           return
        }
        let null = ScreenChar{ascii_character: 0,color_code: self.get_color_code()};
        if self.read_buffer(self.get_height()-1,self.get_column_position()+dist)!=null{
            self.set_column_position(self.get_column_position() + dist)
        }
    }

    // Actual print stuff
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.get_column_position() >= self.get_width() {
                    self.new_line();
                }

                let row = self.get_height() - 1;
                let col = self.get_column_position();

                let color_code = self.get_color_code();
                self.write_buffer(row, col, ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                let null = ScreenChar{ascii_character: 0, color_code};
                if col < self.get_width()-1 &&self.read_buffer(row,col+1)==null{
                    self.write_buffer(row, col+1, ScreenChar {
                        ascii_character: b' ',
                        color_code,
                    });
                }
                self.move_cursor_right(1);
            }
        }
    }

    fn new_line(&mut self) {
        if self.get_blinked() {
            self.blink();
        }
        for row in 1..self.get_height() {
            for col in 0..self.get_width() {
                let character = self.read_buffer(row, col);

                self.write_buffer(row - 1, col, character);
            }
        }
        self.clear_row(self.get_height() - 1);
        self.move_cursor_left(self.get_width());
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: 0,
            color_code: self.get_color_code(),
        };
        for col in 0..self.get_width() {
            self.write_buffer(row, col, blank);
        }
    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Delete
                0x7f => self.delete(),
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
        if self.get_column_position() != 0 {
            self.move_cursor_left(1);
            self.write_byte(b' ');
            self.move_cursor_left(1);
        }
        for character in self.get_column_position()+1..self.get_width()-1{
            let temp = self.read_buffer(self.get_height()-1, character);
            self.write_buffer(self.get_height()-1, character-1, temp);
        }
    }

    fn delete(&mut self){
        if self.get_column_position() != self.get_width()-1{
            self.write_byte(b' ');
            self.move_cursor_left(1);
        }
        for character in self.get_column_position()+1..self.get_width()-1{
            let temp = self.read_buffer(self.get_height()-1, character);
            self.write_buffer(self.get_height()-1, character-1, temp);
        }
    }

    // Buffer write stuff
    fn write_buffer(&mut self, row: usize, col: usize, character: ScreenChar);

    fn read_buffer(&self, row: usize, col: usize) -> ScreenChar;

}

pub struct AdvancedWriter {
    blinked: bool,
    blink_on: bool,
    terminal_border: bool,
    mode: Graphics640x480x16,
    column_position: usize,
    buffer: AdvancedBuffer,
    old_buffer: AdvancedBuffer,
    color_code: ColorCode,
    back_color: Color16,
    front_color: Color16,
    blinked_color: Color16,

}

impl AdvancedWriter {
    fn new() -> AdvancedWriter {
        let mode = Graphics640x480x16::new();
        AdvancedWriter {
            mode,
            column_position: 0,
            color_code: ColorCode::new(Color16::Yellow, Color16::Black),
            buffer: AdvancedBuffer::new(),
            old_buffer: AdvancedBuffer::new(),
            back_color: Color16::Black,
            front_color: Color16::Yellow,
            blinked_color: Color16::Black,
            blinked: false,
            blink_on: true,
            terminal_border: false,
        }
    }

    pub fn init(&mut self) {
        self.mode.set_mode();
        self.wipe_buffer();
    }

    // Graphics specific stuff - don't move into trait
    // For use with the writer - draws characters
    // If this stops working, try replacing &self with &mut self
    pub fn draw_character(&self, x: usize, y: usize, character: ScreenChar) {
        let ascii_character = character.ascii_character;
        let (front_color, back_color) = character.color_code.decompose();


        self.mode.draw_character(x, y, ascii_character as char, front_color, back_color);
    }

    // This draws a character but assumes that you know which character was already drawn there - this is an optimization because it doesn't update already drawn pixels.
    pub fn draw_different_character(&self, x: usize, y: usize, old_character: ScreenChar, new_character: ScreenChar) {
        if new_character.color_code != old_character.color_code {
            self.draw_character(x, y, new_character);
        }
        else {
            let ascii_character = old_character.ascii_character;
            let (front_color, back_color) = new_character.color_code.decompose();
            let ascii_character_new = new_character.ascii_character;

            self.mode.draw_different_character(x, y, ascii_character as char, ascii_character_new as char, front_color, back_color);
        }
    }

    //Draws a character at specified coordinates - you need to specify both the background and foreground color
    pub fn draw_char(&self, x: usize, y: usize, character: char, front_color: Color16, back_color: Color16) {
        self.mode.set_write_mode_2();
        self.mode.draw_character(x, y, character, front_color, back_color);
    }

    //Draws a character at specified coordinates - you need to specify foreground color - it will not overwrite any other pixels
    pub fn draw_char_fast(&self, x: usize, y: usize, character: char, front_color: Color16) {
        self.mode.set_write_mode_2();
        self.mode.draw_character_fast(x, y, character, front_color);
    }

    pub fn clear_screen(&self, color: Color16) {
        self.mode.clear_screen(color);
    }

    pub fn draw_line(&self, start: Point<isize>, end: Point<isize>, color: Color16) {
        self.mode.draw_line(start, end, color);
    }

    pub fn draw_rect(&self, start: Point<isize>, end: Point<isize>, color: Color16) {
        let y1 = start.1;
        let y2 = end.1;
        let x1 = start.0;
        let x2 = end.0;

        for i in min(x1, x2)..max(x1, x2) {
            self.mode.draw_line((i, y1), (i, y2), color);
        }
    }

    pub fn draw_circle(&mut self, center: Point<isize>, radius: isize, color: Color16) {
        let mut x: isize = 0;
        let mut y = radius;
        let mut d : isize = 3 - 2 * radius;

        while y >= x {
            self.mode.draw_line((x + center.0, y + center.1), (x + center.0, -y + center.1), color);
            self.mode.draw_line((-x + center.0, y + center.1), (-x + center.0, -y + center.1), color);
            self.mode.draw_line((-y + center.0, x + center.1), (y + center.0, x + center.1), color);
            self.mode.draw_line((-y + center.0, -x + center.1), (y + center.0, -x + center.1), color);
            x += 1;
            if d > 0 {
                y -= 1;
                d = d + 4 * (x - y) + 10;
            }
            else {
                d = d + 4*x + 6;
            }
        }
    }

    // NOTE: draws this at the center - this is the official logo for our operating system.
    pub fn draw_logo(&mut self, x: isize, y: isize, scale: isize) {
        // 25
        self.draw_circle((x, y), 8 * scale as isize, Color16::Pink);
        self.draw_circle((x - 4*scale, y - 4*scale), scale as isize, Color16::LightCyan);
        self.draw_circle((x + 4*scale, y - 4*scale), scale as isize, Color16::LightCyan);
        self.draw_circle((x, y), 2 * scale as isize, Color16::LightGreen);

        for i in 0..scale {
            self.draw_line(
                (x - 5 * scale + i, y + 3 * scale),
                (x - scale + i, y + 6 * scale),
                Color16::LightRed);
            self.draw_line(
                (x + 5 * scale - i, y + 3 * scale),
                (x + scale - i, y + 6 * scale),
                Color16::LightRed);
        }
    }

    pub fn set_pixel(&self, x: usize, y: usize, color: Color16) {
        self.mode.set_write_mode_2();
        self.mode.set_pixel(x, y, color);
    }

    pub fn draw_buffer(&mut self) {
        // This also sets write mode 2
        //self.clear_screen(Color16::Black);
        self.mode.set_write_mode_2();
        let mut offset = 0;
        if self.terminal_border {
            offset = 1;
        }
        for (index1, (row_new, row_old)) in self.buffer.chars.iter().zip(self.old_buffer.chars.iter()).enumerate() {
            if index1 > self.get_height() {
                continue;
            }
            for (index2, (character_new, character_old)) in row_new.iter().zip(row_old.iter()).enumerate() {
                if index2 > self.get_width() {
                    continue;
                }
                if character_new.ascii_character != character_old.ascii_character || character_new.color_code != character_old.color_code {
                    self.draw_character((index2 + offset) * 8, (index1 + offset) * 8, *character_new)
                }
            }
        }
        self.old_buffer = self.buffer;
    }

    // For when you know what's on the pixels/don't want to erase everything drawn on
    pub fn clear_buffer(&mut self) {
        self.mode.set_write_mode_2();
        for i in 0..self.get_height() {
            for j in 0..self.get_width() {
                self.write_buffer(i, j, ScreenChar {
                    ascii_character: 0,
                    color_code: self.color_code,
                });
            }
        }
        self.draw_buffer();
    }

    pub fn wipe_buffer(&mut self) {
        self.clear_buffer();
        self.clear_screen(self.back_color);
    }

    // Terminal Border stuff
    pub fn enable_border(&mut self) {
        self.terminal_border = true;
    }

    pub fn disable_border(&mut self) {
        self.terminal_border = false;
    }
}

impl PrintWriter for AdvancedWriter {
    fn get_height(&self) -> usize {
        if self.terminal_border {
            BUFFER_HEIGHT_ADVANCED - 2
        }
        else {
            BUFFER_HEIGHT_ADVANCED
        }
    }

    fn get_width(&self) -> usize {
        if self.terminal_border {
            BUFFER_WIDTH_ADVANCED - 2
        }
        else {
            BUFFER_WIDTH_ADVANCED
        }
    }

    fn get_front_color(&self) -> Color16 {
        self.front_color
    }
    fn set_front_color_attr(&mut self, color: Color16) {
        self.front_color = color;
    }

    fn get_back_color(&self) -> Color16  {
        self.back_color
    }
    fn set_back_color_attr(&mut self, color: Color16) {
        self.back_color = color;

    }

    fn get_color_code(&self) -> ColorCode {
        ColorCode::new(self.front_color, self.back_color)
    }
    fn set_color_code_attr(&mut self, color_code: ColorCode) {
        self.color_code = color_code;
    }

    fn get_blinked(&self) -> bool {
        self.blinked
    }
    fn set_blinked(&mut self, blinked: bool) {
        self.blinked = blinked;
    }

    fn get_blink_on(&self) -> bool {
        self.blink_on
    }
    fn set_blink_on(&mut self, blink_on: bool) {
        self.blink_on = blink_on;
    }

    fn get_blink_color(&self) -> Color16 {
        self.blinked_color
    }
    fn set_blink_color(&mut self, blink_color: Color16) {
        self.blinked_color = blink_color;
    }

    fn get_column_position(&self) -> usize {
        self.column_position
    }
    fn set_column_position(&mut self, column_position: usize) {
        self.column_position = column_position;
    }

    fn write_buffer(&mut self, row: usize, col: usize, character: ScreenChar) {
        self.buffer.chars[row][col] = character;
    }

    fn read_buffer(&self, row: usize, col: usize) -> ScreenChar {
        self.buffer.chars[row][col]
    }
}

impl fmt::Write for AdvancedWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        self.draw_buffer();
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
    blinked_color: Color16,
    blink_on: bool,
    blinked: bool,
}

impl Writer {
    fn new() -> Writer {
        let mode = Text80x25::new();
        Writer {
            column_position: 0,
            color_code: ColorCode::new(Color16::Yellow, Color16::Black),
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
            mode,
            back_color: Color16::Black,
            front_color: Color16::Yellow,
            blinked_color: Color16::Black,
            blink_on: true,
            blinked: false,
        }
    }

    pub fn init(&mut self) {
        self.mode.set_mode();
        unsafe {
            outb(0x3D4, 0x0A);
            outb(0x3D5, 0x20);
        };
    }
}

impl PrintWriter for Writer {
    fn get_height(&self) -> usize {
        BUFFER_HEIGHT
    }

    fn get_width(&self) -> usize {
        BUFFER_WIDTH
    }

    fn get_front_color(&self) -> Color16 {
        self.front_color
    }
    fn set_front_color_attr(&mut self, color: Color16) {
        self.front_color = color;
    }

    fn get_back_color(&self) -> Color16  {
        self.back_color
    }
    fn set_back_color_attr(&mut self, color: Color16) {
        self.back_color = color;

    }

    fn get_color_code(&self) -> ColorCode {
        ColorCode::new(self.front_color, self.back_color)
    }
    fn set_color_code_attr(&mut self, color_code: ColorCode) {
        self.color_code = color_code;
    }

    fn get_blinked(&self) -> bool {
        self.blinked
    }
    fn set_blinked(&mut self, blinked: bool) {
        self.blinked = blinked;
    }

    fn get_blink_on(&self) -> bool {
        self.blink_on
    }
    fn set_blink_on(&mut self, blink_on: bool) {
        self.blink_on = blink_on;
    }

    fn get_blink_color(&self) -> Color16 {
        self.blinked_color
    }
    fn set_blink_color(&mut self, blink_color: Color16) {
        self.blinked_color = blink_color;
    }

    fn get_column_position(&self) -> usize {
        self.column_position
    }
    fn set_column_position(&mut self, column_position: usize) {
        self.column_position = column_position;
    }

    fn write_buffer(&mut self, row: usize, col: usize, character: ScreenChar) {
        self.buffer.chars[row][col].write(character);
    }

    fn read_buffer(&self, row: usize, col: usize) -> ScreenChar {
        self.buffer.chars[row][col].read()
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
    pub text: bool,
    blink_timer: usize,
}

impl ModeController {
    fn new() -> ModeController {
        ModeController { text: true, blink_timer: 0}
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

        if !self.text {
            self.text = true;
            interrupts::without_interrupts(|| {
                WRITER.lock().init();
            });
        }
    }

    pub fn graphics_init(&mut self) {
        if self.text {
            self.text = false;
            interrupts::without_interrupts(|| {
                ADVANCED_WRITER.lock().init();
            });
        }
    }

    pub fn blink_current(&mut self) {
        if  self.blink_timer == 0 {
            if self.text {
                WRITER.lock().blink();
            }
            else {
                ADVANCED_WRITER.lock().blink();
                ADVANCED_WRITER.lock().draw_buffer();
            }
            self.blink_timer = 4;
        }
        else {
            self.blink_timer -= 1;
        }

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

     interrupts::without_interrupts(|| {
       if MODE.lock().text {
           WRITER.lock().write_fmt(args).unwrap();
       }
       else {
           ADVANCED_WRITER.lock().write_fmt(args).unwrap();
       }
     });
 }

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}
