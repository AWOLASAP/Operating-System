use crate::vga_buffer::{MODE, WRITER, ADVANCED_WRITER, PrintWriter};
use vga::colors::Color16;
use alloc::collections::vec_deque::VecDeque;
use crate::rng::RNGSEED;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::keyboard_routing::KEYBOARD_ROUTER;
use crate::timer_routing::TIME_ROUTER;
use x86_64::instructions::interrupts;
use rand_pcg::Lcg128Xsl64;
use rand_core::SeedableRng;
use rand_core::{RngCore};

//Generic game consts
const BLOCK_SIZE: usize = 20;

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
    x: isize,
    y: isize,
}
pub struct Tetris {
    pub key: u8,
    board: [[Color16; 10]; 28],
    rendered_board: [[Color16; 10]; 28],
    old_rendered_board: [[Color16; 10]; 28],
    bag: VecDeque<u8>,
    piece_falling: bool,
    run: bool,
    score: u64,
    current_piece: RenderPiece,
    move_timer: usize,
}

impl Tetris {
    fn new() -> Tetris {
        Tetris { 
            key: 0,
            board: [[Color16::Black; 10]; 28],
            rendered_board: [[Color16::Black; 10]; 28],
            old_rendered_board: [[Color16::DarkGrey; 10]; 28],
            bag:  VecDeque::with_capacity(14),
            piece_falling: false,
            run: true,
            score: 0,
            current_piece: RenderPiece {
                piece: I,
                position: 0,
                x: 0,
                y: 0,
            },
            move_timer: 6,
         }
    }

    pub fn init(&mut self) {
        self.board = [[Color16::Black; 10]; 28];
        self.rendered_board = [[Color16::Black; 10]; 28];
        self.old_rendered_board = [[Color16::DarkGrey; 10]; 28];
        self.bag = VecDeque::with_capacity(14);
        self.piece_falling = false;
        self.run = true;
        self.score = 0;
        self.current_piece = RenderPiece {
            piece: I,
            position: 0,
            x: 0,
            y: 0,
        };
        self.move_timer = 6;
    
        for i in 0..4 {
            for j in 0..10 {
                self.board[i + 24][j] = Color16::LightGrey;
            }
        }

        unsafe {KEYBOARD_ROUTER.force_unlock()};
        KEYBOARD_ROUTER.lock().mode = 2;
        TIME_ROUTER.lock().mode = 1;
        //ADVANCED_WRITER.lock().disable_blink();
    }

