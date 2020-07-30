use lazy_static::lazy_static;
use spin::Mutex;
use crate::ata_block_driver::AtaPio;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use hashbrown::HashMap;
use core::option::Option;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::u64;
use alloc::format;

// Note to me tomorrow - we're going to use Rc<RefCell<File>> and Directory because
// It gives me interior mutability (RefCell), and shared ownership (Rc). This is important
// because it lets me change it at different times for different reasons to do different things. 
// Basically I can hold references to the USTARItems in both Directories and the File master array
// Because they kinda need to be mutable (to make defragmenting optimized and possible)

// Should store all the information needed to have it be movable around disk
trait USTARItem {
    // Handles changing filenames and stuff
    // NOTE: filenames for things like folders do matter, so make sure to change subfolders and stuff too
    // Or (evil idea) - implement that in the change name/prefix for folders
    fn get_name(&self) -> String;
    fn set_name(&mut self, name: String);
    fn set_prefix(&mut self, prefix: String);
    fn get_name_and_prefix(&self) -> String;
    //fn set_name_and_prefix(&mut self, combined: String);


    fn should_write(&mut self);
    fn get_should_write(&self) -> bool;

    // Get writable representation - this is how the driver actually applies changes to disk
    // Driver should auto handle writing the vector/using the right number of sectors, but 
    // care should still be made to making it a correct multiple
    fn get_writable_representation(&mut self) -> Vec<u8>;
    

    // Gets and sets the block ID - do this RARELY, probably only during initialization of the
    // file and defragmenting - otherwise you could lose the file and corrupt the disk
    fn get_block_id(&self) -> u64;
    fn set_block_id(&mut self, block_id: u64);


}

pub struct Directory {
    contents: Vec<Rc<RefCell<File>>>,
    subdirectories: Vec<Rc<RefCell<Directory>>>,

    //Additional needed stuff not from the USTAR filesystem.
    // What block is this hosted on? (we need this for writing to disk)
    block_id: u64,
    // Should we write this entire file to disk?
    write: bool,

    // Stuff needed by the USTAR filesystem
    // https://wiki.osdev.org/USTAR
    name: String,
    mode: String,
    owner_id: u64, 
    group_id: u64,
    size: u64, // Should always be 0
    time: String,
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
    pub fn from_block(block: Vec<u8>, block_id: u64) -> Directory {
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
        let mut time = String::with_capacity(11);
        // Skip over the null byte - storing this as a string because we don't care how it works
        for i in 136..147 {
            let chr = match block.get(i) {
                Some(chr) => *chr as char,
                None => '\0',
            };
            time.push(chr);      
        }
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
            write: false,

            subdirectories: subdirectories,
            contents: files,
        }
    }

    pub fn reinit_from_block(&mut self, block: Vec<u8>, block_id: u64) {
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
        let mut time = String::with_capacity(11);
        // Skip over the null byte - storing this as a string because we don't care how it works
        for i in 136..147 {
            let chr = match block.get(i) {
                Some(chr) => *chr as char,
                None => '\0',
            };
            time.push(chr);      
        }
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
        self.name = name;
        self.mode = mode;
        self.owner_id = owner_id;
        self.group_id = group_id;
        self.time = time;
        self.size = size;
        self.checksum = header;
        self.type_flag = type_flag;
        self.linked_name = linked_name;
        self.owner_name = owner_name;
        self.group_name = group_name;
        self.device_minor_number = device_minor_number;
        self.device_major_number = device_major_number;
        self.prefix = filename_prefix;
        self.block_id = block_id;
        self.write = false;
    }

    pub fn to_block(&mut self) -> Vec<u8> {
        let mut block = Vec::with_capacity(512);

        // Filename
        block.extend(unsafe { self.name.as_mut_vec().iter().cloned() } );

        // Mode - 0000777\0
        let mut mode = vec![48u8, 48u8, 48u8, 48u8, 55u8, 55u8, 55u8, 0u8];
        block.append(&mut mode);

        // Owner and group ID - 0000420\0
        let mut id = vec![48u8, 48u8, 48u8, 48u8, 52u8, 50u8, 48u8, 0u8];
        block.append(&mut id);
        let mut id = vec![48u8, 48u8, 48u8, 48u8, 52u8, 50u8, 48u8, 0u8];
        block.append(&mut id);

        // Size (octal numbers)
        block.push(48);
        let size = format!("{:o}", self.size);
        let mut size = size.into_bytes();
        size.reverse(); 
        for i in (0..10).rev() {
            let chr = match size.get(i) {
                Some(chr) => *chr,
                None => 48,
            };
            block.push(chr);
        }
        block.push(0);

        // Time (string)
        block.extend(unsafe { self.time.as_mut_vec().iter().cloned() } );
        block.push(0);


        // Checksum (octal numbers)
        let checksum = format!("{:o}", self.checksum);
        let mut checksum = checksum.into_bytes();
        checksum.reverse(); 
        for i in (0..6).rev() {
            let chr = match checksum.get(i) {
                Some(chr) => *chr,
                None => 48,
            };
            block.push(chr);
        }
        block.push(0);
        block.push(32);


        // Type - 0 for file, 5 for folder
        block.push(b'5'); 

        // Linked name - we don't support links, so don't care about this - supposed to be 0
        for i in 0..100 {
            block.push(0);
        }
        // Ustar indicators
        let mut ustar = vec![b'u', b's', b't', b'a', b'r', 0u8];
        block.append(&mut ustar);
        // 00
        block.push(48);
        block.push(48);

        // user name
        block.extend(unsafe { self.owner_name.as_mut_vec().iter().cloned() } );
        // Group name
        block.extend(unsafe { self.group_name.as_mut_vec().iter().cloned() } );

        // Device major and minor number - 0000000\0
        let mut num = vec![48u8, 48u8, 48u8, 48u8, 48u8, 48u8, 48u8, 0u8];
        block.append(&mut num);
        let mut num = vec![48u8, 48u8, 48u8, 48u8, 48u8, 48u8, 48u8, 0u8];
        block.append(&mut num);

        block.extend(unsafe { self.prefix.as_mut_vec().iter().cloned() } );

        for i in 0..12 {
            block.push(0);
        }

        // Regenerate the checksum - otherwise archivemount sees it as corrupted
        let mut header = 0;
        for (i, n) in block.iter().enumerate() {
            if i > 147 && i < 155 {
                header += 32;
            }
            else {
                header += *n as u64;                
            }
        }

        self.checksum = header; 
        let checksum = format!("{:o}", header);
        
        let mut checksum = checksum.into_bytes();
        checksum.reverse(); 
        for i in (0..6).rev() {
            let chr = match checksum.get(i) {
                Some(chr) => *chr,
                None => 48,
            };
            block[(6 - i) + 147] =  chr;
        }

        block
    }

}

