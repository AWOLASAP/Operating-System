use lazy_static::lazy_static;
use spin::{Mutex};
use crate::vga_buffer::ADVANCED_WRITER;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use crate::ustar::USTARFS;
use crate::println;
use crate::keyboard_routing::KEYBOARD_ROUTER;
use x86_64::instructions::interrupts;
use crate::vga_buffer::PrintWriter;
use crate::alloc::string::ToString;
use crate::commands::COMMANDRUNNER;
use alloc::collections::vec_deque::VecDeque;

//

pub struct BrainF {
    instructions: Vec<u8>,
    instruction_pointer: usize,
    data: Vec<u8>,
    data_pointer: usize,
    input_buffer: VecDeque<u8>,
    jump_back_table: Vec<usize>,
}

impl Default for BrainF {
    fn default() -> BrainF {
        BrainF::new()
    }
}

impl BrainF {
    pub fn new() -> BrainF {
        BrainF{
            instructions: Vec::new(),
            instruction_pointer: 0,
            data: vec![0; 30000],
            data_pointer: 0,
            input_buffer: VecDeque::with_capacity(100),
            jump_back_table: Vec::new(),
        }
    }

    // Exits on
    pub fn bf_loop(&mut self) -> bool {
        loop {
            if let Some(instruction) = self.instructions.get(self.instruction_pointer) {
                let instruction = *instruction as char;
                //print!("[{} at {}]", instruction, self.instruction_pointer);
                if instruction == '+' {
                    self.inc_data_at(self.data_pointer);
                    self.instruction_pointer += 1;
                }
                else if instruction == '-' {
                    self.dec_data_at(self.data_pointer);
                    self.instruction_pointer += 1;
                }
                else if instruction == '<' {
                    if self.data_pointer != 0 {
                        self.data_pointer -= 1;
                    }
                    self.instruction_pointer += 1;
                }
                else if instruction == '>' {
                    self.data_pointer += 1;
                    self.instruction_pointer += 1;
                }
                else if instruction == '.' {
                    print!("{}", self.data_at(self.data_pointer) as char);
                    self.instruction_pointer += 1;
                }
                else if instruction == ',' {
                    if !self.input_buffer.is_empty() {
                        if let Some(input) = self.input_buffer.pop_front() {
                            self.set_data_at(self.data_pointer, input);
                        }
                        self.instruction_pointer += 1;
                    }
                    else {
                        return false;
                    }
                }
                else if instruction == '[' {
                    if self.data_at(self.data_pointer) == 0 {
                        //Jump forward
                        self.instruction_pointer += 1;
                        let mut level = 1;
                        while level != 0 {
                            if let Some(new_instruction) = self.instructions.get(self.instruction_pointer) {
                                let new_instruction = *new_instruction as char;
                                if new_instruction == '[' {
                                    level += 1;
                                }
                                else if new_instruction == ']' {
                                    level -= 1;
                                }
                            }
                            else {
                                println!("Brainf script errored at: {}", self.instruction_pointer);
                                return true;
                            }
                            self.instruction_pointer += 1;
                        }
                    }
                    else {
                        self.instruction_pointer += 1;
                        self.jump_back_table.push(self.instruction_pointer);
                    }
                }
                else if instruction == ']' {
                    if self.data_at(self.data_pointer) != 0 {
                        if let Some(inst_ptr) = self.jump_back_table.pop() {
                            self.jump_back_table.push(inst_ptr);
                            self.instruction_pointer = inst_ptr;
                        }
                        //Jump backward
                    }
                    else {
                        self.instruction_pointer += 1;
                        self.jump_back_table.pop();
                    }
                }
                else {
                    self.instruction_pointer += 1;
                }
            }
            else {
                return true;
            }
        }
    }

    pub fn handle_bf_loop(&mut self) {
        if self.bf_loop() {
            self.handle_esc();
        }
    }

