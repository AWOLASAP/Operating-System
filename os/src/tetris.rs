use crate::vga_buffer::{MODE, WRITER, ADVANCED_WRITER, PrintWriter};
use vga::colors::Color16;
use alloc::collections::vec_deque::VecDeque;
use crate::rng::RNGSEED;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::keyboard_routing::KEYBOARD_ROUTER;
use x86_64::instructions::interrupts;

#[derive(Clone, Copy)]
struct Piece {
    rotations: [u16; 4],
    color: Color16,
}

const I: Piece = Piece {
    rotations:  [0x0F00, 0x2222, 0x00F0, 0x4444],
    color: Color16::Cyan,
};

const J: Piece = Piece {
    rotations:  [0x44C0, 0x8E00, 0x6440, 0x0E20],
    color: Color16::Blue,
};

const L: Piece = Piece {
    rotations:  [0x4460, 0x0E80, 0xC440, 0x2E00],
    color: Color16::Brown,
};

const O: Piece = Piece {
    rotations:  [0xCC00, 0xCC00, 0xCC00, 0xCC00],
    color: Color16::Yellow,
};

const S: Piece = Piece {
    rotations:  [0x06C0, 0x8C40, 0x6C00, 0x4620],
    color: Color16::Green,
};

const T: Piece = Piece {
    rotations:  [0x0E40, 0x4C40, 0x4E00, 0x4640],
    color: Color16::Magenta,
};

const Z: Piece = Piece {
    rotations:  [0x0C60, 0x4C80, 0xC600, 0x2640],
    color: Color16::Red,
};

const PIECES: [Piece; 7] = [I, J, L, O, S, T, Z]; 

struct RenderPiece {
    piece: Piece,
    position: u8, 
    x: usize,
    y: usize,
}

fn gen_bag(bag: &mut VecDeque<u8>) {
    let mut pieces = [0, 1, 2, 3, 4, 5, 6, 0, 1, 2, 3, 4, 5, 6];
    for i in 0..14 {
        interrupts::without_interrupts(|| {
            pieces[((1103515245u64 * RNGSEED.lock().get() + 12345u64) % 14u64) as usize] = pieces[((1103515245u64 * RNGSEED.lock().get() + 12345u64) % 14u64) as usize]
        });
    }
    for i in pieces.iter() {
        bag.push_back(*i as u8);
    }
}

fn deserialize_piece(piece: &RenderPiece) -> [[bool; 4]; 4] {
    let mut result = [[false; 4]; 4];
    let rotation = piece.piece.rotations[(piece.position % 4) as usize];
    let mut bit: u16 = 0x8000;

    let mut row: usize = 0;
    let mut col: usize = 0;
    while (bit > 0) {
        if (rotation & bit) > 0 {
            result[row][col] = true;
        }
        bit = bit >> 1;
        col += 1;
        if col == 4 {
            row += 1;
            col = 0;
        }
    }
    result
}

fn render(board: [[Color16; 10]; 24], old_board: [[Color16; 10]; 24], score: usize, bag: VecDeque<u8>) {

}


pub fn tetris() {
    // NOTE: last 4 in board is for piece spawning
    let mut board = [[Color16::Black; 10]; 24];
    let mut old_board = [[Color16::Black; 10]; 24];
    let mut bag: VecDeque<u8> = VecDeque::with_capacity(14);
    let mut piece_falling = false;
    let mut run = true;
    let mut current_piece = RenderPiece {
        piece: I,
        position: 0,
        x: 0,
        y: 0,
    };

    KEYBOARD_ROUTER.lock().mode = 2;

    while run {
        if bag.is_empty() {
            gen_bag(&mut bag);
        }
        if piece_falling {
            let key = TETRIS_KEY_HANDLER.lock().get();
            let mut rotated = false;
            if key == 1 {
                current_piece.x -= 1;
            }
            else if key == 2 {
                current_piece.x += 1;
            }
            else if key == 3 {
                current_piece.y += 1;
            }
            else if key == 4 {

            }
            else if key == 5 {

            }
            else if key == 6 {

            }
            else if key == 7 {

            }
            else if key == 8 {

            }
            if rotated {
                
            }
            else {
                let deserialized_piece = deserialize_piece(&current_piece);
                for row in 0..4 {
                    for col in 0..4 {

                    }
                }
            }

        }
        else {
            let piece = bag.pop_front();
            let piece = match piece {
                Some(i) => i,
                None => 1,
            };
            let piece = PIECES[piece as usize];
            current_piece = RenderPiece {
                piece: piece,
                position: 0,
                x: 3,
                y: 0,
            };
            piece_falling = true;
        }
        
    }
}

pub struct KeyboardInterface {
    pub key: u8,
}

impl KeyboardInterface {
    fn new() -> KeyboardInterface {
        KeyboardInterface { key: 0 }
    }

    pub fn set(&mut self, key: u8) {
        self.key = key;
    }

    pub fn get(&mut self) -> u8 {
        let k = self.key;
        self.key = 0;
        k
    }
}

lazy_static! {
    pub static ref TETRIS_KEY_HANDLER: Mutex<KeyboardInterface> = {
        Mutex::new(KeyboardInterface::new())
    };
}