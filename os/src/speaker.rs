use crate::println;
use x86::io::inb;
use x86::io::outb;
use crate::timer_routing::TIME_ROUTER;
use lazy_static::lazy_static;
use spin::Mutex;


lazy_static! {
    pub static ref SPEAKER: Mutex<Speaker> = {
        Mutex::new(Speaker::new())
    };
}

pub struct Speaker {
    timer: i32,
    timer_limit: i32,
}

impl Speaker {

    pub fn new() -> Speaker { Speaker{
            timer: 0,
            timer_limit: 0,
        }
    }

    pub fn play_sound(&self, frequence: i32) {
        let div: i32;
        let tmp: i32;

        // Set the PIT to the desired frequency
        // if frequence is 0, stop the function
        if frequence == 0 {
            println!("\nInvalid Frequency: {}", frequence);
            return;
        } else {
            div = 1193180 / frequence;
        }

        unsafe {
            outb(0x43, 0xb6);
            outb(0x42, (div) as u8);
            outb(0x42, (div >> 8) as u8);
        }    

        // And play the sound using the PC speaker
        unsafe { tmp = inb(0x61).into(); }
        if tmp != (tmp | 3) {
            unsafe { outb(0x61, tmp as u8| 3); }
            println!("{}", tmp as u8 | 3)
        }

        println!("\nBEEP!");
    }

    // Make it shutup
    pub fn no_sound(&self) {
         unsafe { let tmp: u8 =inb(0x61) & 0xFC; }
        
        //outb(0x61);
    }

    pub fn start_timer(&mut self, limit: i32) {
        self.timer = 0;
        self.timer_limit = limit;
        TIME_ROUTER.lock().mode = 2;
    }

    pub fn inc_timer(&mut self) {
        self.timer += 1;
        if self.timer >= self.timer_limit {
            self.stop_timer();
        }
    }
    
    pub fn stop_timer(&mut self) {
        unsafe {TIME_ROUTER.force_unlock()};
        TIME_ROUTER.lock().mode = 0;
    }

}

// Make a beep
pub fn beep(freq: i32) {
    SPEAKER.play_sound(freq);
    SPEAKER.start_timer(100);
    SPEAKER.no_sound();
    //set_PIT_2(old_frequency);
}

pub fn inc_speaker_timer_fn() {
    SPEAKER.inc_timer();
}

// Macro to allow beeps to be played in other files
#[macro_export]
macro_rules! play_beep {
    ($f: expr) => {crate::speaker::beep($f)};
}

#[macro_export]
macro_rules! inc_speaker_timer {
    () => {crate::speaker::inc_speaker_timer_fn()};
}
