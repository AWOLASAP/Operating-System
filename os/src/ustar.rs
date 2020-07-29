use lazy_static::lazy_static;
use spin::Mutex;
use os::ata_block_driver::AtaPio;
use alloc::vec::Vec;
use alloc::string::String;
use hashbrown::HashMap;
use core::option::Option;

// Note to me tomorrow - we're going to use Rc<RefCell<File>> and Directory because
// It gives me interior mutability (RefCell), and shared ownership (Rc). This is important
// because it lets me change it at different times for different reasons to do different things. 
// Basically I can hold references to the USTARItems in both Directories and the File master array
// Because they kinda need to be mutable (to make defragmenting optimized and possible)

trait USTARItem {
    // Handles changing filenames and stuff
    // NOTE: filenames for things like folders do matter, so make sure to change subfolders and stuff too
    // Or (evil idea) - implement that in the change name/prefix for folders
    fn get_name(&self) -> String;
    fn set_name(&mut self, name: String);
    fn set_prefix(&mut self, prefix: String);
    fn get_name_and_prefix(&self) -> String;


    fn should_write(&mut self);
    fn get_should_write(&self) -> bool;

    // Get writable representation - this is how the driver actually applies changes to disk
    // Driver should auto handle writing the vector/using the right number of sectors, but 
    // care should still be made to making it a correct multiple
    fn get_writable_representation(&self) -> Vec<u8>;
    

    // Gets and sets the block ID - do this RARELY, probably only during initialization of the
    // file and defragmenting - otherwise you could lose the file and corrupt the disk
    fn get_block_id(&self) -> u64;
    fn set_block_id(&mut self, block_id: u64);


}

struct Directory {
    contents: Vec<File>,
    subdirectories: Vec<Directory>,

    //Additional needed stuff not from the USTAR filesystem.
    // What block is this hosted on? (we need this for writing to disk)
    block_id: u64,

    // Stuff needed by the USTAR filesystem
    // https://wiki.osdev.org/USTAR
    name: String,
    mode: String,
    owner_id: u64, 
    group_id: u64,
    size: u64, // Should always be 0
    time: u64,
    checksum: u64, // 256 + the sum of all the bytes in this header except the checksum field.
    type_flag: u8, // Should always be 5
    linked_name: String,
    owner_name: String, 
    group_name: String,
    device_major_number: u64,
    device_minor_number: u64,
    prefix: String,
}

impl Directory {
    fn from_block(block: Vec<u8>, block_id: u64) -> File {

    }



}

struct File {
    data: Vec<u8>,

    //Additional needed stuff not from the USTAR filesystem.
    // What block is this hosted on? (we need this for writing to disk)
    block_id: u64,

    // Stuff needed by the USTAR filesystem
    // https://wiki.osdev.org/USTAR
    name: String,
    mode: String,
    owner_id: u64, 
    group_id: u64,
    size: u64,
    time: u64,
    checksum: u64, // 256 + the sum of all the bytes in this header except the checksum field.
    type_flag: u8, // Should always be 0
    linked_name: String,
    owner_name: String, 
    group_name: String,
    device_major_number: u64,
    device_minor_number: u64,
    prefix: String,
}

impl File {
    fn from_block(block: Vec<u8>, block_id: u64) -> File {
        // Handle name
        let name = String::with_capacity(100);
        for i in 0..100 {
            let chr = match block.get(i) {
                Some(chr) => chr as char,
                None => 'a',
            };
        }
    }

    fn get_data(&self) -> Vec<u8> {

    }
    // This handles size 
    fn set_data(&mut self, size: Vec<u8>) {

    }
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
    // Absolute paths need to start with a /
    // Relative paths cannot start with a /
    pub fn get_id(&mut self) {
        self.current_dirs.insert(self.current_dirs_tracker, self.root);
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