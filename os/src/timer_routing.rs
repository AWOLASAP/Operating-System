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

pub struct Modes {
    pub terminal: bool,
    pub tetris: bool,
    pub beep: bool,
    pub song: bool,
}

impl Modes {
    fn new() -> Modes {
        Modes {
            terminal: true,
            tetris: false,
            beep: false,
            song: false,
        }
    }
}

pub struct TimeRouter {
    pub mode: Modes,
}

impl TimeRouter {
    fn new() -> TimeRouter {
        TimeRouter { mode: Modes::new() }
    }

    // Called on every timer interrupt
    pub fn handle(&mut self) {
        if self.mode.terminal {
            MODE.lock().blink_current();
            RNGSEED.lock().inc();
        }
        if self.mode.tetris {
            TETRIS.lock().game_loop();
            RNGSEED.lock().inc();
        } 
        if self.mode.beep {
            inc_speaker_timer!();
        }
        if self.mode.song {
            PCSPEAKER.lock().song_loop();
        }
    }
}

lazy_static! {
    pub static ref TIME_ROUTER: Mutex<TimeRouter> = {
        Mutex::new(TimeRouter::new())
    };
}
