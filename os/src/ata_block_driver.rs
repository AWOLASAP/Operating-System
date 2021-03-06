/*
Copyright (c) 2017 Hannes Karppila

Permission is hereby granted, free of charge, to any person obtaining
a copy of this software and associated documentation files (the
"Software"), to deal in the Software without restriction, including
without limitation the rights to use, copy, modify, merge, publish,
distribute, sublicense, and/or sell copies of the Software, and to
permit persons to whom the Software is furnished to do so, subject to
the following conditions:

The above copyright notice and this permission notice shall be included
in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/
// I used most of this code from https://github.com/Dentosal/rust_os
// However, I added write support to the ATA_PIO driver

#![allow(dead_code)]

use alloc::vec::Vec;
use cpuio::UnsafePort;

const SECTOR_SIZE: usize = 0x200;

const PORT_DATA: u16 = 0x1F0;
const PORT_SECCOUNT: u16 = 0x1F2;
const PORT_LBA0: u16 = 0x1F3;
const PORT_LBA1: u16 = 0x1F4;
const PORT_LBA2: u16 = 0x1F5;
const PORT_LBA3: u16 = 0x1F6;
const PORT_COMMAND: u16 = 0x1F7;
const PORT_DEV_CTRL: u16 = 0x3F6;

#[derive(Debug, Clone)]
pub struct DriveProperties {
    lba28_sectors: u32,
    lba48_sectors: Option<u64>,
}
impl DriveProperties {
    fn supports_lba48(&self) -> bool {
        self.lba48_sectors.is_some()
    }

    fn sector_count(&self) -> u64 {
        self.lba48_sectors.unwrap_or(self.lba28_sectors as u64)
    }
}

pub struct AtaPio {
    properties: DriveProperties,
}
impl AtaPio {
    pub fn try_new() -> AtaPio  {
        unsafe {
            Self::check_floating_bus();
            Self::reset_drives();
        }
        let properties = unsafe { Self::identify() };

        AtaPio { properties }
    }

    #[inline]
    unsafe fn send_command(cmd: u8) {
        let mut cmd_port = UnsafePort::<u8>::new(PORT_COMMAND);
        cmd_port.write(cmd);
    }

    #[inline]
    unsafe fn read_status() -> u8 {
        let mut status_port = UnsafePort::<u8>::new(PORT_COMMAND);
        status_port.read()
    }

    unsafe fn reset_drives() {
        // https://wiki.osdev.org/ATA_PIO_Mode#Resetting_a_drive_.2F_Software_Reset

        // TODO: currently using (primary bus, master drive) only

        let mut ctrl = UnsafePort::<u8>::new(PORT_DEV_CTRL);

        // Disable interrupts, run software reset
        ctrl.write(0);

        // Wait for BSY to be clear and RDY set
        for _ in 0..4 {
            // 400ns delay
            let _ = ctrl.read();
        }

        loop {
            let v = ctrl.read();
            if (v & 0xc0) == 0x40 {
                // BSY clear, RDY set?
                break;
            }
        }
    }

    unsafe fn select_drive() {
        // https://wiki.osdev.org/ATA_PIO_Mode#400ns_delays

        Self::reset_drives(); // HACK: selects master drive
    }

    unsafe fn check_floating_bus() {
        let data: u8 = Self::read_status();
        if data == 0xFF {
            panic!("No ATA drives attached.");
        }
    }

    /// Polls ATA controller to see if the drive is ready
    #[inline]
    unsafe fn is_ready() -> bool {
        for _ in 0..4 {
            let _ = Self::read_status();
        }
        let data: u8 = Self::read_status();
        (data & 0xc0) == 0x40 // BSY clear, RDY set?
    }

    /// Polls ATA controller to until the drive is ready
    unsafe fn wait_ready() {
        while !Self::is_ready() {}
    }

    unsafe fn identify() -> DriveProperties {
        // https://wiki.osdev.org/ATA_PIO_Mode#IDENTIFY_command

        // I know this is bad, but it as of now hardcodes the drive to identify
        let mut port_lba3 = UnsafePort::<u8>::new(PORT_LBA3);
        port_lba3.write(0xB0);

        // Clear LBA_N ports
        let mut port_seccount = UnsafePort::<u8>::new(PORT_SECCOUNT);
        port_seccount.write(0);
        let mut port_lba0 = UnsafePort::<u8>::new(PORT_LBA0);
        port_lba0.write(0);
        let mut port_lba1 = UnsafePort::<u8>::new(PORT_LBA1);
        port_lba1.write(0);
        let mut port_lba2 = UnsafePort::<u8>::new(PORT_LBA2);
        port_lba2.write(0);


        // Send IDENTIFY command
        Self::send_command(0xEC);

        //for j in 0..10000 {
        //    let _ = ctrl.read();
        //}

        let mut first_cleared = true;
        loop {
            let data: u8 = Self::read_status();

            if data == 0 {
                panic!("ATA_PIO: Drive does not exist");
            }

            if (data & 1) != 0 {
                panic!("ATA_PIO: Drive controller error on IDENTIFY");
            }

            if (data & (1 << 7)) != 0 {
                // is busy
                continue;
            }

            if first_cleared {
                first_cleared = false;
                let v1 = port_lba1.read();
                let v2 = port_lba2.read();
                if v1 != 0 || v2 != 0 {
                    panic!("ATA_PIO: Not an ATA drive");
                }
                continue;
            }

            if (data & (1 << 3)) != 0 {
                break;
            }
        }

        let mut data_port = UnsafePort::<u16>::new(PORT_DATA);
        let mut data: [u16; 256] = [0; 256];

        // Lints are annoying
        #[allow(clippy::all)]
        for i in 0..256 {
            data[i] = data_port.read();
            //for j in 0..10000 {
            //    let _ = ctrl.read();
            //}
        }

        let lba48_supported = (data[83] & (1 << 10)) != 0;
        let lba28_sectors = (data[60] as u32) | ((data[61] as u32) << 0x10);
        let lba48_sectors: Option<u64> = if lba48_supported {
            Some(
                (data[100] as u64)
                    | ((data[101] as u64) << 0x10)
                    | ((data[102] as u64) << 0x20)
                    | ((data[103] as u64) << 0x30),
            )
        } else {
            None
        };

        if lba28_sectors == 0 && (lba48_sectors.is_none() || lba48_sectors == Some(0)) {
            panic!("ATA_PIO: The drive controller does not support LBA.");
        }

        DriveProperties {
            lba28_sectors,
            lba48_sectors,
        }
    }
    /// # Safety
    /// 
    /// This function uses ports, which make it unsafe. Carry on.
    pub unsafe fn read_lba(&self, lba: u32, sectors: u8) -> Vec<u8> {
        // https://wiki.osdev.org/ATA_read/write_sectors#Read_in_LBA_mode

        assert!(sectors > 0);
        let _data: u8 = Self::read_status();

        // Send bits 24-27 of LBA, drive number and LBA mode
        let mut port = UnsafePort::<u8>::new(PORT_LBA3);
        let mut bits24_27: u8 = (lba >> 24) as u8;
        assert!(bits24_27 < 8);
        bits24_27 |= 0b11110000; // Drive select - we want the slave drive
        port.write(bits24_27);

        // Send number of sectors
        let mut port = UnsafePort::<u8>::new(PORT_SECCOUNT);
        port.write(sectors);

        // Send bits 0-7 of LBA
        let mut port = UnsafePort::<u8>::new(PORT_LBA0);
        port.write((lba & 0xFF) as u8);

        // Send bits0xB016-23 of LBA
        let mut port = UnsafePort::<u8>::new(PORT_LBA2);
        port.write(((lba & 0xFF0000) >> 0x10) as u8);

        // Send command
        Self::send_command(0x20); // Read with retry

        Self::wait_ready();

        let mut data_port = UnsafePort::<u16>::new(PORT_DATA);
        let u16_per_sector = SECTOR_SIZE / 2;

        let mut result: Vec<u8> = Vec::new();
        for _ in 0..sectors {
            for _ in 0..u16_per_sector {
                let word: u16 = data_port.read();
                result.push((word & 0xFF) as u8);
                result.push(((word & 0xFF00) >> 0x8) as u8);
            }
        }

        result
    }

    fn init(&mut self) -> bool {
        true
    }

    fn sector_size(&self) -> u64 {
        0x200
    }

    /// Capacity in sectors
    fn capacity_sectors(&mut self) -> u64 {
        self.properties.sector_count()
    }

    fn read(&mut self, sector: u64) -> Vec<u8> {
        assert!(sector < self.properties.sector_count());

        unsafe { self.read_lba(sector as u32, 1) }
    }

    // Make the fs driver do the hard job of converting Vec<u8> to Vec<u16>
    /// # Safety
    /// 
    /// This function uses ports, which make it unsafe. Carry on.
    pub unsafe fn write(&self, lba: u32, sectors: u8, data: Vec<u16>) {
        // https://wiki.osdev.org/ATA_read/write_sectors#Read_in_LBA_mode
        Self::wait_ready();

        assert!(sectors > 0);

        // Send bits 24-27 of LBA, drive number and LBA mode
        let mut port = UnsafePort::<u8>::new(PORT_LBA3);
        let mut bits24_27: u8 = (lba >> 24) as u8;
        assert!(bits24_27 < 8);
        bits24_27 |= 0b11110000; // Drive select - we want the slave drive
        port.write(bits24_27);

        // Send number of sectors
        let mut port = UnsafePort::<u8>::new(PORT_SECCOUNT);
        port.write(sectors);

        // Send bits 0-7 of LBA
        let mut port = UnsafePort::<u8>::new(PORT_LBA0);
        port.write((lba & 0xFF) as u8);

        // Send bits 8-15 of LBA
        let mut port = UnsafePort::<u8>::new(PORT_LBA1);
        port.write(((lba & 0xFF00) >> 0x8) as u8);

        // Send bits 16-23 of LBA
        let mut port = UnsafePort::<u8>::new(PORT_LBA2);
        port.write(((lba & 0xFF0000) >> 0x10) as u8);

        // Send command
        Self::send_command(0x30); // Read with retry

        Self::wait_ready();

        let mut data_port = UnsafePort::<u16>::new(PORT_DATA);
        let u16_per_sector = SECTOR_SIZE / 2;
        // Not sure if this code works
        /*for i in data.iter() {
            data_port.write(*i);
        }*/
        for i in 0..sectors {
            for j in 0..u16_per_sector {
                let word = data.get(i as usize*u16_per_sector + j);
                let word=  match word {
                    Some(i) => *i,
                    None => 0,
                };
                data_port.write(word);
            }
        }



    }
}