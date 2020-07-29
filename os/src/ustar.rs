use lazy_static::lazy_static;
use spin::Mutex;
use os::ata_block_driver::AtaPio;
use alloc::vec::Vec;
use alloc::string::String;
use hashbrown::HashMap;
use core::option::Option;

trait USTARItem {
    fn get_name(&self) -> String;
    fn set_name(&mut self, name: String);
    fn set_prefix(&mut self, prefix: String);
    fn get_name_and_prefix(&self) -> String;

    fn get_size(&self) -> usize;

    fn should_write(&mut self);
    fn get_should_write(&self) -> bool;

    // Stuff related to directories
    fn is_directory(&self) -> bool;
    fn get_dir_contents(&self) -> Vec<USTARItem>;
}

struct Directory {
    contents: Vec<File>,
    subdirectories: Vec<Directory>,
    data: Vec<u8>,
}

struct File {

}


struct USTARFileSystem {
    block_driver: AtaPio,
    files: Vec<USTARItem>,
    current_dirs: HashMap<u64, Directory>,
    current_dirs_tracker: u64,
    root: Directory,
}

impl UstarFileSystem {
    fn new() -> USTARFileSystem {
        USTARFileSystem {

        }   
    }

    pub fn init(&mut self) {
        // Read in all the files/directories
    }

    pub fn defragment(&mut self) {
        // Remove all files named defragment, than move the rest of the files
    }

    fn write(&mut self) {
        //Write any changes
    }

    // Directory based seeking functions (will handle things like ls in the future)
    pub fn get_id(&mut self) {
        self.current_dirs.insert(self.current_dirs_tracker, root);
    }

    pub fn list_files(&self, id: u64) -> Vec<String> {

    }

    pub fn list_subdirectories(&self, id: u64) -> Vec<String> {

    }

    pub fn change_directory(&mut self, directory: String, id: u64) -> bool {

    }

    pub fn change_directory_absolute_path(&mut self, path: String, id: u64) -> bool {

    }

    pub fn read_file(&self, file: String, id: u64) -> Option<Vec<u8>> {

    }

    pub fn read_file_absolute_path(&self, path: String, id: u64) -> Option<Vec<u8>> {
        
    }

    pub fn write_file(&self, file: String, data: Vec<u8>, id: u64) -> bool {

    }

    pub fn write_file_absolute_path(&self, path: String, data: Vec<u8>, id: u64) -> bool {
        
    }
}

lazy_static! {
    pub static ref USTARFS: Mutex<UstarFileSystem> = {
        Mutex::new(UstarFileSystem::new())
    };
}