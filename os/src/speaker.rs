#![allow(non_snake_case)]

use x86::io::inb; use x86::io::outb;
use crate::timer_routing::TIME_ROUTER;
use crate::keyboard_routing::KEYBOARD_ROUTER;
use lazy_static::lazy_static;
use spin::Mutex;


lazy_static! {
    pub static ref PCSPEAKER: Mutex<PcSpeaker> = {
        Mutex::new(PcSpeaker::new())
    };
}

pub struct PcSpeaker {
    timer: i32,
    current_run: i32,
    total_runs: i32,
    timer_limit: i32,
    timer_done: bool,
    div: i32,
    tmp: i32,
}


impl Default for PcSpeaker {
    fn default() -> PcSpeaker {
        PcSpeaker::new()
    }
}

// Class to handle playing sounds through the PC speaker
impl PcSpeaker {
    // Creates a new class with default values
    pub fn new() -> PcSpeaker { 
        PcSpeaker{
            timer: 0,
            current_run: 1,
            total_runs: 1,
            timer_limit: 0,
            timer_done: false,
            div: 0,
            tmp: 0,
        }
    }

    // play a frequency through the PC speaker.
    pub fn play_sound(&mut self, frequence: i32) {
        // Make sure nothing is playing beforehand
        self.no_sound();

        // Set the PIT to the desired frequency
        // If `frequence` is 0, stop the function since nothing should be played
        // 1193180 is the frequency of the PIT
        if frequence == 0 {
            return;
        } else {
            self.div = 1193180 / frequence;
        }

        // Write to the IO ports that are connected to the PIT
        // First tell the PIT what values we want, e.g. channel 2, etc.
        // then the high value, and lastly the low value
        unsafe {
            outb(0x43, 0xb6);
            outb(0x42, (self.div) as u8);
            outb(0x42, (self.div >> 8) as u8);
        }    

        // Tell the PIT to start
        unsafe { self.tmp = inb(0x61).into(); }
        if self.tmp != (self.tmp | 3) {
            unsafe { outb(0x61, self.tmp as u8| 3); }
        }
    }

    // Make it shutup by writing to the IO port to stop
    pub fn no_sound(&mut self) {
        unsafe { self.tmp = (inb(0x61) & 0xFC) as i32; }
        
        unsafe { outb(0x61, self.tmp as u8); }
    }

    // Start the loop for the Tetris song
    pub fn start_song_loop(&mut self, repeat: i32) {
        self.timer = 0;
        self.current_run += 1;
        self.total_runs = repeat;
        TIME_ROUTER.lock().mode.terminal = false;
        TIME_ROUTER.lock().mode.song = true;
        unsafe {KEYBOARD_ROUTER.force_unlock()};
        KEYBOARD_ROUTER.lock().mode.song = true;
    }

    // This is what is called by TIME_ROUTER every loop
    pub fn song_loop(&mut self) {
        self.timer += 1;
        self.tet_ost();
    }
     
    // Stop the song loop
    pub fn stop_song_loop(&mut self, force: bool) {
        unsafe {TIME_ROUTER.force_unlock()};
        TIME_ROUTER.lock().mode.terminal = true;
        TIME_ROUTER.lock().mode.song = false;
        unsafe {KEYBOARD_ROUTER.force_unlock()};
        KEYBOARD_ROUTER.lock().mode.song = false;
        self.no_sound();
        if force {
            self.current_run = 0;
        }
        else if self.total_runs == 0 || self.current_run < self.total_runs {
            self.start_song_loop(self.total_runs);
        } else {
            self.current_run = 0;
        }
    }

    // Start the timer used for simple beeps
    pub fn start_timer(&mut self, limit: i32) {
        self.timer = 0;
        self.timer_limit = limit;
        self.timer_done = false;
        TIME_ROUTER.lock().mode.terminal = false;
        TIME_ROUTER.lock().mode.beep = true;
    }

    // Increment the simple timer
    pub fn inc_timer(&mut self) {
        self.timer += 1;
        if self.timer >= self.timer_limit {
            self.stop_timer();
            self.no_sound();
        }
    }
    
