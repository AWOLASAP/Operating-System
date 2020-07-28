use lazy_static::lazy_static;
use spin::Mutex;
use crate::inc_speaker_timer;
use crate::{
    tetris::TETRIS,
    rng::RNGSEED,
    vga_buffer::MODE,
    inc_speaker_timer
};


/* 
 * MODES
 * 0 - Terminal + RNG
 * 1 - Tetris + RNG
 * 2 - Speaker
*/

pub struct TimeRouter {
    pub mode: usize,
}

impl TimeRouter {
    fn new() -> TimeRouter {
        TimeRouter { mode: 0 }
    }

    // Called on every timer interrupt
    pub fn handle(&mut self) {
        if self.mode == 0 {
            MODE.lock().blink_current();
            RNGSEED.lock().inc();
        }
        else if self.mode == 1 {
            TETRIS.lock().game_loop();
            RNGSEED.lock().inc();
        } 
        else if self.mode == 2 {
            inc_speaker_timer!();
        }
    }
}

lazy_static! {
    pub static ref TIME_ROUTER: Mutex<TimeRouter> = {
        Mutex::new(TimeRouter::new())
    };
}
