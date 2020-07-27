use crate::println;
use x86::io::inb;
use x86::io::outb;


pub fn play_sound(frequence: i32) {
    let div: i32;
    let tmp: i32;

    // Set the PIT to the desired frequency
    if frequence == 0 {
        prinln!("Invalid Frequency: {}", frequence);
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
}

// Make it shutup
pub fn no_sound() {
     unsafe { let tmp: u8 =inb(0x61) & 0xFC; }
    
    //outb(0x61);
}

// Make a beep
pub fn beep(freq: i32) {
    play_sound(freq);
    //timer_wait(10);
    println!("\nBEEP!");
    no_sound();
    //set_PIT_2(old_frequency);
}

// Macro to allow beeps to be played in other files
#[macro_export]
macro_rules! play_beep {
    ($f: expr) => {crate::speaker::beep($f)};
}