    // Stop the simple timer
    pub fn stop_timer(&mut self) {
        unsafe {TIME_ROUTER.force_unlock()};
        TIME_ROUTER.lock().mode.terminal = true;
        TIME_ROUTER.lock().mode.beep = false;
        self.timer_done = true;
        self.no_sound();
    }

    // All of the frequencies/notes of the Tetris soundtrack,
    // note just lead notes are played, no bass
    pub fn tet_ost(&mut self) {
        let _R   =     0;
        let _C0  = 16.35;
        let _CS0 = 17.32;
        let _D0  = 18.35;
        let _DS0 = 19.45;
        let _E0  = 20.60;
        let _F0  = 21.83;
        let _FS0 = 23.12;
        let _G0  = 24.50;
        let _GS0 = 25.96;
        let _A0  = 27.50;
        let _AS0 = 29.14;
        let _B0  = 30.87;
        let _C1  = 32.70;
        let _CS1 = 34.65;
        let _D1  = 36.71;
        let _DS1 = 38.89;
        let _E1  = 41.20;
        let _F1  = 43.65;
        let _FS1 = 46.25;
        let _G1  = 49.00;
        let _GS1 = 51.91;
        let _A1  = 55.00;
        let _AS1 = 58.27;
        let _B1  = 61.74;
        let _C2  = 65.41;
        let _CS2 = 69.30;
        let _D2  = 73.42;
        let _DS2 = 77.78;
        let _E2  = 82.41;
        let _F2  = 87.31;
        let _FS2 = 92.50;
        let _G2  = 98.00;
        let _GS2 = 103.83;
        let _A2  = 110.00;
        let _AS2 = 116.54;
        let _B2  = 123.47;
        let _C3  = 130.81;
        let _CS3 = 138.59;
        let _D3  = 146.83;
        let _DS3 = 155.56;
        let _E3  = 164.81;
        let _F3  = 174.61;
        let _FS3 = 185.00;
        let _G3  = 196.00;
        let _GS3 = 207.65;
        let _A3  = 220.00;
        let _AS3 = 233.08;
        let _B3  = 246.94;
        let _C4  = 261.63;
        let _CS4 = 277.18;
        let _D4  = 293.66;
        let _DS4 = 311.13;
        let _E4  = 329.63;
        let _F4  = 349.23;
        let _FS4 = 369.99;
        let _G4  = 392.00;
        let _GS4 = 415.30;
        let _A4  = 440.00;
        let _AS4 = 466.16;
        let _B4  = 493.88;
        let _C5  = 523.25;
        let _CS5 = 554.37;
        let _D5  = 587.33;
        let _DS5 = 622.25;
        let _E5  = 659.25;
        let _F5  = 698.46;
        let _FS5 = 739.99;
        let _G5  = 783.99;
        let _GS5 = 830.61;
        let _A5  = 880.00;
        let _AS5 = 932.33;
        let _B5  = 987.77;
        let _C6  = 1046.50;
        let _CS6 = 1108.73;
        let _D6  = 1174.66;
        let _DS6 = 1244.51;
        let _E6  = 1318.51;
        let _F6  = 1396.91;
        let _FS6 = 1479.98;
        let _G6  = 1567.98;
        let _GS6 = 1661.22;
        let _A6  = 1760.00;
        let _AS6 = 1864.66;
        let _B6  = 1975.53;
        let _C7  = 2093.00;
        let _CS7 = 2217.46;
        let _D7  = 2349.32;
        let _DS7 = 2489.02;
        let _E7  = 2637.02;
        let _F7  = 2793.83;
        let _FS7 = 2959.96;
        let _G7  = 3135.96;
        let _GS7 = 3322.44;
        let _A7  = 3520.00;
        let _AS7 = 3729.31;
        let _B7  = 3951.07;
        let _C8  = 4186.01;
        let _CS8 = 4434.92;
        let _D8  = 4698.63;
        let _DS8 = 4978.03;
        let _E8  = 5274.04;
        let _F8  = 5587.65;
        let _FS8 = 5919.91;
        let _G8  = 6271.93;
        let _GS8 = 6644.88;
        let _A8  = 7040.00;
        let _AS8 = 7458.62;
        let _B8  = 7902.13;
        match self.timer {

            3 => self.play_sound(_E5 as i32),
            9 => self.play_sound(_B4 as i32),
            12 => self.play_sound(_C5 as i32),
            15 => self.play_sound(_D5 as i32),
            21 => self.play_sound(_C5 as i32),
            24 => self.play_sound(_B4 as i32),
            27 => self.play_sound(_A4 as i32),
            33 => self.play_sound(_A4 as i32),
            36 => self.play_sound(_C5 as i32),
            39 => self.play_sound(_E5 as i32),
            45 => self.play_sound(_D5 as i32),
            48 => self.play_sound(_C5 as i32),
            51 => self.play_sound(_B4 as i32),
            57 => self.play_sound(_B4 as i32),
            60 => self.play_sound(_C5 as i32),
            63 => self.play_sound(_D5 as i32),
            69 => self.play_sound(_E5 as i32),
            75 => self.play_sound(_C5 as i32),
            81 => self.play_sound(_A4 as i32),
            87 => self.play_sound(_A4 as i32),
            93 => self.play_sound(_R  as i32),
            105 => self.play_sound(_D5 as i32),
            111 => self.play_sound(_F5 as i32),
            114 => self.play_sound(_A5 as i32),
            120 => self.play_sound(_G5 as i32),
            123 => self.play_sound(_F5 as i32),
            126 => self.play_sound(_E5 as i32),
            135 => self.play_sound(_C5 as i32),
            138 => self.play_sound(_E5 as i32),
            144 => self.play_sound(_D5 as i32),
            147 => self.play_sound(_C5 as i32),
            150 => self.play_sound(_B4 as i32),
            156 => self.play_sound(_B4 as i32),
            159 => self.play_sound(_C5 as i32),
            162 => self.play_sound(_D5 as i32),
            168 => self.play_sound(_E5 as i32),
            174 => self.play_sound(_C5 as i32),
            180 => self.play_sound(_A4 as i32),
            186 => self.play_sound(_A4 as i32),
            192 => self.play_sound(_R  as i32),
            198 => self.play_sound(_E4 as i32),
            210 => self.play_sound(_C4 as i32),
            222 => self.play_sound(_D4 as i32),
            234 => self.play_sound(_B3 as i32),
            246 => self.play_sound(_C4 as i32),
            268 => self.play_sound(_A3 as i32),
            280 => self.play_sound(_GS3 as i32),
            292 => self.play_sound(_B3 as i32),
            304 => self.play_sound(_E4 as i32),
            316 => self.play_sound(_C4 as i32),
            328 => self.play_sound(_D4 as i32),
            340 => self.play_sound(_B3 as i32),
            352 => self.play_sound(_C4 as i32),
            358 => self.play_sound(_E4 as i32),
            364 => self.play_sound(_A4 as i32),
            370 => self.play_sound(_A4 as i32),
            376 => self.play_sound(_GS4 as i32),
            394 => self.play_sound(_R as i32),
            400 => self.stop_song_loop(false),
            _ => (),
        }

    }
}

