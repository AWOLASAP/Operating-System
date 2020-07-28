use x86_64::instructions::port::Port;

enum FloppyRegisters {
    STATUS_REGISTER_A                = 0x3F0, // read-only
    STATUS_REGISTER_B                = 0x3F1, // read-only
    DIGITAL_OUTPUT_REGISTER          = 0x3F2,
    TAPE_DRIVE_REGISTER              = 0x3F3,
    MAIN_STATUS_REGISTER             = 0x3F4, // read-only
    // Commands, parameter information, result codes, and disk data
    DATA_FIFO                        = 0x3F5,
    DIGITAL_INPUT_REGISTER           = 0x3F7, // read-only
}

//Rust gets sad when we define the same value in an enum
impl FloppyRegisters {
    pub const DATARATE_SELECT_REGISTER: FloppyRegisters         = FloppyRegisters::MAIN_STATUS_REGISTER; // write-only
    pub const CONFIGURATION_CONTROL_REGISTER: FloppyRegisters   = FloppyRegisters::DIGITAL_INPUT_REGISTER;  // write-only
}

const FLOPPY_SECTORS_PER_TRACK: u32 = 18;


// Returns cylinder, head, and sector
fn lba_to_chs(lba: u32) -> (u32, u32, u32) {
    return (lba / (2 * FLOPPY_SECTORS_PER_TRACK), ((lba % (2 * FLOPPY_SECTORS_PER_TRACK)) / FLOPPY_SECTORS_PER_TRACK), ((lba % (2 * FLOPPY_SECTORS_PER_TRACK)) % FLOPPY_SECTORS_PER_TRACK + 1));
}

// Writes a byte to the command buffer
fn write_cmd(cmd: u8) {
    
}

fn reset_controller() {
    let mut fifo = Port::new(FloppyRegisters::DATA_FIFO as u16);
    // Configure A suggestion would be: drive polling mode off, FIFO on, threshold = 8, implied seek on, precompensation 0. 

    // Lock
    loop {
        let msr = read_msr();
        let mut lock_state = true;
        if msr & 0x80 == 0x80 && msr & 0x40 == 0 {
            // Write command code
            unsafe { fifo.write(20u8); }
            // Read result bit
            while read_msr() & 0x80 == 0 {
                if read_msr() & 0x40 == 0x40 {
                    unsafe { fifo.write(0); }
                    loop {
                        let result = unsafe { fifo.read() };
                        if result != 1 << 4 {
                            lock_state = false;
                        }
                        if read_msr() & 0x80 == 0x80 || !(read_msr() & 0x10 == 0x10 && read_msr() & 0x40 == 0x40) {
                            break;
                        }
                    }
                }
            }
            // Final command check
            if read_msr() & 0x80 != 0x80 || (read_msr() & 0x10 == 0x10 || read_msr() & 0x40 == 0x40) {
                lock_state = false;
            }
            if lock_state {
                break;
            }
        }
    }
    // Use DOR to reset 
    let dor_definitions: u8 = 4;
    let mut dor = Port::new(FloppyRegisters::DIGITAL_OUTPUT_REGISTER as u16);
    let old_dor = unsafe { dor.read() };
    unsafe { dor.write(dor_definitions); }
    unsafe { dor.write(old_dor); }

    // Spam interrupts 
}

fn read_msr() -> u8 {
    let mut msr = Port::new(FloppyRegisters::MAIN_STATUS_REGISTER as u16);
    unsafe { msr.read() }
}

// Starts up the driver
pub fn init() {
    reset_controller();
    select_drive(0);

}

// Initializes 

// RN, only support for drive 0 is implemented. This might change.
pub fn select_drive(drive: u8) {
    let mut fifo = Port::new(FloppyRegisters::DATA_FIFO as u16);
    // Sets datarate
    let mut ccr = Port::new(FloppyRegisters::CONFIGURATION_CONTROL_REGISTER as u16);
    let mut dsr = Port::new(FloppyRegisters::DATARATE_SELECT_REGISTER as u16);
    unsafe {
        ccr.write(3u8);
        dsr.write(3u8);
    }
    // Specify command
    loop {
        let msr = read_msr();
        if msr & 0x80 == 0x80 && msr & 0x40 == 0 {
            unsafe { fifo.write(3u8); }
            while read_msr() & 0x80 == 0 {
                if read_msr() & 0x40 == 0 {
                    unsafe { fifo.write(0); }
                }
            }
            break;
        }
    }
    // Write DOR
    let dor_definitions: u8 = 0x10 | 4 | drive;
    let mut dor = Port::new(FloppyRegisters::DIGITAL_OUTPUT_REGISTER as u16);
    unsafe { dor.write(dor_definitions); }
    // Lock command 

    // Now we configure the drive - the driver assumes the supplied drive is 2.88M

    // Perpendicular mode
}