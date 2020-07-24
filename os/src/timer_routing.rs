use lazy_static::lazy_static;
use crate::vga_buffer::{MODE};
use spin::Mutex;
use crate::tetris::TETRIS;
use crate::rng::RNGSEED;

/* MODES
0 - Terminal + RNG
1 - Tetris + RNG
*/
pub struct TimeRouter {
    pub mode: usize,
}

impl TimeRouter {
    fn new() -> TimeRouter {
        TimeRouter { mode: 0 }
    }

    pub fn handle(&mut self) {
        if self.mode == 0 {
            MODE.lock().blink_current();
            RNGSEED.lock().inc();
        }
        else if self.mode == 1 {
            TETRIS.lock().game_loop();
            RNGSEED.lock().inc();
        }
    }
}

lazy_static! {
    pub static ref TIME_ROUTER: Mutex<TimeRouter> = {
        Mutex::new(TimeRouter::new())
    };
}
