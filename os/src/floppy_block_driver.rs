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

bitflags! {
    struct MsrStatus: u8{
        const RQM = 0x80;
        const DIO = 0x40;
        const NDMA = 0x20;
        const CB = 0x10;
        const ACTD = 8;
        const ACTC = 4;
        const ACTB = 2;
        const ACTA = 1;
    }
}

impl MsrStatus {
    pub fn set_bits(&mut self, bits: u8) {
        self.bits = bits;
    }
}



// Returns cylinder, head, and sector
fn lba_to_chs(lba: u32) -> (u32, u32, u32) {
    return (lba / (2 * FLOPPY_SECTORS_PER_TRACK), ((lba % (2 * FLOPPY_SECTORS_PER_TRACK)) / FLOPPY_SECTORS_PER_TRACK), ((lba % (2 * FLOPPY_SECTORS_PER_TRACK)) % FLOPPY_SECTORS_PER_TRACK + 1));
}

// Starts up the driver
pub fn init() {
    select_drive(0);
    let mut ccr = Port::new(FloppyRegisters::CONFIGURATION_CONTROL_REGISTER as u16);
    let mut dsr = Port::new(FloppyRegisters::DATARATE_SELECT_REGISTER as u16);
    unsafe {
        ccr.write(3u8);
        dsr.write(3u8);
    }
}

// RN, only support for drive 0 is implemented. This might change.
pub fn select_drive(drive: u8) {
    let dor_definitions: u8 = 0x10 | 4 | drive;
    let mut dor = Port::new(FloppyRegisters::DIGITAL_OUTPUT_REGISTER as u16);
    unsafe { dor.write(dor_definitions); }
}