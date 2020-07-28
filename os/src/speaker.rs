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
        // Make sure nothing is playing before      
        self.no_sound();

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

    pub fn start_song_loop(&mut self) {
        self.timer = 0;
        TIME_ROUTER.lock().mode = 3;
    }

    pub fn song_loop(&mut self) {
        self.timer += 1;
        self.tet_ost();
    }
     
    pub fn stop_song_loop(&mut self) {
        unsafe {TIME_ROUTER.force_unlock()};
        TIME_ROUTER.lock().mode = 0;
        self.no_sound();
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
        match self.timer {
           /* 0 => self.play_sound(21),
            2 => self.play_sound(31),
            4 => self.play_sound(16),
            6 => self.play_sound(18),
            8 => self.play_sound(16),
            10 => self.play_sound(31),
            12 => self.play_sound(27),
            14 => self.stop_song_loop(),
           */

            000 => self.play_sound(17.32 as i32),
            003 => self.play_sound(18.35 as i32),
            006 => self.play_sound(19.45 as i32),
            009 => self.play_sound(20.60 as i32),
            012 => self.play_sound(21.83 as i32),
            015 => self.play_sound(23.12 as i32),
            018 => self.play_sound(24.50 as i32),
            021 => self.play_sound(25.96 as i32),
            024 => self.play_sound(27.50 as i32),
            027 => self.play_sound(29.14 as i32),
            030 => self.play_sound(30.87 as i32),
            033 => self.play_sound(32.70 as i32),
            036 => self.play_sound(34.65 as i32),
            039 => self.play_sound(36.71 as i32),
            042 => self.play_sound(38.89 as i32),
            045 => self.play_sound(41.20 as i32),
            048 => self.play_sound(43.65 as i32),
            051 => self.play_sound(46.25 as i32),
            054 => self.play_sound(49.00 as i32),
            057 => self.play_sound(51.91 as i32),
            060 => self.play_sound(55.00 as i32),
            063 => self.play_sound(58.27 as i32),
            066 => self.play_sound(61.74 as i32),
            069 => self.play_sound(65.41 as i32),
            072 => self.play_sound(69.30 as i32),
            075 => self.play_sound(73.42 as i32),
            078 => self.play_sound(77.78 as i32),
            081 => self.play_sound(82.41 as i32),
            084 => self.play_sound(87.31 as i32),
            087 => self.play_sound(92.50 as i32),
            090 => self.play_sound(98.00 as i32),
            093 => self.play_sound(103.83 as i32),
            096 => self.play_sound(110.00 as i32),
            099 => self.play_sound(116.54 as i32),
            102 => self.play_sound(123.47 as i32),
            105 => self.play_sound(130.81 as i32),
            108 => self.play_sound(138.59 as i32),
            111 => self.play_sound(146.83 as i32),
            114 => self.play_sound(155.56 as i32),
            117 => self.play_sound(164.81 as i32),
            120 => self.play_sound(174.61 as i32),
            123 => self.play_sound(185.00 as i32),
            126 => self.play_sound(196.00 as i32),
            129 => self.play_sound(207.65 as i32),
            132 => self.play_sound(220.00 as i32),
            135 => self.play_sound(233.08 as i32),
            138 => self.play_sound(246.94 as i32),
            141 => self.play_sound(261.63 as i32),
            144 => self.play_sound(277.18 as i32),
            147 => self.play_sound(293.66 as i32),
            150 => self.play_sound(311.13 as i32),
            153 => self.play_sound(329.63 as i32),
            156 => self.play_sound(349.23 as i32),
            159 => self.play_sound(369.99 as i32),
            162 => self.play_sound(392.00 as i32),
            165 => self.play_sound(415.30 as i32),
            168 => self.play_sound(440.00 as i32),
            171 => self.play_sound(466.16 as i32),
            174 => self.play_sound(493.88 as i32),
            177 => self.play_sound(523.25 as i32),
            180 => self.play_sound(554.37 as i32),
            183 => self.play_sound(587.33 as i32),
            186 => self.play_sound(622.25 as i32),
            189 => self.play_sound(659.25 as i32),
            192 => self.play_sound(698.46 as i32),
            195 => self.play_sound(739.99 as i32),
            198 => self.play_sound(783.99 as i32),
            201 => self.play_sound(830.61 as i32),
            204 => self.play_sound(880.00 as i32),
            207 => self.play_sound(932.33 as i32),
            210 => self.play_sound(987.77 as i32),
            213 => self.play_sound(1046.50 as i32),
            216 => self.play_sound(1108.73 as i32),
            219 => self.play_sound(1174.66 as i32),
            222 => self.play_sound(1244.51 as i32),
            225 => self.play_sound(1318.51 as i32),
            228 => self.play_sound(1396.91 as i32),
            231 => self.play_sound(1479.98 as i32),
            234 => self.play_sound(1567.98 as i32),
            237 => self.play_sound(1661.22 as i32),
            240 => self.play_sound(1760.00 as i32),
            243 => self.play_sound(1864.66 as i32),
            246 => self.play_sound(1975.53 as i32),
            249 => self.play_sound(2093.00 as i32),
            252 => self.play_sound(2217.46 as i32),
            255 => self.play_sound(2349.32 as i32),
            258 => self.play_sound(2489.02 as i32),
            261 => self.play_sound(2637.02 as i32),
            264 => self.play_sound(2793.83 as i32),
            267 => self.play_sound(2959.96 as i32),
            270 => self.play_sound(3135.96 as i32),
            273 => self.play_sound(3322.44 as i32),
            276 => self.play_sound(3520.00 as i32),
            279 => self.play_sound(3729.31 as i32),
            282 => self.play_sound(3951.07 as i32),
            285 => self.play_sound(4186.01 as i32),
            288 => self.play_sound(4434.92 as i32),
            291 => self.play_sound(4698.63 as i32),
            294 => self.play_sound(4978.03 as i32),
            297 => self.play_sound(5274.04 as i32),
            300 => self.play_sound(5587.65 as i32),
            303 => self.play_sound(5919.91 as i32),
            306 => self.play_sound(6271.93 as i32),
            309 => self.play_sound(6644.88 as i32),
            312 => self.play_sound(7040.00 as i32),
            315 => self.play_sound(7458.62 as i32),
            318 => self.play_sound(7902.13 as i32),
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

pub fn inc_speaker_timer_fn() {
    PCSPEAKER.lock().inc_timer();
}

pub fn play_tet_ost_fn() {
    PCSPEAKER.lock().start_song_loop();
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
