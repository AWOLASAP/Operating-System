use crate::vga_buffer::ADVANCED_WRITER;
use vga::colors::Color16;
use alloc::collections::vec_deque::VecDeque;
use crate::rng::RNGSEED;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::keyboard_routing::KEYBOARD_ROUTER;
use crate::timer_routing::TIME_ROUTER;
use x86_64::instructions::interrupts;
use rand_pcg::Lcg128Xsl64;
use rand_core::{SeedableRng,RngCore};
use alloc::string::String;
use alloc::string::ToString;

//Generic game constant(s)
const BLOCK_SIZE: usize = 19;

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

const UNPIECE: Piece = Piece {
    rotations: [0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF],
    color: Color16::Black,
};

const PIECES: [Piece; 7] = [I, J, L, O, S, T, Z];

const LEVEL_TIMES: [usize; 28] = [5, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2];
const MAX_LEVEL_SPEED: usize = 1;

const SCORE_LINE_TABLE: [usize; 5] = [0, 40, 100, 300, 1200];
const SCORE_MULTIPLIER: [usize; 28] = [1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4];
const SCORE_MAX_LEVEL: usize = 5;

const LINES_PER_LEVEL: usize = 10;

struct RenderPiece {
    piece: Piece,
    held: bool,
    position: u8,
    x: isize,
    y: isize,
}
pub struct Tetris {
    pub key: u8,
    board: [[Color16; 10]; 28],
    rendered_board: [[Color16; 10]; 28],
    old_rendered_board: [[Color16; 10]; 28],
    old_held: [[Color16; 4]; 4],
    old_next: [[Color16; 4]; 4],
    bag: VecDeque<u8>,
    piece_falling: bool,
    run: bool,
    score: usize,
    current_piece: RenderPiece,
    move_timer: usize,
    held_piece: Piece,
    next_piece: Piece,
    lines_cleared_in_level: usize,
    level: usize,
}

impl Tetris {
    // Makes the struct, but doesn't really initialize it - you need to do that with Tetris::init();
    fn new() -> Tetris {
        Tetris {
            key: 0,
            board: [[Color16::Black; 10]; 28],
            rendered_board: [[Color16::Black; 10]; 28],
            old_rendered_board: [[Color16::DarkGrey; 10]; 28],
            old_held: [[Color16::Black; 4]; 4],
            old_next: [[Color16::Black; 4]; 4],
            bag:  VecDeque::with_capacity(14),
            piece_falling: false,
            run: true,
            score: 0,
            current_piece: RenderPiece {
                piece: UNPIECE,
                held: false,
                position: 0,
                x: 0,
                y: 0,
            },
            move_timer: 6,
            held_piece: UNPIECE,
            next_piece: UNPIECE,
            lines_cleared_in_level: 0,
            level: 0,
         }
    }

    //Not only sets/resets every relevant variable, but also
    pub fn init(&mut self) {
        self.board = [[Color16::Black; 10]; 28];
        self.rendered_board = [[Color16::Black; 10]; 28];
        self.old_rendered_board = [[Color16::DarkGrey; 10]; 28];
        self.old_held = [[Color16::Black; 4]; 4];
        self.old_next = [[Color16::Black; 4]; 4];
        self.bag = VecDeque::with_capacity(14);
        self.piece_falling = false;
        self.run = true;
        self.score = 0;
        self.lines_cleared_in_level = 0;
        self.level = 0;
        self.current_piece = RenderPiece {
            piece: UNPIECE,
            held: false,
            position: 0,
            x: 0,
            y: 0,
        };
        self.move_timer = 6;
        self.held_piece = UNPIECE;
        self.next_piece = UNPIECE;

        for i in 0..4 {
            for j in 0..10 {
                self.board[i + 24][j] = Color16::LightGrey;
            }
        }

        //Utility functions which 1. Clear the graphics screen, 2. redirect keyboard input to tetris, 3. redirect timing signals
        interrupts::without_interrupts(|| {
            print!("Tetris started");
            ADVANCED_WRITER.lock().wipe_buffer();
            unsafe {KEYBOARD_ROUTER.force_unlock()};
            KEYBOARD_ROUTER.lock().mode.tetris = true;
            KEYBOARD_ROUTER.lock().mode.terminal = false;
            TIME_ROUTER.lock().mode.tetris = true;
            TIME_ROUTER.lock().mode.terminal = false;
            //ADVANCED_WRITER.lock().disable_blink();
            for i in 0..24 {
                ADVANCED_WRITER.lock().draw_rect((420 as isize, (i * BLOCK_SIZE + i) as isize), (220 as isize, ((i + 1) * BLOCK_SIZE - 1 + i) as isize), Color16::DarkGrey);
            }
            for j in 0..10 {
                ADVANCED_WRITER.lock().draw_rect(((220 + j * BLOCK_SIZE + j - 1) as isize, 0 as isize), ((220 + (j + 1) * BLOCK_SIZE + j) as isize, 480 as isize), Color16::DarkGrey);
            }

        });


        self.gen_bag();
        let piece = self.bag.pop_front();
        let piece = match piece {
            Some(i) => i,
            None => 1,
        };
        self.next_piece = PIECES[piece as usize];

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
                if self.level > 28 {
                    self.move_timer = MAX_LEVEL_SPEED;
                }
                else {
                    self.move_timer = LEVEL_TIMES[self.level];
                }
                self.move_timer = 5;
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
                self.hold();
            }
            else if key == 9 {
                // This turns tetris off
                unsafe {TIME_ROUTER.force_unlock()};
                KEYBOARD_ROUTER.lock().mode.terminal = true;
                KEYBOARD_ROUTER.lock().mode.tetris = false;
                TIME_ROUTER.lock().mode.terminal = true;
                TIME_ROUTER.lock().mode.tetris = false;
                ADVANCED_WRITER.lock().wipe_buffer();
                println!();
                return;
            }

