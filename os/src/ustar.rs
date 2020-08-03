use lazy_static::lazy_static;
use spin::{Mutex};
use crate::ata_block_driver::AtaPio;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use hashbrown::HashMap;
use core::option::Option;
use alloc::sync::Arc;
use alloc::sync::Weak;
use core::u64;
use alloc::format;
use crate::alloc::string::ToString;
use crate::println;

// Note to me tomorrow - we're going to use Arc<Mutex<File>> and Directory because
// It gives me interior mutability (Mutex), and shared ownership (Arc). This is important
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
    fn get_should_write(&mut self) -> bool;

    // Get writable representation - this is how the driver actually applies changes to disk
    // Driver should auto handle writing the vector/using the right number of sectors, but 
    // care should still be made to making it a correct multiple
    fn get_writable_representation(&mut self) -> Vec<u8>;
    

    // Gets and sets the block ID - do this RARELY, probably only during initialization of the
    // file and defragmenting - otherwise you could lose the file and corrupt the disk
    fn get_block_id(&self) -> u64;
    fn set_block_id(&mut self, block_id: u64);

    // Gets the size - important for writing 
    fn get_size(&self) -> u64;
}

pub struct Directory {
    // Directory specific stuff
    contents: Vec<Arc<Mutex<File>>>,
    subdirectories: Vec<Arc<Mutex<Directory>>>,
    // Not sure if gonna add, but this could be cool 
    // Makes cd .. much easier.
    parent: Weak<Mutex<Directory>>,

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
    // For when you're making a new directory going in disk
    // Where does it go, and what is it's name (must include parent folders too)
    fn new(block_id: u64, name: String) -> Directory {
        // Handle name
        let name_variable = name.into_bytes();
        let mut name = String::with_capacity(100);
        for i in 0..100 {
            match name_variable.get(i) {
                Some(chr) => name.push(*chr as char),
                None => (),
            };
            
        }
        // Mode
        let mode = String::from("100777");
        // User and group ID
        let owner_id = 420;
        let group_id = 420;
        // Size
        let size =0;
        let mut time = String::with_capacity(11);
        // Skip over the null byte - storing this as a string because we don't care how it works
        for i in 136..147 {
            time.push('0');      
        }
        // Header checksum
        let mut header = 0;
        // You get to be headerless for a bit - until someone tries to write
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
            filename_prefix.push('\0');
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
            parent: Weak::new(),
        }
        
    }

    // For when you need a new directory not backed by disk.
    fn new_directory(name: String) -> Directory {
        // Mode
        let mode = String::from("100777");
        // User and group ID
        let owner_id = 420;
        let group_id = 420;
        // Size
        let size = 0;
        // Time
        let time = String::from("");
        let header = 0;
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
            filename_prefix.push('\0');
        }
        // Setup directory specific Variables
        let subdirectories = Vec::new();
        let files = Vec::new();

        // block_id of  u64::MAX signals that we don't want it to exist on disk
        let block_id = u64::MAX;
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
            parent: Weak::new(),
        }
    }

    // Only should be used for initialization from the disk
    pub fn from_block(block: Vec<u8>, block_id: u64) -> Directory {
        // Handle name
        let mut name = String::with_capacity(100);
        for i in 0..100 {
            let chr = match block.get(i) {
                Some(chr) => *chr as char,
                None => '\0',
            };
            if chr != '\0' {
                name.push(chr);
            }
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
            parent: Weak::new(),
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
            if chr != '\0' {
                name.push(chr);
            }
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
        // Made it so that white space is not part of the representation of a file
        while block.len() < 100 {
            block.push(0);
        }

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


    fn get_short_name(&self) -> String {
        let possibilities = self.name.split('/');
        let mut result = "";
        for i in possibilities {
            if i.len() > 0 {
                result = i;
            }
        }
        let mut result = result.to_string();
        result.push('/');
        result
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


    fn get_should_write(&mut self) -> bool {
        if self.write {
            self.write = false;
            return true;
        }
        else {
            return false;
        }
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

    fn get_size(&self) -> u64 {
        self.size
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
    // For when you're making a new file going in disk
    // Where does it go, and what is it's name (must include parent folders too)
    fn new(block_id: u64, name: String) -> File {
        // Handle name
        let name_variable = name.into_bytes();
        let mut name = String::with_capacity(100);
        for i in 0..100 {
            match name_variable.get(i) {
                Some(chr) => name.push(*chr as char),
                None => (),
            };
            
        }
        // Mode
        let mode = String::from("100777");
        // User and group ID
        let owner_id = 420;
        let group_id = 420;
        // Size
        let size =0;
        let mut time = String::with_capacity(11);
        // Skip over the null byte - storing this as a string because we don't care how it works
        for i in 136..147 {
            time.push('0');      
        }
        // Header checksum
        let mut header = 0;
        // You get to be headerless for a bit - until someone tries to write
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
            filename_prefix.push('\0');
        }
        // Setup directory specific Variables
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



    pub fn from_block(block: Vec<u8>, block_id: u64) -> File {
        // Handle name
        let mut name = String::with_capacity(100);
        for i in 0..100 {
            let chr = match block.get(i) {
                Some(chr) => *chr as char,
                None => '\0',
            };
            if chr != '\0' {
                name.push(chr);
            }
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
        // Made it so that white space is not part of the representation of a file
        while block.len() < 100 {
            block.push(0);
        }

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

    fn get_short_name(&self) -> String {
        let possibilities = self.name.split('/');
        let mut result = "";
        for i in possibilities {
            result = i;
        }
        result.to_string()
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

    fn get_should_write(&mut self) -> bool {
        if self.write {
            self.write = false;
            return true;
        }
        else {
            return false;
        }
    }

    fn get_writable_representation(&mut self) -> Vec<u8> {
        let mut size = self.size + 512;
        if size % 512 == 0 {
            size /= 512; 
        }
        else {
            size = (size - size % 512) / 512 + 1; 
        }
        size *= 512;
        let mut res = Vec::with_capacity(size as usize);
        res.extend(self.to_block());
        res.extend(self.data.clone());
        while res.capacity() != res.len() {
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

    fn get_size(&self) -> u64 {
        self.size
    }
}

pub struct USTARFileSystem {
    block_driver: AtaPio,
    files: Vec<Arc<Mutex<dyn USTARItem + Send + Sync>>>,
    current_dirs: HashMap<u64, Arc<Mutex<Directory>>>,
    current_dirs_tracker: u64,
    root: Arc<Mutex<Directory>>,
    block_used_ptr: u64,
}

impl USTARFileSystem {
    fn new() -> USTARFileSystem {
        let driver = AtaPio::try_new();
        let files = Vec::new();
        let current_dirs = HashMap::new();
        let root = Arc::new(Mutex::new(Directory::new_directory("/".to_string())));
        root.lock().parent = Arc::downgrade(&root);

        USTARFileSystem {
            block_driver: driver,
            files: files,
            current_dirs: current_dirs,
            current_dirs_tracker: 1,
            root: Arc::new(Mutex::new(Directory::new_directory("/".to_string()))),
            block_used_ptr: 0,
        }
    }

    
    pub fn init(&mut self) {
        unsafe {
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
            // End planning comment block

            // Main file acquiescence loop
            let mut end = false;
            let mut counter: u32 = 0;
            while !end {
                let block = self.block_driver.read_lba(counter, 1);

                if self.check_magic_value(&block) {
                    let type_flag = self.get_typeflag_(&block);
                    if type_flag == 0 {
                        // Most of this handles the size of the file
                        let mut file = File::from_block(block, counter as u64);
                        counter += 1;
                        let mut size = file.size;

                        if size % 512 == 0 {
                            size = size / 512; 
                        }
                        else {
                            size = (size - size % 512) / 512 + 1; 
                        }
                        let size_orig = size;
                        // This code let us read more than one block in at a time, but it was causing corruption/errors, which I do not like
                        /*
                        let mut blocks_written = 0;
                        if size % 255 == 0 {
                            blocks_written = size / 255; 
                        }
                        else {
                            blocks_written = (size - size % 255) / 255 + 1; 
                        }
                        */
                        let mut data = Vec::with_capacity(size_orig as usize);

                        for i in 0..size {
                            data.append(&mut self.block_driver.read_lba(counter, 1));
                            counter = counter + 1;
                        }
                        /*
                        for i in 0..blocks_written {
                            if size >= 255 {
                                data.append(&mut self.block_driver.read_lba(counter, 1));
                                size -= 1;
                            }
                            else {
                                data.append(&mut self.block_driver.read_lba(counter, size as u8));
                            }
                        }
                        counter += size_orig as u32;
                        */
                        data.truncate(file.size as usize);
                        file.set_data(data);
                        // Should handle things like generating the directory structure and putting it in the block vector
                        if file.name != "defrag" {
                            self.place_file_in_vfs(file);
                        }
                    }
                    else if type_flag == 5 {
                        let mut folder = Directory::from_block(block, counter as u64);
                        counter += 1;
                        if folder.name != "defrag" {
                            self.place_folder_in_vfs(folder);
                        }
                    }
                    else {
                        // Unsupported type - hope that it's only one block
                        counter += 1;
                    }
                }
                else {
                    end = true;
                }

            }
            self.block_used_ptr = counter as u64;
            // Somehow sort the thing
        }
    }

    fn check_magic_value(&self, block: &Vec<u8>) -> bool {
        let val = [b'u', b's', b't', b'a', b'r', 0, b'0', b'0'];
        let mut magic = true;
        for i in 257..265 {
            if (*block)[i] == val[i - 257] {
                
            }
            else {
                magic = false;
            }
        }
        magic
    }

    fn get_typeflag_(&self, block: &Vec<u8>) -> u8 {
        (*block)[156] - 48
    }

    fn split_path(&self, path: &str) -> Vec<String> {
        let mut result =  Vec::new();
        for i in (*path).split('/') {
            result.push(i.to_string());
        }
        if result[result.len() - 1].is_empty() {
            result.pop();
        }
        if !result.is_empty() && result[0].is_empty() {
            result.remove(0);
        }

        result
    }

    fn place_file_in_vfs(&mut self, file: File) {
        let parent_dir = self.generate_path_if_does_not_exist(&(file.name));
        let file = Arc::new(Mutex::new(file));
        parent_dir.lock().contents.push(Arc::clone(&file));
        self.files.push(file);
    }

    fn place_folder_in_vfs(&mut self, mut folder: Directory) {
       let parent_dir = self.generate_path_if_does_not_exist(&(folder.name));
       // Check if subfolder exists - if so, update it instead of replacing it 
       let mut parent = parent_dir.lock();
       for i in parent.subdirectories.iter() {
           if folder.name == i.lock().name {
               i.lock().reinit_from_block(folder.to_block(), folder.block_id);
               let result = Arc::clone(i);
               self.files.push(result);
               return
           }
       }
       folder.parent = Arc::downgrade(&parent_dir);
       let folder = Arc::new(Mutex::new(folder));
       parent.subdirectories.push(Arc::clone(&folder));
       self.files.push(folder);
    }

    // Returns the parent directory of the given path. Will always return the parent directory. Even if it doesn't exist (because it creates it)

    fn generate_path_if_does_not_exist(&mut self, path: &String) -> Arc<Mutex<Directory>> {
        let mut decomposed = self.split_path(&path);
        decomposed.pop();
        let mut current_dir = Arc::clone(&self.root);
        for i in decomposed.iter() {
            //Check if child directory exists
            let mut success = false;

            let current_dir_clone = Arc::clone(&current_dir);
            {
                let subdirs = &current_dir_clone.lock().subdirectories;
            
                for d in subdirs.iter() {
                    let mut child_path = self.split_path(&d.lock().name);
                    let last_item = match child_path.pop() {
                        Some(item) => item,
                        None => "".to_string(),
                    };

                    if *i == last_item {
                        success = true;
                        current_dir = Arc::clone(d);
                        break;
                    }
                }
            }
            if success {
                // Do nothing - we've already moved the current dir
            }
            else {
                let current_dir_clone = Arc::clone(&current_dir);
                let mut current_dir_clone = current_dir_clone.lock();
                let current_name = &current_dir_clone.name;
                let current_name = format!("{}{}/", current_name, *i);
                let mut directory = Directory::new_directory(current_name);
                // Using current_dir because current_dir_clone is unlocked
                directory.parent = Arc::downgrade(&current_dir);
                let directory = Arc::new(Mutex::new(directory));
                let new_directory = Arc::clone(&directory);
                current_dir_clone.subdirectories.push(directory);
                current_dir = new_directory;

                // Do something - create a non-disk backed directory and add it to the current_dir
            }
        }

        current_dir
    }

    pub fn print_root(&self) {
        self.print_folder_recursive(Arc::clone(&self.root));
    }

    fn print_folder_recursive(&self, folder: Arc<Mutex<Directory>>) {
        let folder = folder.lock();
        print!("Printing folder: {}", folder.name);
        if folder.block_id == u64::MAX {
            println!("   Is not backed");
        }
        else {
            println!("   Is backed");
        }
        for i in folder.contents.iter() {
            println!("{}", i.lock().get_short_name());
        }
        for i in folder.subdirectories.iter() {
            self.print_folder_recursive(Arc::clone(i));
        }
    }

    pub fn defragment(&mut self) {
        // Remove all files named defrag than move the rest of the files (blockwise), so that it's still valid USTAR
        self.write();
        let mut counter = 0;
        for i in self.files.iter() {
            let mut item = i.lock();
            if item.get_name() != "defrag" {
                item.set_block_id(counter);
                let mut size = item.get_size();
                if size % 512 == 0 {
                    size /= 512; 
                }
                else {
                    size = (size - size % 512) / 512 + 1; 
                }
                size += 1;
                // Good for debugging
                //println!("Defragging {} with size {}", item.get_name(), size);
                counter += size;
                item.should_write();
            }
            else {

            }
        }
        self.block_used_ptr = counter;
        self.write();
    }
    

    pub fn write(&mut self) {
        //Write any changes
        for i in self.files.iter() {
            let mut item = i.lock();
            if item.get_should_write() {
                let mut data = item.get_writable_representation();
                while data.len() % 512 != 0 {
                    data.push(0);
                }
                let mut size = data.len();
                let mut id = item.get_block_id();
                if size % 512 == 0 {
                    size /= 512; 
                }
                else {
                    size = (size - size % 512) / 512 + 1; 
                }
                let size_orig = size;
                // Good for debugging
                //println!("Writing {} at {} with size {}", item.get_name(), id, size);
                // Each write request/sector
                for i in 0..size {
                    let mut data_to_write = Vec::with_capacity(256);
                    for j in 0..256 {
                        data_to_write.push(((data[i*512 + j*2 + 1] as u16) << 8) | data[i*512 + j*2] as u16); 
                    } 
                    unsafe { self.block_driver.write(id as u32, 1, data_to_write)};
                    id += 1;
                }

            }
        }
        // Write two null 
        unsafe { self.block_driver.write(self.block_used_ptr as u32, 1, Vec::new())};
        unsafe { self.block_driver.write(self.block_used_ptr as u32 + 1, 1, Vec::new())};
    }
    /*
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
    // Maybe a path parser would be a good idea
    */

    pub fn set_all_files_to_write(&mut self) {
        for i in self.files.iter() {
            i.lock().should_write();
        }
    }

    pub fn get_id(&mut self) -> u64 {
        self.current_dirs_tracker += 1;
        self.current_dirs.insert(self.current_dirs_tracker, Arc::clone(&self.root));
        self.current_dirs_tracker
    }

    
    pub fn list_files(&self, id: u64) -> Vec<String> {
        let current_dir = self.current_dirs[&id].lock();
        let mut result = Vec::with_capacity(current_dir.contents.len());
        for i in current_dir.contents.iter() {
            result.push(i.lock().get_short_name());
        }
        result
    }

    pub fn list_subdirectories(&self, id: u64) -> Vec<String> {
        let current_dir = self.current_dirs[&id].lock();
        let mut result = Vec::with_capacity(current_dir.subdirectories.len());
        for i in current_dir.subdirectories.iter() {
            result.push(i.lock().get_short_name());
        }
        result
    }

    // Checks if a file path is relative or absolute
    fn is_absolute(&self, path: &str) -> bool {
        (match path.split('/').next() {Some(val) => val.len(), None => 0} == 0)
    }

    fn resolve_directory_absolute(&self, path: String) -> Option<Arc<Mutex<Directory>>> {
        let mut current_dir = Arc::clone(&self.root);
        for i in self.split_path(&path).iter() {
            let newref = Arc::clone(&current_dir);
            let subdirs = &newref.lock().subdirectories;
            let mut changed = false;
            for d in subdirs.iter() {
                let mut child_path = self.split_path(&d.lock().name);
                let last_item = match child_path.pop() {
                    Some(item) => item,
                    None => "".to_string(),
                };
    
                if *i == last_item {
                    let d_clone = Arc::clone(d);
                    changed = true;
                    current_dir = d_clone;
                    break;
                }
            }         
            if !changed {
                return None;
            }
        }
        Some(current_dir)    
    }

    fn resolve_directory_relative(&self, path: String, id: u64) -> Option<Arc<Mutex<Directory>>> {
        let mut current_dir = Arc::clone(match self.current_dirs.get(&id) {
            Some(current_dir) => current_dir,
            None => &self.root,
        });
        for i in self.split_path(&path).iter() {
            if i == ".." {
                let parent_dir = current_dir.lock().parent.upgrade();
                let parent = match parent_dir {
                    Some(p) => p,
                    None => Arc::clone(&self.root),
                };
                current_dir = parent;
            }
            else if i == "." {
                // Do nothing - this is the same path
            }
            else {
                let newref = Arc::clone(&current_dir);
                let subdirs = &newref.lock().subdirectories;
                let mut changed = false;
                for d in subdirs.iter() {
                    let mut child_path = self.split_path(&d.lock().name);
                    let last_item = match child_path.pop() {
                        Some(item) => item,
                        None => "".to_string(),
                    };
        
                    if *i == last_item {
                        let d_clone = Arc::clone(d);
                        changed = true;
                        current_dir = d_clone;
                        break;
                    }
                }         
                if !changed {
                    return None;
                }
            }
        }
        Some(current_dir)
    }

    fn resolve_directory(&mut self, path: String, id: Option<u64>) -> Option<Arc<Mutex<Directory>>> {
        if self.is_absolute(&path) {
            self.resolve_directory_absolute(path)
        }
        else {
            let id = match id {
                Some(id) => id,
                None => self.get_id(),
            };
            self.resolve_directory_relative(path,id)
        }
    }

    fn split_last_and_first(&self, path: String) -> (String, String) {
        let abs = self.is_absolute(&path);
        let mut split = self.split_path(&path);
        let mut part1 = String::new();
        let name = match split.pop() {
            Some(i) => i,
            None => "".to_string(),
        };
        for i in split.iter() {
            part1.push_str(i);
            part1.push_str("/");
        }
        if abs {
            let mut res = String::from("/");
            res.push_str(&part1);
            return (res, name);

        }
        (part1, name)
    }
    
    fn resolve_file(&mut self, path: String, id: Option<u64>) -> Option<Arc<Mutex<File>>> {
        let (path, file) = self.split_last_and_first(path);
        if self.is_absolute(&path) && !path.is_empty() {
            let parent_dir = match self.resolve_directory_absolute(path) {
                Some(thing) => thing,
                None => return None,
            };
            let parent = parent_dir.lock();
            for i in parent.contents.iter() {
                if i.lock().get_short_name() == file {
                    return Some(Arc::clone(i));
                }
            }
            None
        }
        else {
            let id = match id {
                Some(id) => id,
                None => self.get_id(),
            };
            let parent_dir = match self.resolve_directory_relative(path,id) {
                Some(thing) => thing,
                None => return None,
            };
            let parent = parent_dir.lock();
            for i in parent.contents.iter() {
                if i.lock().get_short_name() == file {
                    return Some(Arc::clone(i));
                }
            }
            None
        }
    }
    
    // Kept for backwards compatability
    pub fn up_directory(&mut self, id: u64) {
        let current_dirs = match self.current_dirs.remove_entry(&id) {
            Some((_, current_dirs)) => current_dirs,
            None => return,
        };
        let current_dir = current_dirs.lock();
        let optional_weak = &current_dir.parent.upgrade();
        let upgraded_pointer = match optional_weak {
            Some(upgrade) => upgrade,
            None => &self.root,
        };
        self.current_dirs.insert(id, Arc::clone(upgraded_pointer));
    }

    pub fn change_directory(&mut self, directory: String, id: u64) -> bool {
        let dir_to_change = self.resolve_directory(directory, Some(id));
        let current_dirs = match self.current_dirs.remove_entry(&id) {
            Some((_, current_dirs)) => current_dirs,
            None => return false,
        };
        match dir_to_change {
            Some(dir) => {
                self.current_dirs.insert(id, Arc::clone(&dir));
                true
            },
            None => {
                self.current_dirs.insert(id, Arc::clone(&current_dirs));
                false
            },
        }
    }
    
    // If a file doesn't exist, returns None
    pub fn read_file(&mut self, file: String, id: Option<u64>) -> Option<Vec<u8>> {
        let file = self.resolve_file(file, id);
        let file_data = match file {
            Some(file) => return Some(file.lock().get_data()),
            None => return None,
        };
    }
    
    // If a file doesn't exist, running this function will create it
    // Doesn't append to the data, but flat out replaces it - changes in allocation need to defrag
    // Does not account for if you write nothing, you're on your own
    pub fn write_file(&mut self, file: String, data: Vec<u8>, id: Option<u64>) {
        let file_string = file.to_string();
        let file = self.resolve_file(file, id);
        let mut file_data = match file {
            Some(file) => {
                let mut new_file = File::from_block(file.lock().to_block(), self.block_used_ptr);
                self.remove_file(file_string, id);
                new_file
            },
            None => {
                let mut new_file = File::new(self.block_used_ptr, file_string);
                new_file
            },
        };
        let mut size = data.len();
        if size % 512 == 0 {
            size = size / 512; 
        }
        else {
            size = (size - size % 512) / 512 + 1; 
        }
        self.block_used_ptr += size as u64 + 1;
        file_data.set_data(data);
        file_data.should_write();
        self.place_file_in_vfs(file_data);
        self.write();
    }
    /*
    pub fn move_file(&self, file: String, data: Vec<u8>, id: u64) {

    }

    pub fn copy_file(&self, file: String, data: Vec<u8>, id: u64) {

    }

    pub fn rename_file(&self, file: String, data: Vec<u8>, id: u64) {

    }
    */
    // Removes a file if it exists, does nothing if it doesn't
    pub fn remove_file(&mut self, file: String, id: Option<u64>) {
        if let Some(file) =  self.resolve_file(file, id) {
            /*let (first, last) = self.split_last_and_first(file.lock().name.to_string());
            if let Some(directory) = self.resolve_directory( first, id) {
                file.lock().name = "defrag".to_string();
                file.lock().should_write();
                println!("{}", file.lock().size);
                let contents =  &mut directory.lock().contents;
                for (i, d) in contents.iter().enumerate() {
                    if d.lock().name == "defrag" {
                        contents.remove(i);
                        break;
                    }
                }
                self.write();
            }*/
            file.lock().name = "defrag".to_string();
            file.lock().should_write();
            println!("{}", file.lock().get_writable_representation().len());
            self.write();
        }
    }
    /*
    pub fn remove_file_absolute_path(&self, path: String) {
        
    }

    // Probably should use directory moving to implement this
    // If it doesn't exist, nothing happens
    pub fn rename_directory(&self, dir: String, new_name: String, id: u64) -> bool {

    }

    // 2nd parameter should not be complete path, but only the new name
    pub fn rename_directory_absolute_path(&self, path: String, new_name: String) -> bool {
        
    }
    */
    // Creates a directory unless there exists a file or directory with a similar name
    pub fn create_directory(&mut self, file: String, id: u64) -> bool {
        if self.is_absolute(&file) {
            return self.create_directory_absolute_path(file);
        }
        let mut file = file.replace("/", "");
        file.push('/');
        let current_dir_arc = &self.current_dirs[&id];
        let mut current_dir = current_dir_arc.lock();
        for i in current_dir.contents.iter() {
            if i.lock().get_short_name() == file && i.lock().block_id != u64::MAX {
                return false;
            }
        }
        for i in current_dir.subdirectories.iter() {
            if i.lock().get_short_name() == file {
                return false;
            }
        }
        // Check if subfolder exists - if so, update it instead of replacing it 
        let mut folder = Directory::new(self.block_used_ptr, format!("{}{}", current_dir.name, file));
        for i in current_dir.subdirectories.iter() {
            if folder.name == i.lock().name {
                i.lock().reinit_from_block(folder.to_block(), folder.block_id);
                let result = Arc::clone(i);
                self.files.push(result);
                return true;
            }
        }
        folder.should_write();
        folder.parent = Arc::downgrade(&current_dir_arc);
        let folder = Arc::new(Mutex::new(folder));
        current_dir.subdirectories.push(Arc::clone(&folder));
        self.files.push(folder);
        drop(current_dir);
        self.block_used_ptr += 1;
        self.write();
        true
    }
    
    pub fn create_directory_absolute_path(&mut self, path: String) -> bool {
        let id = self.get_id();
        let dir = self.generate_path_if_does_not_exist(&path);
        self.current_dirs.remove_entry(&id);
        self.current_dirs.insert(id, dir);
        let mut string = self.split_path(&path);
        let name = match string.pop() {
            Some(i) => i,
            None => "".to_string(),
        };
        self.create_directory(name, id)    
    }
    
    fn remove_directory_recursive(&mut self, folder: Arc<Mutex<Directory>>) {
        let mut folder = folder.lock();
        folder.name = "defrag".to_string();
        folder.should_write();
        for i in folder.contents.iter() {
            i.lock().name = "defrag".to_string();
            i.lock().should_write();
        }
        for i in folder.subdirectories.iter() {
            println!("{}", i.lock().name);
            self.remove_directory_recursive(Arc::clone(i));
        }
    }

    // Removes a directory if it exists, does nothing if it doesn't
    pub fn remove_directory(&mut self, file: String, id: Option<u64>) {
        if let Some(dir) =  self.resolve_directory(file, id) {
            dir.lock().name = "defrag".to_string();
            dir.lock().should_write();
            let upgraded = dir.lock().parent.upgrade();
            if let Some(parent) = upgraded {
                let subdirs =  &mut parent.lock().subdirectories;
                for (i, d) in subdirs.iter().enumerate() {
                    if d.lock().name == "defrag" {
                        subdirs.remove(i);
                        break;
                    }
                }
            }
            self.remove_directory_recursive(dir);
            self.write();
        }
    }

    // 2nd parameter should be complete path, but not the first one (first one should be relative)
    // If 1st param doesn't exist, or 2nd param already exists, it won't do anything
    // TODO: Later
    pub fn move_directory(&mut self, dir: String, new_path: String, id: Option<u64>) -> bool {
        let dir = self.resolve_directory(dir, id);
        let mut parent_move_dir = self.resolve_directory(new_path.to_string(), id);
        if Option::is_none(&dir)  {
            return false;
        }
        if self.is_absolute(&new_path) {
            let (last, first) = self.split_last_and_first(new_path);
            parent_move_dir = self.resolve_directory(last, id);
        }
        else if self.split_path(&new_path).len() == 1 {
            // If length is one, it's a single path thing
        }
        else {
            let (last, first) = self.split_last_and_first(new_path);
            parent_move_dir = self.resolve_directory(last, id);
            // Length more, which means it's a "compound" path
        }


        true
    }

    pub fn copy_directory(&mut self, path: String, new_path: String) {

    }

    pub fn rename_directory(&mut self, path: String, new_path: String) {

    }

}

lazy_static! {
    pub static ref USTARFS: Mutex<USTARFileSystem> = {
        Mutex::new(USTARFileSystem::new())
    };
}