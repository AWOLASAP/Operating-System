use lazy_static::lazy_static;
use spin::Mutex;
use crate::{
    tetris::TETRIS,
    rng::RNGSEED,
    vga_buffer::MODE,
    inc_speaker_timer,
    speaker::PCSPEAKER,
};


/*
 * MODES
 * 0 - Terminal + RNG
 * 1 - Tetris + RNG
 * 2 - Speaker - Beep
 * 3 - Speaker - Song
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
            RNGSEED.lock().inc();
        }
        else if self.mode == 2 {
            inc_speaker_timer!();
        }
        else if self.mode == 3 {
            PCSPEAKER.lock().song_loop();
        }
    }
}

lazy_static! {
    pub static ref TIME_ROUTER: Mutex<TimeRouter> = {
        Mutex::new(TimeRouter::new())
    };
}
