use lazy_static::lazy_static;
use spin::Mutex;

// This entire file is just providing a singleton which functions as a provider for RNG seeds
// In a given boot, you can be reasonably confident that you will not encounter the same seed

pub struct RngSeed {
    pub times: u64,
}

impl RngSeed {
    pub fn inc(&mut self) {
        self.times += 1;
    }

    pub fn get(&mut self) -> u64 {
        self.times += 1;
        self.times
    }
}

lazy_static! {
    pub static ref RNGSEED: Mutex<RngSeed> = Mutex::new(RngSeed {times: 0});
}