impl USTARItem for Directory {
    fn get_name(&self) -> String {
        return self.name.clone();
    }

    fn set_name(&mut self, name: String) {
        self.name = name.clone();
        self.name.reserve_exact(100 - self.name.len());
        while self.name.len() < 100 {
            self.name.push('\0');
        }
    }

    fn get_name_and_prefix(&self) -> String {
        return format!("{}{}", self.name, self.prefix);
    }

    fn set_prefix(&mut self, prefix: String) {
        self.prefix = prefix.clone();
        self.prefix.reserve_exact(100 - self.prefix.len());
        while self.prefix.len() < 100 {
            self.prefix.push('\0');
        }
    }

    /*fn set_name_and_prefix(&mut self, combined: String) {
        self.name = name.clone();
        self.name.reserve_exact(100 - self.name.len());
        while self.name.len() < 100 {
            self.name.push('\0');
        }
    }*/

    fn should_write(&mut self) {
        self.write = true;
    }

    fn get_should_write(&self) -> bool {
        self.write
    }

    fn get_writable_representation(&mut self) -> Vec<u8> {
        self.to_block()
    }

    fn get_block_id(&self) -> u64 {
        self.block_id
    }

    fn set_block_id(&mut self, block_id: u64) {
        self.block_id = block_id;
    }
}

pub struct File {
    data: Vec<u8>,

    //Additional needed stuff not from the USTAR filesystem.
    // What block is this hosted on? (we need this for writing to disk)
    block_id: u64,
    // Should we write this entire file to disk?
    write: bool,

    // Stuff needed by the USTAR filesystem
    // https://wiki.osdev.org/USTAR
    pub name: String,
    mode: String,
    owner_id: u64, 
    group_id: u64,
    size: u64,
    time: String,
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
        let mut time = String::with_capacity(11);
        // Skip over the null byte - storing this as a string because we don't care how it works
        for i in 136..147 {
            let chr = match block.get(i) {
                Some(chr) => *chr as char,
                None => '\0',
            };
            time.push(chr);      
        }
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
            write: false,

