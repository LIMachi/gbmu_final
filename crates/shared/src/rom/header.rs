use std::ops::BitXor;
use log::warn;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Console {
    GBC,
    Other
}

impl Console {
    pub fn requires_gbc(&self) -> bool { self == &Console::GBC }
}

impl From<u8> for Console {
    fn from(value: u8) -> Self {
        match value {
            0x80 => Console::GBC,
            _ => Console::Other
        }
    }
}

#[derive(Debug, Clone)]
pub enum Gameboy {
    DMG,
    Super
}

impl From<u8> for Gameboy {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Gameboy::DMG,
            0x03 => Gameboy::Super,
            _ => panic!("Not a valid instruction set!")
        }
    }
}

mod capability {
    pub const NONE: u8 = 0b00000; // always
    pub const RAM: u8  = 0b00001;
    pub const SRAM: u8 = 0b00011;
    pub const BATT: u8 = 0b00100; // battery = save
    pub const TMR: u8  = 0b01000; // timer
    pub const MR: u8   = 0b10000; // rumble
}

pub struct Capabilities(u8);

impl Capabilities {
    pub fn ram(&self) -> bool {
        (self.0 & capability::RAM) != 0
    }

    pub fn save(&self) -> bool {
        (self.0 & (capability::RAM | capability::BATT)) != 0
    }

    pub fn switch(&self) -> bool {
        (self.0 & capability::SRAM.bitxor(capability::RAM)) != 0
    }

    pub fn rumble(&self) -> bool {
        (self.0 & capability::MR) != 0
    }

