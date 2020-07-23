use os::vga_buffer::{MODE, WRITER, ADVANCED_WRITER, PrintWriter};
use vga::colors::Color16;
use alloc::collections::vec_deque::VecDeque;
use crate::rng::RNGSEED;

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
    x: usize,
    y: usize,
}

fn gen_bag(bag: &mut VecDeque<u8>) {
    let mut pieces = [0, 1, 2, 3, 4, 5, 6, 0, 1, 2, 3, 4, 5, 6];
    for i in 0..14 {
        interrupts::without_interrupts(|| {
            pieces[(1103515245 * RNGSEED.lock().get() + 12345) % 14] = pieces[(1103515245 * RNGSEED.lock().get() + 12345) % 14]
        });
    }
    for i in pieces.iter() {
        VecDeque.push(i as u8);
    }
}

pub fn tetris() {
    let mut board = [[Color16::Black; 10]; 20];
    let mut old_board = [[Color16::Black; 10]; 20];
    let mut bag: VecDeque<u8> = VecDeque::with_capacity(14);
    let mut piece_falling = false;

    while true {
        if bag.is_empty() {
            gen_bag(bag);
        }
        if piece_falling {
            
        }
        else {
            let piece = bag.pop();
        }
        
    }
}