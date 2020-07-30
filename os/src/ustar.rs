use lazy_static::lazy_static;
use spin::Mutex;
use crate::ata_block_driver::AtaPio;
use alloc::vec::Vec;
use alloc::string::String;
use hashbrown::HashMap;
use core::option::Option;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::u64;

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
    contents: Vec<Rc<RefCell<File>>>,
    subdirectories: Vec<Rc<RefCell<Directory>>>,

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
    fn from_block(block: Vec<u8>, block_id: u64) -> Directory {
        // Handle name
        let mut name = String::with_capacity(100);
        for i in 0..100 {
            let chr = match block.get(i) {
                Some(chr) => *chr as char,
                None => '\0',
            };
            name.push(chr);
        }
        // Mode
        let mode = String::from("100777");
        // User and group ID
        let owner_id = 420;
        let group_id = 420;
        // Size
        let mut size = String::with_capacity(10);
        // Skip over the null and 0 byte
        for i in 125..135 {
            let chr = match block.get(i) {
                Some(chr) => *chr as char,
                None => '\0',
            };
            size.push(chr);      
        }
        let size = u64::from_str_radix(size.as_str(), 8);
        let size = match size {
            Ok(i) => i,
            Err(_) => 0,
        };
        // Time
        let mut time = String::with_capacity(10);
        // Skip over the null and 0 byte
        for i in 125..135 {
            let chr = match block.get(i) {
                Some(chr) => *chr as char,
                None => '\0',
            };
            time.push(chr);      
        }
        let time = u64::from_str_radix(time.as_str(), 8);
        let time = match time {
            Ok(i) => i,
            Err(_) => 0,
        };
        // Header checksum
        let mut header = 0;
        // 6223-48-49-48-52-48-53-32+32+32+32+32+32+32+32+32 (example for the hello world file, convert it to octal)
        for (i, n) in block.iter().enumerate() {
            if i > 147 && i < 155 {
                header += 32;
            }
            else {
                header += *n as u64;                
            }
        }
        // Type (should always be 5)
        let type_flag = 5;
        // Linked file name - same name as the normal
        let linked_name = name.clone();
        // Owner and group name
        let mut owner_name = String::with_capacity(32);
        owner_name.push_str("weed");
        let mut group_name = String::with_capacity(32);
        group_name.push_str("weed");
        for i in 0..28 {
            owner_name.push('\0');
            group_name.push('\0');
        }
        // Device major and minor version - not parsing because it probably doesn't matter
        let device_major_number = 0;
        let device_minor_number = 0;
        // Filename prefix
        let mut filename_prefix = String::with_capacity(155);
        for i in 345..500 {
            let chr = match block.get(i) {
                Some(chr) => *chr as char,
                None => '\0',
            };
            filename_prefix.push(chr);
        }
        // Setup directory specific Variables
        let subdirectories = Vec::new();
        let files = Vec::new();
        Directory { 
            name: name,
            mode: mode,
            owner_id: owner_id,
            group_id: group_id,
            size: size,
            time: time,
            checksum: header,
            type_flag: type_flag,
            linked_name: linked_name,
            owner_name: owner_name,
            group_name: group_name,
            device_minor_number: device_minor_number,
            device_major_number: device_major_number,
            prefix: filename_prefix,

            block_id: block_id,

            subdirectories: subdirectories,
            contents: files,
        }
    }



}

pub struct File {
    data: Vec<u8>,

    //Additional needed stuff not from the USTAR filesystem.
    // What block is this hosted on? (we need this for writing to disk)
    block_id: u64,

    // Stuff needed by the USTAR filesystem
    // https://wiki.osdev.org/USTAR
    pub name: String,
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
    pub fn from_block(block: Vec<u8>, block_id: u64) -> File {
        // Handle name
        let mut name = String::with_capacity(100);
        for i in 0..100 {
            let chr = match block.get(i) {
                Some(chr) => *chr as char,
                None => '\0',
            };
            name.push(chr);
        }
        // Mode
        let mode = String::from("100777");
        // User and group ID
        let owner_id = 420;
        let group_id = 420;
        // Size
        let mut size = String::with_capacity(10);
        // Skip over the null and 0 byte
        for i in 125..135 {
            let chr = match block.get(i) {
                Some(chr) => *chr as char,
                None => '\0',
            };
            size.push(chr);      
        }
        let size = u64::from_str_radix(size.as_str(), 8);
        let size = match size {
            Ok(i) => i,
            Err(_) => 0,
        };
        // Time
        let mut time = String::with_capacity(10);
        // Skip over the null and 0 byte
        for i in 125..135 {
            let chr = match block.get(i) {
                Some(chr) => *chr as char,
                None => '\0',
            };
            time.push(chr);      
        }
        let time = u64::from_str_radix(time.as_str(), 8);
        let time = match time {
            Ok(i) => i,
            Err(_) => 0,
        };
        // Header checksum
        let mut header = 0;
        // 6223-48-49-48-52-48-53-32+32+32+32+32+32+32+32+32 (example for the hello world file, convert it to octal)
        for (i, n) in block.iter().enumerate() {
            if i > 147 && i < 155 {
                header += 32;
            }
            else {
                header += *n as u64;                
            }
        }
        // Type (should always be 0)
        let type_flag = 0;
        // Linked file name - same name as the normal
        let linked_name = name.clone();
        // Owner and group name
        let mut owner_name = String::with_capacity(32);
        owner_name.push_str("weed");
        let mut group_name = String::with_capacity(32);
        group_name.push_str("weed");
        for i in 0..28 {
            owner_name.push('\0');
            group_name.push('\0');
        }
        // Device major and minor version - not parsing because it probably doesn't matter
        let device_major_number = 0;
        let device_minor_number = 0;
        // Filename prefix
        let mut filename_prefix = String::with_capacity(155);
        for i in 345..500 {
            let chr = match block.get(i) {
                Some(chr) => *chr as char,
                None => '\0',
            };
            filename_prefix.push(chr);
        }
        // Setup empty data
        let mut data = Vec::new();
        File {
            name: name,
            mode: mode,
            owner_id: owner_id,
            group_id: group_id,
            size: size,
            time: time,
            checksum: header,
            type_flag: type_flag,
            linked_name: linked_name,
            owner_name: owner_name,
            group_name: group_name,
            device_minor_number: device_minor_number,
            device_major_number: device_major_number,
            prefix: filename_prefix,

            block_id: block_id,

            data: data,
        }
    }

    fn to_block(&self) -> Vec<u8> {

    }

    fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }
    // This handles size 
    fn set_data(&mut self, data: Vec<u8>) {
        self.data = data.clone();
    }
}

struct USTARFileSystem {
    block_driver: AtaPio,
    files: Vec<Rc<RefCell<USTARItem>>>,
    current_dirs: HashMap<u64, Directory>,
    current_dirs_tracker: u64,
    root: Directory,
}
/*
impl UstarFileSystem {
    fn new() -> USTARFileSystem {
        USTARFileSystem {

        }   
    }

    pub fn init(&mut self) {
        // Read in all the files/directories
        // First mainly process directories to build the structure of the VFS, then place files in it
        // Initialize (read the data) for the files in the 2nd pass
        // Well actually, do this differently
        // Process any directories that are relative to root 
        // Basically keep on going through, check if a path can be accessed (and thus subdirectories can be added)
        // Can fill the files array early - probably on first run
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
*/