use lazy_static::lazy_static;
use spin::Mutex;
use os::ata_block_driver::AtaPio;
use alloc::vec::Vec;

trait USTARItem {
    fn get_name(&self) -> String {
        
    }
}

struct Directory {

}

struct File {

}


struct USTARFileSystem {
    block_driver: AtaPio,
}

impl UstarFileSystem {
    fn new() -> USTARFileSystem {
        USTARFileSystem {

        }   
    }

    pub fn init() {
        
    }
}

lazy_static! {
    pub static ref USTARFS: Mutex<UstarFileSystem> = {
        Mutex::new(UstarFileSystem::new())
    };
}