    #[inline]
    pub fn init_to_index(&mut self, index: usize) {
        if self.data.capacity() <= index {
            println!("Reserved");
            println!("{}", index);
            self.data.reserve(index - self.data.capacity() + 1);
            println!("Successfully");
        }
        for _ in self.data.len()..self.data.capacity() {
            self.data.push(0);
        }
    }

    #[inline]
    pub fn data_at(&mut self, index: usize) -> u8 {
        self.init_to_index(index);
        self.data[index]
    }

    #[inline]
    pub fn set_data_at(&mut self, index: usize, data: u8) {
        self.init_to_index(index);
        self.data[index] = data;
    }

    #[inline]
    pub fn inc_data_at(&mut self, index: usize) {
        self.init_to_index(index);
        self.data[index] += 1;
    }

    #[inline]
    pub fn dec_data_at(&mut self, index: usize) {
        self.init_to_index(index);
        self.data[index] -= 1;
    }

    #[inline]
    pub fn handle_scancode(&mut self, scancode: char) {
        print!("{}", scancode);
        self.input_buffer.push_back(scancode as u8);
        self.handle_bf_loop();
    }

    pub fn init_keyboard(&mut self, file: String, id: Option<u64>) {
        if let Some(data) = USTARFS.lock().read_file(file.to_string(), id) {
            self.instructions = data;
            // Init the keyboard stuff
            interrupts::without_interrupts(|| {
                ADVANCED_WRITER.lock().wipe_buffer();
                unsafe {KEYBOARD_ROUTER.force_unlock()};
                KEYBOARD_ROUTER.lock().mode.brainf = true;
                KEYBOARD_ROUTER.lock().mode.bfesc = true;
                KEYBOARD_ROUTER.lock().mode.terminal = false;
                KEYBOARD_ROUTER.lock().mode.screenbuffer = false;
                ADVANCED_WRITER.lock().disable_blink();
            });
            self.instruction_pointer = 0;
            self.data_pointer = 0;
            self.jump_back_table = Vec::new();
            self.data = vec![0; 30000];
            self.handle_bf_loop();
        }
        else {
            println!("File doesn't exist");

        }

    }

    pub fn init_file(&mut self, file: String, file2: String, id: Option<u64>) {
        if let Some(data) = USTARFS.lock().read_file(file.to_string(), id) {
            if  let Some(input) = USTARFS.lock().read_file(file2.to_string(), id)  {
                self.instructions = data;
                self.input_buffer = VecDeque::from(input);
                // Init the keyboard stuff
                interrupts::without_interrupts(|| {
                    ADVANCED_WRITER.lock().wipe_buffer();
                    unsafe {KEYBOARD_ROUTER.force_unlock()};
                    KEYBOARD_ROUTER.lock().mode.bfesc = true;
                    KEYBOARD_ROUTER.lock().mode.terminal = false;
                    KEYBOARD_ROUTER.lock().mode.screenbuffer = false;
                    ADVANCED_WRITER.lock().disable_blink();
                });
            }
            else {
                println!("2nd file not found");
            }
        }
        else {
            println!("File doesn't exist");

        }
    }

    pub fn handle_esc(&mut self) {
        interrupts::without_interrupts(|| {
            unsafe {KEYBOARD_ROUTER.force_unlock()};
            unsafe {COMMANDRUNNER.force_unlock()};
            KEYBOARD_ROUTER.lock().mode.brainf = false;
            KEYBOARD_ROUTER.lock().mode.bfesc = false;
            KEYBOARD_ROUTER.lock().mode.terminal = true;
            ADVANCED_WRITER.lock().enable_blink();
            //print!("[user@rust {}]# ", USTARFS.lock().cwd(COMMANDRUNNER.lock().dir_id));
        });
    }

}

lazy_static! {
    pub static ref BRAINF: Mutex<BrainF> = {
        Mutex::new(BrainF::new())
    };
}