            if move_down {
                self.current_piece.y += 1;
                descended = true;
            }
            else {
                self.move_timer -= 1;
            }
            // Left and right bounds checking - makes it so that if a piece IS too far to one side, it won't be after this
            let deserialized_piece = self.deserialize_piece();
            #[allow(clippy::all)]
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
                //Prevents rotations which put blocks in other blocks
                let deserialized_piece = self.deserialize_piece();
                if descended {
                    self.current_piece.y -= 1;
                }
                #[allow(clippy::all)]
                'colcalc1: for row in 0..4 {
                    for col in 0..4 {
                        #[allow(clippy::all)]
                        if deserialized_piece[row][col] {
                            if self.board[(self.current_piece.y + row as isize) as usize][(self.current_piece.x + col as isize) as usize] != Color16::Black {
                                self.current_piece.position += rot_dir_inverse as u8;
                                break 'colcalc1;
                            }
                        }
                    }
                }
                if descended {
                    self.current_piece.y += 1;
                }
            }
            else if moved_dir != 0 {
                //Prevents illegal left/right movement
                let deserialized_piece = self.deserialize_piece();
                if descended {
                    self.current_piece.y -= 1;
                }
                #[allow(clippy::all)]
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
                //Runs if the piece goes down
                let deserialized_piece = self.deserialize_piece();
                'columncalc: loop {
                    for row in 0..4 {
                        for col in 0..4 {
                            #[allow(clippy::all)]
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
            //Renders the moving piece (not th  stationary stuff)
            self.rendered_board = [[Color16::Black; 10]; 28];
            #[allow(clippy::all)]
            for row_1 in 0..4 {
                for col_1 in 0..4 {
                    if deserialized_piece[row_1][col_1] {
                        self.rendered_board[(self.current_piece.y + row_1 as isize) as usize][(self.current_piece.x + col_1 as isize) as usize] = self.current_piece.piece.color;
                    }
                }
            }

        }
        else {
            //Handles getting a new piece - includes support for next_piece
            self.current_piece = RenderPiece {
                piece: self.next_piece,
                held: false,
                position: 0,
                x: 3,
                y: 0,
            };
            let piece = self.bag.pop_front();
            let piece = match piece {
                Some(i) => i,
                None => 1,
            };
            self.next_piece = PIECES[piece as usize];

            self.piece_falling = true;
        }
        let mut lines_cleared = 0;
        for i in 4..24 {
            let mut line = true;
            for j in 0..10 {
                if self.board[i][j] == Color16::Black {
                    line = false;
                }
            }
            if line {
                lines_cleared += 1;
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
        if self.lines_cleared_in_level + lines_cleared > LINES_PER_LEVEL {
            self.lines_cleared_in_level += lines_cleared;
            self.lines_cleared_in_level -= LINES_PER_LEVEL;
            self.level += 1;
        }
        else {
            self.lines_cleared_in_level += lines_cleared;
        }
        if self.level > 28 {
            self.score += SCORE_MAX_LEVEL * SCORE_LINE_TABLE[lines_cleared];
        }
        else {
            self.score += SCORE_MULTIPLIER[self.level] * SCORE_LINE_TABLE[lines_cleared];
        }
        for i in 0..4 {
            for j in 0..10 {
                if self.board[i][j] != Color16::Black {
                    unsafe {TIME_ROUTER.force_unlock()};
                    KEYBOARD_ROUTER.lock().mode.terminal = true;
                    KEYBOARD_ROUTER.lock().mode.tetris = false;
                    TIME_ROUTER.lock().mode.terminal = true;
                    TIME_ROUTER.lock().mode.tetris = false;
                    ADVANCED_WRITER.lock().wipe_buffer();
                    println!();
                    return;
                }
            }
        }
        self.render();

    }
    // Holds the piece, and, if there is a piece held, replaces the current piece
    fn hold(&mut self) {
        let piece = self.held_piece;
        if !self.current_piece.held {
            self.held_piece = self.current_piece.piece;
            if piece.color == Color16::Black {
                self.piece_falling = false;
            }
            else {
                self.current_piece = RenderPiece {
                    piece,
                    held: true,
                    position: 0,
                    x: 3,
                    y: 0,
                };
            }
        }

    }

    // Generates a random 14-bag of pieces
    fn gen_bag(&mut self) {
        let mut pieces = [0, 1, 2, 3, 4, 5, 6, 0, 1, 2, 3, 4, 5, 6];
        let mut rand_num = Lcg128Xsl64::seed_from_u64(RNGSEED.lock().get());
        for _i in 0..30 {
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

    //All the following functions turn a piece from its rotation to a [[bool; 4]; 4], where each true is a place where the piece has a block
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
            bit >>= 1;
            col += 1;
            if col == 4 {
                row += 1;
                col = 0;
            }
        }
        result
    }

    fn deserialize_held_piece(&mut self) -> [[bool; 4]; 4] {
        let mut result = [[false; 4]; 4];
        let rotation = self.held_piece.rotations[0];
        let mut bit: u16 = 0x8000;

        let mut row: usize = 0;
        let mut col: usize = 0;
        while bit > 0 {
            if (rotation & bit) > 0 {
                result[row][col] = true;
            }
            bit >>= 1;
            col += 1;
            if col == 4 {
                row += 1;
                col = 0;
            }
        }
        result
    }

    fn deserialize_next_piece(&mut self) -> [[bool; 4]; 4] {
        let mut result = [[false; 4]; 4];
        let rotation = self.next_piece.rotations[0];
        let mut bit: u16 = 0x8000;

        let mut row: usize = 0;
        let mut col: usize = 0;
        while bit > 0 {
            if (rotation & bit) > 0 {
                result[row][col] = true;
            }
            bit >>= 1;
            col += 1;
            if col == 4 {
                row += 1;
                col = 0;
            }
        }
        result
    }

    // Renders the pieces - composits the rendered board and then the stationary board, and only renders pixels that have changed.
    fn render(&mut self) {
        ADVANCED_WRITER.lock().clear_buffer();
        ADVANCED_WRITER.lock().draw_buffer();
        let mut composited_board: [[Color16; 10]; 28] = [[Color16::Black; 10]; 28];
        let mut composited_held: [[Color16; 4]; 4] = [[Color16::Black; 4]; 4];
        let mut composited_next: [[Color16; 4]; 4] = [[Color16::Black; 4]; 4];

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
                        advanced_writer.draw_rect(((220 + j * BLOCK_SIZE + j) as isize, (i * BLOCK_SIZE + i) as isize), ((220 + (j + 1) * BLOCK_SIZE + j) as isize, ((i + 1) * BLOCK_SIZE - 1 + i) as isize), composited_board[i + 4][j]);
                    }
                }
            }
            
        });
        self.old_rendered_board = composited_board;
        let held_piece = self.deserialize_held_piece();
        for i in 0..4 {
            for j in 0..4 {
                if held_piece[i][j] {
                    composited_held[i][j] = self.held_piece.color;
                }
            }
        }
        interrupts::without_interrupts(|| {
            #[allow(clippy::all)]
            for i in 0..4 {
                for j in 0..4 {
                    if self.old_held[i][j] != composited_held[i][i] {
                        ADVANCED_WRITER.lock().draw_rect(((70 + j * BLOCK_SIZE) as isize, (i * BLOCK_SIZE) as isize), ((70 + (j + 1) * BLOCK_SIZE) as isize, ((i + 1) * BLOCK_SIZE - 1) as isize), composited_held[i][j]);
                    }
                }
            }
        });
        self.old_held = composited_held;
        let next_piece = self.deserialize_next_piece();
        for i in 0..4 {
            for j in 0..4 {
                if next_piece[i][j] {
                    composited_next[i][j] = self.next_piece.color;
                }
            }
        }
        interrupts::without_interrupts(|| {
            #[allow(clippy::all)]
            for i in 0..4 {
                for j in 0..4 {
                    if self.old_next[i][j] != composited_next[i][i] {
                        ADVANCED_WRITER.lock().draw_rect(((490 + j * BLOCK_SIZE) as isize, (i * BLOCK_SIZE) as isize), ((490 + (j + 1) * BLOCK_SIZE) as isize, ((i + 1) * BLOCK_SIZE - 1) as isize), composited_next[i][j]);
                    }
                }
            }
        });
        self.old_next = composited_next;
        for (i, c) in String::from("Score:").as_bytes().iter().enumerate() {
            ADVANCED_WRITER.lock().draw_char_with_scaling(432 + i * 16, 124, 2, *c as char, Color16::White, Color16::Black);
        }
        for (i, c) in self.score.to_string().as_bytes().iter().enumerate() {
            ADVANCED_WRITER.lock().draw_char_with_scaling(432 + i * 16, 140, 2, *c as char, Color16::White, Color16::Black);
        }
        for (i, c) in String::from("Level:").as_bytes().iter().enumerate() {
            ADVANCED_WRITER.lock().draw_char_with_scaling(432 + i * 16, 156, 2, *c as char, Color16::White, Color16::Black);
        }
        for (i, c) in (self.level + 1).to_string().as_bytes().iter().enumerate() {
            ADVANCED_WRITER.lock().draw_char_with_scaling(432 + i * 16, 172, 2, *c as char, Color16::White, Color16::Black);
        }
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