    pub fn rtc(&self) -> bool {
        (self.0 & capability::TMR) != 0
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Cartridge {
    ROM,
    ROM_MBC1,
    ROM_RAM_MBC1,
    ROM_RAM_MBC1_BATT,
    ROM_MBC2,
    ROM_MBC2_BATT,
    ROM_RAM,
    ROM_RAM_BATT,
    ROM_MMM01,
    ROM_MMM01_SRAM,
    ROM_MMM01_SRAM_BATT,
    ROM_MBC3_TMR_BATT,
    ROM_RAM_MBC3_TMR_BATT,
    ROM_MBC3,
    ROM_RAM_MBC3,
    ROM_RAM_MBC3_BATT,
    ROM_MBC5,
    ROM_RAM_MBC5,
    ROM_RAM_MBC5_BATT,
    ROM_MBC5_MR,
    ROM_MBC5_MR_SRAM,
    ROM_MBC5_MR_SRAM_BATT,
    UNSUPPORTED
}

pub enum Mbc {
    MBC0,
    MBC1,
    MBC2,
    MBC3,
    MBC5,
    Unknown
}

impl Cartridge {

    pub fn capabilities(&self) -> Capabilities {
        use capability::*;
        match self {
            Cartridge::ROM                     => Capabilities(NONE),
            Cartridge::ROM_MBC1                => Capabilities(NONE),
            Cartridge::ROM_RAM_MBC1            => Capabilities(RAM),
            Cartridge::ROM_RAM_MBC1_BATT       => Capabilities(RAM | BATT),
            Cartridge::ROM_MBC2                => Capabilities(NONE),
            Cartridge::ROM_MBC2_BATT           => Capabilities(RAM | BATT),
            Cartridge::ROM_RAM                 => Capabilities(RAM),
            Cartridge::ROM_RAM_BATT            => Capabilities(RAM | BATT),
            Cartridge::ROM_MMM01               => Capabilities(NONE),
            Cartridge::ROM_MMM01_SRAM          => Capabilities(SRAM),
            Cartridge::ROM_MMM01_SRAM_BATT     => Capabilities(SRAM | BATT),
            Cartridge::ROM_MBC3_TMR_BATT       => Capabilities(TMR | BATT),
            Cartridge::ROM_RAM_MBC3_TMR_BATT   => Capabilities(RAM | TMR | BATT),
            Cartridge::ROM_MBC3                => Capabilities(NONE),
            Cartridge::ROM_RAM_MBC3            => Capabilities(RAM),
            Cartridge::ROM_RAM_MBC3_BATT       => Capabilities(RAM | BATT),
            Cartridge::ROM_MBC5                => Capabilities(NONE),
            Cartridge::ROM_RAM_MBC5            => Capabilities(RAM),
            Cartridge::ROM_RAM_MBC5_BATT       => Capabilities(RAM | BATT),
            Cartridge::ROM_MBC5_MR             => Capabilities(MR),
            Cartridge::ROM_MBC5_MR_SRAM        => Capabilities(MR | SRAM),
            Cartridge::ROM_MBC5_MR_SRAM_BATT   => Capabilities(MR | SRAM | BATT),
            Cartridge::UNSUPPORTED             => Capabilities(NONE)
        }
    }

    pub fn mbc(&self) -> Mbc {
        match self {
            Cartridge::ROM                     => Mbc::MBC0,
            Cartridge::ROM_RAM_MBC1            => Mbc::MBC1,
            Cartridge::ROM_MBC1                => Mbc::MBC1,
            Cartridge::ROM_RAM_MBC1_BATT       => Mbc::MBC1,
            Cartridge::ROM_MBC2                => Mbc::MBC2,
            Cartridge::ROM_MBC2_BATT           => Mbc::MBC2,
            Cartridge::ROM_RAM                 => Mbc::MBC0,
            Cartridge::ROM_RAM_BATT            => Mbc::MBC0,
            Cartridge::ROM_MMM01               => Mbc::Unknown,
            Cartridge::ROM_MMM01_SRAM          => Mbc::Unknown,
            Cartridge::ROM_MMM01_SRAM_BATT     => Mbc::Unknown,
            Cartridge::ROM_MBC3_TMR_BATT       => Mbc::MBC3,
            Cartridge::ROM_RAM_MBC3_TMR_BATT   => Mbc::MBC3,
            Cartridge::ROM_MBC3                => Mbc::MBC3,
            Cartridge::ROM_RAM_MBC3            => Mbc::MBC3,
            Cartridge::ROM_RAM_MBC3_BATT       => Mbc::MBC3,
            Cartridge::ROM_MBC5                => Mbc::MBC5,
            Cartridge::ROM_RAM_MBC5            => Mbc::MBC5,
            Cartridge::ROM_RAM_MBC5_BATT       => Mbc::MBC5,
            Cartridge::ROM_MBC5_MR             => Mbc::MBC5,
            Cartridge::ROM_MBC5_MR_SRAM        => Mbc::MBC5,
            Cartridge::ROM_MBC5_MR_SRAM_BATT   => Mbc::MBC5,
            Cartridge::UNSUPPORTED             => Mbc::Unknown
        }
    }
}

impl From<u8> for Cartridge {
    fn from(value: u8) -> Self {
        match value {
            0x0 => Cartridge::ROM,
            0x1 => Cartridge::ROM_MBC1,
            0x2 => Cartridge::ROM_RAM_MBC1,
            0x3 => Cartridge::ROM_RAM_MBC1_BATT,
            0x5 => Cartridge::ROM_MBC2,
            0x6 => Cartridge::ROM_MBC2_BATT,
            0x8 => Cartridge::ROM_RAM,
            0x9 => Cartridge::ROM_RAM_BATT,
            0xB => Cartridge::ROM_MMM01,
            0xC => Cartridge::ROM_MMM01_SRAM,
            0xD => Cartridge::ROM_MMM01_SRAM_BATT,
            0xF => Cartridge::ROM_MBC3_TMR_BATT,
            0x10 => Cartridge::ROM_RAM_MBC3_TMR_BATT,
            0x11 => Cartridge::ROM_MBC3,
            0x12 => Cartridge::ROM_RAM_MBC3,
            0x13 => Cartridge::ROM_RAM_MBC3_BATT,
            0x19 => Cartridge::ROM_MBC5,
            0x1A => Cartridge::ROM_RAM_MBC5,
            0x1B => Cartridge::ROM_RAM_MBC5_BATT,
            0x1C => Cartridge::ROM_MBC5_MR,
            0x1D => Cartridge::ROM_MBC5_MR_SRAM,
            0x1E => Cartridge::ROM_MBC5_MR_SRAM_BATT,
            _ => Cartridge::UNSUPPORTED
        }
    }
}

#[derive(Debug, Clone)]
pub struct RomSize(u8);

#[derive(Debug, Clone)]
pub struct RamSize(u8);

impl RomSize {
    pub const BANK_SIZE: u16 = 16384;

    pub fn new(v: u8) -> Self { Self(v) }
    pub fn banks(&self) -> u8 {
        match self.0 {
            n @0..=6 => 2u8.pow((n + 1) as u32),
            0x52 => 72,
            0x53 => 80,
            0x54 => 96,
            n => { warn!("invalid rom size {n}"); 0 }
        }
    }

    pub fn size(&self) -> usize {
        self.banks() as usize * RomSize::BANK_SIZE as usize
    }
}

impl RamSize {
    pub const BANK_SIZE: u16 = 8192;

    pub fn new(v: u8) -> Self { Self(v) }

    pub fn banks(&self) -> u8 {
        match self.0 {
            2 => 1,
            3 => 4,
            4 => 16,
            5 => 8,
            n => { warn!("invalid ram size {n}"); 0 }
        }
    }

    pub fn size(&self) -> usize {
        self.banks() as usize * RamSize::BANK_SIZE as usize
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    pub logo: Vec<u8>,
    pub title: String,
    pub kind: Console,
    pub license: [u8; 2],
    pub funcs: Gameboy,
    pub cartridge: Cartridge,
    pub rom_size: RomSize,
    pub ram_size: RamSize,
    pub gaijin: bool,
    pub lcode: u8,
    pub rom_v: u8,
    pub check: u8,
    pub checksum: u16
}

impl Header {
    pub fn new(mem: &[u8]) -> Self {
        let cs = &mem[0x14E..=0x14F];
        Self {
            logo: mem[0x104..0x134].to_vec(),
            title: String::from_utf8_lossy(&mem[0x134..=0x142]).to_string(),
            kind: Console::from(mem[0x143]),
            license: [mem[0x144], mem[0x145]],
            funcs: Gameboy::from(mem[0x146]),
            cartridge: Cartridge::from(mem[0x147]),
            rom_size: RomSize::new(mem[0x148]),
            ram_size: RamSize::new(mem[0x149]),
            gaijin: mem[0x14A] != 0,
            lcode: mem[0x14B],
            rom_v: mem[0x14C],
            check: mem[0x14D],
            checksum: u16::from_le_bytes([cs[1], cs[0]])
        }
    }
}
