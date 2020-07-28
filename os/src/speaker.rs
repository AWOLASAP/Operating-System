use crate::println;
use x86::io::inb;
use x86::io::outb;
use crate::timer_routing::TIME_ROUTER;
use lazy_static::lazy_static;
use spin::Mutex;


lazy_static! {
    pub static ref PCSPEAKER: Mutex<PcSpeaker> = {
        Mutex::new(PcSpeaker::new())
    };
}

pub struct PcSpeaker {
    timer: i32,
    timer_limit: i32,
    timer_done: bool,
    div: i32,
    tmp: i32,
}

impl PcSpeaker {
    pub fn new() -> PcSpeaker { 
        PcSpeaker{
            timer: 0,
            timer_limit: 0,
            timer_done: false,
            div: 0,
            tmp: 0,
        }
    }

    pub fn play_sound(&mut self, frequence: i32) {
        // Set the PIT to the desired frequency
        // If `frequence` is 0, stop the function
        if frequence == 0 {
            println!("\nInvalid Frequency: {}", frequence);
            return;
        } else {
            self.div = 1193180 / frequence;
        }

        unsafe {
            outb(0x43, 0xb6);
            outb(0x42, (self.div) as u8);
            outb(0x42, (self.div >> 8) as u8);
        }    

        // And play the sound using the PC speaker
        unsafe { self.tmp = inb(0x61).into(); }
        if self.tmp != (self.tmp | 3) {
            unsafe { outb(0x61, self.tmp as u8| 3); }
        }

        println!("\nBEEP!");
    }

    // Make it shutup
    pub fn no_sound(&mut self) {
        unsafe { self.tmp = (inb(0x61) & 0xFC) as i32; }
        
        unsafe { outb(0x61, self.tmp as u8); }
    }

    pub fn start_timer(&mut self, limit: i32) {
        self.timer = 0;
        self.timer_limit = limit;
        self.timer_done = false;
        TIME_ROUTER.lock().mode = 2;
    }

    pub fn inc_timer(&mut self) {
        self.timer += 1;
        if self.timer >= self.timer_limit {
            self.stop_timer();
            self.no_sound();
        }
    }
    
    pub fn stop_timer(&mut self) {
        unsafe {TIME_ROUTER.force_unlock()};
        TIME_ROUTER.lock().mode = 0;
        self.timer_done = true;
        self.no_sound();
    }

    pub fn tet_ost(&mut self) {
        let notes = [21, 31, 16, 18, 16, 31, 27];

        for note in notes.iter() {
            self.play_sound(*note as i32);
            self.start_timer(3);
            while self.timer_done == false {}
        }
    }


}

// Make a beep
pub fn beep(freq: i32, len: i32) {
    PCSPEAKER.lock().play_sound(freq);
    PCSPEAKER.lock().start_timer(len);
    //set_PIT_2(old_frequency);
}

pub fn inc_speaker_timer_fn() {
    PCSPEAKER.lock().inc_timer();
}

pub fn play_tet_ost_fn() {
    PCSPEAKER.lock().tet_ost();
}

// Macro to allow beeps to be played in other files
#[macro_export]
macro_rules! play_beep {
    ($f: expr, $l: expr) => {crate::speaker::beep($f, $l)};
}

#[macro_export]
macro_rules! inc_speaker_timer {
    () => {crate::speaker::inc_speaker_timer_fn()};
}

#[macro_export]
macro_rules! play_tet_ost {
    () => {crate::speaker::play_tet_ost_fn()};
}