            data: data,
        }
    }

    pub fn to_block(&mut self) -> Vec<u8> {
        let mut block = Vec::with_capacity(512);

        // Filename
        block.extend(unsafe { self.name.as_mut_vec().iter().cloned() } );

        // Mode - 0000777\0
        let mut mode = vec![48u8, 48u8, 48u8, 48u8, 55u8, 55u8, 55u8, 0u8];
        block.append(&mut mode);

        // Owner and group ID - 0000420\0
        let mut id = vec![48u8, 48u8, 48u8, 48u8, 52u8, 50u8, 48u8, 0u8];
        block.append(&mut id);
        let mut id = vec![48u8, 48u8, 48u8, 48u8, 52u8, 50u8, 48u8, 0u8];
        block.append(&mut id);

        // Size (octal numbers)
        block.push(48);
        let size = format!("{:o}", self.size);
        let mut size = size.into_bytes();
        size.reverse(); 
        for i in (0..10).rev() {
            let chr = match size.get(i) {
                Some(chr) => *chr,
                None => 48,
            };
            block.push(chr);
        }
        block.push(0);

        // Time (string)
        block.extend(unsafe { self.time.as_mut_vec().iter().cloned() } );
        block.push(0);


        // Checksum (octal numbers)
        let checksum = format!("{:o}", self.checksum);
        let mut checksum = checksum.into_bytes();
        checksum.reverse(); 
        for i in (0..6).rev() {
            let chr = match checksum.get(i) {
                Some(chr) => *chr,
                None => 48,
            };
            block.push(chr);
        }
        block.push(0);
        block.push(32);


        // Type - 0 for file, 5 for folder
        block.push(b'0'); 

        // Linked name - we don't support links, so don't care about this - supposed to be 0
        for i in 0..100 {
            block.push(0);
        }
        // Ustar indicators
        let mut ustar = vec![b'u', b's', b't', b'a', b'r', 0u8];
        block.append(&mut ustar);
        // 00
        block.push(48);
        block.push(48);

        // user name
        block.extend(unsafe { self.owner_name.as_mut_vec().iter().cloned() } );
        // Group name
        block.extend(unsafe { self.group_name.as_mut_vec().iter().cloned() } );

        // Device major and minor number - 0000000\0
        let mut num = vec![48u8, 48u8, 48u8, 48u8, 48u8, 48u8, 48u8, 0u8];
        block.append(&mut num);
        let mut num = vec![48u8, 48u8, 48u8, 48u8, 48u8, 48u8, 48u8, 0u8];
        block.append(&mut num);

        block.extend(unsafe { self.prefix.as_mut_vec().iter().cloned() } );

        for i in 0..12 {
            block.push(0);
        }
        // Regenerate the checksum - otherwise archivemount sees it as corrupted
        let mut header = 0;
        for (i, n) in block.iter().enumerate() {
            if i > 147 && i < 155 {
                header += 32;
            }
            else {
                header += *n as u64;                
            }
        }

        self.checksum = header; 
        let checksum = format!("{:o}", header);
        
        let mut checksum = checksum.into_bytes();
        checksum.reverse(); 
        for i in (0..6).rev() {
            let chr = match checksum.get(i) {
                Some(chr) => *chr,
                None => 48,
            };
            block[(6 - i) + 147] =  chr;
        }

        block
    }

    fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }
    // This handles size 
    fn set_data(&mut self, data: Vec<u8>) {
        self.data = data.clone();
        self.size = data.len() as u64;
    }
}

impl USTARItem for File {
    fn get_name(&self) -> String {
        return self.name.clone();
    }

    fn set_name(&mut self, name: String) {
        self.name = name.clone();
        self.name.reserve_exact(100 - self.name.len());
        while self.name.len() < 100 {
            self.name.push('\0');
        }
    }

    fn get_name_and_prefix(&self) -> String {
        return format!("{}{}", self.name, self.prefix);
    }

    fn set_prefix(&mut self, prefix: String) {
        self.prefix = prefix.clone();
        self.prefix.reserve_exact(100 - self.prefix.len());
        while self.prefix.len() < 100 {
            self.prefix.push('\0');
        }
    }

    /*fn set_name_and_prefix(&mut self, combined: String) {
        self.name = name.clone();
        self.name.reserve_exact(100 - self.name.len());
        while self.name.len() < 100 {
            self.name.push('\0');
        }
    }*/

    fn should_write(&mut self) {
        self.write = true;
    }

    fn get_should_write(&self) -> bool {
        self.write
    }

    fn get_writable_representation(&mut self) -> Vec<u8> {
        let mut res = Vec::with_capacity((512 + self.size + 512 - (self.size % 512)) as usize);
        res.extend(self.to_block());
        res.extend(self.data.clone());
        for i in 0..(512 - (self.size % 512)) {
            res.push(0);
        }
        res
    }

    fn get_block_id(&self) -> u64 {
        self.block_id
    }

    fn set_block_id(&mut self, block_id: u64) {
        self.block_id = block_id;
    }
}

struct USTARFileSystem {
    block_driver: AtaPio,
    files: Vec<Rc<RefCell<USTARItem>>>,
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
        // First mainly process directories to build the structure of the VFS, then place files in it
        // Initialize (read the data) for the files in the 2nd pass
        // Well actually, do this differently
        // Process any directories that are relative to root 
        // Basically keep on going through, check if a path can be accessed (and thus subdirectories can be added)
        // Can fill the files array early - probably on first run
        // Actually another thing we need to keep track of 
        // There might be a world where we can create directories not backed by disk - this makes sense - fill in the 
        // Disk info as we get it - just add a method to the directory that lets us mutate it based on the entry if found
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