    pub fn game_loop(&mut self) {
        if self.bag.is_empty() {
            self.gen_bag();
    
        }
        if self.piece_falling {
            let key = self.get();
            
            let mut try_until_fall = false;
            let mut rotated = false;
            let mut move_down = false;
            let mut moved_dir: isize = 0;
            let mut rot_dir_inverse: isize = 0;
            // Probably an extraneous variable but who cares :shrug:
            let mut descended = false;

            if self.move_timer == 0 {
                self.move_timer = 6;
                move_down = true;
            }
            if key == 1 {
                self.current_piece.x -= 1;
                moved_dir = -1;
            }
            else if key == 2 {
                self.current_piece.x += 1;
                moved_dir = 1;
            }
            else if key == 3 {
                self.current_piece.y += 1;
                if move_down {
                    move_down = false;
                }
                descended = true;
            }
            else if key == 4 {
                try_until_fall = true;
                descended = true;
            }
            else if key == 5 {
                self.current_piece.position += 3;
                rot_dir_inverse = 1;
                rotated = true;
            }
            else if key == 6 {
                self.current_piece.position += 1;
                rot_dir_inverse = 3;
                rotated = true;
            }
            else if key == 7 {
                self.current_piece.position += 2;
                rot_dir_inverse = 2;
                rotated = true;
            }
            else if key == 8 {
    
            }

            if move_down {
                self.current_piece.y += 1;
                descended = true;
            }
            else {
                self.move_timer -= 1;
            }
            // Left and right bounds checking
            let deserialized_piece = self.deserialize_piece();
            for row_1 in 0..4 {
                for col_1 in 0..4 {
                    if deserialized_piece[row_1][col_1] {
                        if self.current_piece.x + col_1 as isize > 9 {
                            self.current_piece.x += 9 - self.current_piece.x - col_1 as isize ; 

                        }
                        else if (self.current_piece.x + col_1 as isize) < 0 {
                            self.current_piece.x = 0 - col_1 as isize ; 
                        }
                    }
                }
            }
            if rotated {
                let deserialized_piece = self.deserialize_piece();
                if descended {
                    self.current_piece.y -= 1;
                }
                'colcalc: for row in 0..4 {
                    for col in 0..4 {
                        if deserialized_piece[row][col] {
                            if self.board[(self.current_piece.y + row as isize) as usize][(self.current_piece.x + col as isize) as usize] != Color16::Black {
                                self.current_piece.position += rot_dir_inverse as u8;
                                break 'colcalc;
                            }
                        }
                    }
                }
                if descended {
                    self.current_piece.y += 1;
                }
            }
            else if moved_dir != 0 {
                let deserialized_piece = self.deserialize_piece();
                if descended {
                    self.current_piece.y -= 1;
                }
                'colcalc: for row in 0..4 {
                    for col in 0..4 {
                        if deserialized_piece[row][col] {
                            if self.board[(self.current_piece.y + row as isize) as usize][(self.current_piece.x + col as isize) as usize] != Color16::Black {
                                self.current_piece.x -= moved_dir;
                                break 'colcalc;
                            }
                        }
                    }
                }
                if descended {
                    self.current_piece.y += 1;
                }

            }
            if descended {
                let deserialized_piece = self.deserialize_piece();
                'columncalc: loop {
                    for row in 0..4 {
                        for col in 0..4 {
                            if deserialized_piece[row][col] {
                                if self.board[(self.current_piece.y + row as isize) as usize][(self.current_piece.x + col as isize) as usize] != Color16::Black {
                                    self.current_piece.y -= 1;
                                    // Imprint stuff to the board
                                    for row_1 in 0..4 {
                                        for col_1 in 0..4 {
                                            if deserialized_piece[row_1][col_1] {
                                                self.board[(self.current_piece.y + row_1 as isize) as usize][(self.current_piece.x + col_1 as isize) as usize] = self.current_piece.piece.color;
                                            }
                                        }
                                    }
                                    self.piece_falling = false;
                                    break 'columncalc;
                                }
                            }
                        }
                    }
                    if !try_until_fall {
                        break 'columncalc;
                    }
                    else {
                        self.current_piece.y += 1;
                    }
                }

            }
            let deserialized_piece = self.deserialize_piece();

            self.rendered_board = [[Color16::Black; 10]; 28];
            for row_1 in 0..4 {
                for col_1 in 0..4 {
                    if deserialized_piece[row_1][col_1] {
                        self.rendered_board[(self.current_piece.y + row_1 as isize) as usize][(self.current_piece.x + col_1 as isize) as usize] = self.current_piece.piece.color;
                    }
                }
            }
    
        }
        else {
            let piece = self.bag.pop_front();
            let piece = match piece {
                Some(i) => i,
                None => 1,
            };
            let piece = PIECES[piece as usize];
            self.current_piece = RenderPiece {
                piece: piece,
                position: 0,
                x: 3,
                y: 0,
            };
            self.piece_falling = true;
        }
        for i in 4..24 {
            let mut line = true;
            for j in 0..10 {
                if self.board[i][j] == Color16::Black {
                    line = false;
                }
            }
            if line {
                for x in (4..i).rev() {
                    for y in 0..10 {
                        self.board[x + 1][y] = self.board[x][y];
                    }
                }
                for y in 0..10 {
                    self.board[4][y] = Color16::Black;
                }
            }
        }
        for i in 0..4 {
            for j in 0..10 {
                if self.board[i][j] != Color16::Black {
                    loop {
                        print!("DEAD");
                    }
                }
            }
        }
        self.render();
        
    }


    fn gen_bag(&mut self) {
        let mut pieces = [0, 1, 2, 3, 4, 5, 6, 0, 1, 2, 3, 4, 5, 6];
        let mut rand_num = Lcg128Xsl64::seed_from_u64(RNGSEED.lock().get());
        for i in 0..14 {
            let r1 = (rand_num.next_u64() % 14) as usize;
            let r2 = (rand_num.next_u64() % 14) as usize;
            let swap = pieces[r1];
            pieces[r1] = pieces[r2];
            pieces[r2] = swap;
        }
        rand_num.next_u32();
        for i in pieces.iter() {
            self.bag.push_back(*i as u8);
        }
    }
    
    fn deserialize_piece(&mut self) -> [[bool; 4]; 4] {
        let mut result = [[false; 4]; 4];
        let rotation = self.current_piece.piece.rotations[(self.current_piece.position % 4) as usize];
        let mut bit: u16 = 0x8000;
    
        let mut row: usize = 0;
        let mut col: usize = 0;
        while bit > 0 {
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
    
    fn render(&mut self) {
        let mut composited_board: [[Color16; 10]; 28] = [[Color16::Black; 10]; 28];
        for i in 0..24 {
            for j in 0..10 {
                if self.board[i + 4][j] != Color16::Black {
                    composited_board[i + 4][j] = self.board[i + 4][j];
                }
                if self.rendered_board[i + 4][j] != Color16::Black {
                    composited_board[i + 4][j] = self.rendered_board[i + 4][j];
                }
            }
        }
        interrupts::without_interrupts(|| {
            let advanced_writer = ADVANCED_WRITER.lock();
            for i in 0..24 {
                for j in 0..10 {
                    if composited_board[i + 4][j] != self.old_rendered_board[i + 4][j] {
                        advanced_writer.draw_rect(((220 + j * BLOCK_SIZE) as isize, (i * BLOCK_SIZE) as isize), ((220 + (j + 1) * BLOCK_SIZE) as isize, ((i + 1) * BLOCK_SIZE - 1) as isize), composited_board[i + 4][j]);
                    }
                }
            }
        });
        self.old_rendered_board = composited_board;   
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
    pub static ref TETRIS: Mutex<Tetris> = {
        Mutex::new(Tetris::new())
    };
}