// Make a beep
pub fn beep(freq: i32, len: i32) {
    PCSPEAKER.lock().play_sound(freq);
    PCSPEAKER.lock().start_timer(len);
    //set_PIT_2(old_frequency);
}

// Increment the timer used for simple beeps
pub fn inc_speaker_timer_fn() {
    PCSPEAKER.lock().inc_timer();
}

// Play the Tetris soundtrack
pub fn play_tet_ost_fn(num: i32) {
    PCSPEAKER.lock().start_song_loop(num);
}

// Stop the Tetris soundtrack
pub fn end_tet_ost_fn() {
    PCSPEAKER.lock().stop_song_loop(true);
}

// Macro to allow beeps to be played in other files
#[macro_export]
macro_rules! play_beep {
    ($f: expr, $l: expr) => {crate::speaker::beep($f, $l)};
}

// Macro to allow incrementing the speaker timer in other files
#[macro_export]
macro_rules! inc_speaker_timer {
    () => {crate::speaker::inc_speaker_timer_fn()};
}

// Macro to allow playing the Tetris soundtrack in other files
#[macro_export]
macro_rules! play_tet_ost {
    ($i: expr) => {crate::speaker::play_tet_ost_fn($i)};
}

// Macro to allow stopping the Tetris soundtrack in other files
#[macro_export]
macro_rules! end_tet_ost {
    () => {crate::speaker::end_tet_ost_fn()};
}
