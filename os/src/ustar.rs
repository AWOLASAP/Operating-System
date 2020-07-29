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

    // Stuff needed by the USTAR filesystem
    name: String,
    mode: String,
    owner_id: u64, 
    group_id: u64,
    size: u64,
    time: u64,
    checksum: u64, // 256 + the sum of all the bytes in this header except the checksum field.
    type_flag: u8, 
    linked_name: String,
    owner_name: String, 
    group_name: String,
    device_major_number: u64,
    device_minor_number: u64,
    prefix: String,
}

struct File {

    // Stuff needed by the USTAR filesystem
    name: String,
    mode: String,
    owner_id: u64, 
    group_id: u64,
    size: u64,
    time: u64,
    checksum: u64, // 256 + the sum of all the bytes in this header except the checksum field.
    type_flag: u8, 
    linked_name: String,
    owner_name: String, 
    group_name: String,
    device_major_number: u64,
    device_minor_number: u64,
    prefix: String,
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
        // Remove all files named defragment, than move the rest of the files (blockwise), so that it's still valid USTAR
    }

    fn write(&mut self) {
        //Write any changes
    }

    // Features to add - 
    // Copy
    // Move
    // Rename
    // (for files)

    // Directory based seeking functions (will handle things like ls in the future)
    // First, call the get_id function - once your program has such an ID it can do things
    // Like control which directory is active (at least for it)
    // Not sure how this will handle having a parent directory deleted (yet)
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

    // If a file doesn't exist, returns None
    pub fn read_file(&self, file: String, id: u64) -> Option<Vec<u8>> {

    }

    pub fn read_file_absolute_path(&self, path: String) -> Option<Vec<u8>> {
        
    }

    // If a file doesn't exist, running this function will create it
    // Doesn't append to the data, but flat out replaces it - changes in allocation need to defrag
    // Does not account for if you write nothing, you're on your own
    pub fn write_file(&self, file: String, data: Vec<u8>, id: u64) {

    }

    pub fn write_file_absolute_path(&self, path: String, data: Vec<u8>) {
        
    }

    // Removes a file if it exists, does nothing if it doesn't
    pub fn remove_file(&self, file: String, id: u64) {

    }

    pub fn remove_file_absolute_path(&self, path: String) {
        
    }

    // Probably should use directory moving to implement this
    // If it doesn't exist, nothing happens
    pub fn rename_directory(&self, dir: String, new_name: String, id: u64) -> bool {

    }

    // 2nd parameter should not be complete path, but only the new name
    pub fn rename_directory_absolute_path(&self, path: String, new_name: String) -> bool {
        
    }

    // Creates a directory unless there exists a file or directory with a similar name
    pub fn create_directory(&self, file: String, id: u64) -> bool {

    }

    pub fn create_directory_absolute_path(&self, path: String) -> bool {
        
    }

    // Removes a directory if it exists, does nothing if it doesn't
    pub fn remove_directory(&self, file: String, id: u64) {
        
    }

    pub fn remove_directory_absolute_path(&self, path: String) {
        
    }

    // 2nd parameter should be complete path, but not the first one (first one should be relative)
    // If 1st param doesn't exist, or 2nd param already exists, it won't do anything
    pub fn move_directory(&self, dir: String, new_path: String, id: u64) -> bool {

    }

    // Ignore the above - for this function, both strings should be complete paths
    pub fn move_directory_absolute_path(&self, path: String, new_path: String) -> bool {
        
    }
}

lazy_static! {
    pub static ref USTARFS: Mutex<UstarFileSystem> = {
        Mutex::new(UstarFileSystem::new())
    };
}