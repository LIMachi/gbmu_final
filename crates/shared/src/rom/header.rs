use std::ops::BitXor;

use log::warn;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Console {
    GBC,
    DMG,
    All,
    Other(u8),
}

impl Default for Console {
    fn default() -> Self { Console::DMG }
}

impl Console {
    pub fn cgb_mode(&self, on_gbc: bool) -> bool {
        on_gbc && matches!(self, Console::GBC | Console::All)
    }
}

impl From<u8> for Console {
    fn from(value: u8) -> Self {
        match value {
            n if n & 0x80 == 0 => Console::DMG,
            0x80 => Console::All,
            0xC0 => Console::GBC,
            n => Console::Other(n)
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Gameboy {
    DMG,
    Super,
    Other,
}

impl Default for Gameboy {
    fn default() -> Self { Self::DMG }
}

impl From<u8> for Gameboy {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Gameboy::DMG,
            0x03 => Gameboy::Super,
            set => {
                log::warn!("Not a valid instruction set {set:#04x}!");
                Gameboy::Other
            }
        }
    }
}

mod capability {
    pub const NONE: u8 = 0b00000;
    // always
    pub const RAM: u8 = 0b00001;
    pub const SRAM: u8 = 0b00011;
    pub const BATT: u8 = 0b00100;
    // battery = save
    pub const TMR: u8 = 0b01000;
    // timer
    pub const MR: u8 = 0b10000; // rumble
}

pub struct Capabilities(u8);

impl Default for Capabilities {
    fn default() -> Self { Self(capability::NONE) }
}

impl Capabilities {
    pub fn ram(&self) -> bool {
        (self.0 & capability::RAM) != 0
    }

    pub fn save(&self) -> bool {
        (self.0 & capability::BATT) != 0 //TODO gros cat pas sur ....
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

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Cartridge {
    Rom,
    RomMbc1,
    RomRamMbc1,
    RomRamMbc1Batt,
    RomMbc2,
    RomMbc2Batt,
    RomRam,
    RomRamBatt,
    RomMmm01,
    RomMmm01Sram,
    RomMmm01SramBatt,
    RomMbc3TmrBatt,
    RomRamMbc3TmrBatt,
    RomMbc3,
    RomRamMbc3,
    RomRamMbc3Batt,
    RomMbc5,
    RomRamMbc5,
    RomRamMbc5Batt,
    RomMbc5Mr,
    RomMbc5MrSram,
    RomMbc5MrSramBatt,
    UNSUPPORTED,
}

impl Default for Cartridge {
    fn default() -> Self { Cartridge::Rom }
}


pub enum Mbc {
    MBC0,
    MBC1,
    MBC2,
    MBC3,
    MBC5,
    Unknown,
}

impl Default for Mbc {
    fn default() -> Self { Mbc::MBC0 }
}

impl Cartridge {
    pub fn capabilities(&self) -> Capabilities {
        use capability::*;
        match self {
            Cartridge::Rom => Capabilities(NONE),
            Cartridge::RomMbc1 => Capabilities(NONE),
            Cartridge::RomRamMbc1 => Capabilities(RAM),
            Cartridge::RomRamMbc1Batt => Capabilities(RAM | BATT),
            Cartridge::RomMbc2 => Capabilities(NONE),
            Cartridge::RomMbc2Batt => Capabilities(RAM | BATT),
            Cartridge::RomRam => Capabilities(RAM),
            Cartridge::RomRamBatt => Capabilities(RAM | BATT),
            Cartridge::RomMmm01 => Capabilities(NONE),
            Cartridge::RomMmm01Sram => Capabilities(SRAM),
            Cartridge::RomMmm01SramBatt => Capabilities(SRAM | BATT),
            Cartridge::RomMbc3TmrBatt => Capabilities(TMR | BATT),
            Cartridge::RomRamMbc3TmrBatt => Capabilities(RAM | TMR | BATT),
            Cartridge::RomMbc3 => Capabilities(NONE),
            Cartridge::RomRamMbc3 => Capabilities(RAM),
            Cartridge::RomRamMbc3Batt => Capabilities(RAM | BATT),
            Cartridge::RomMbc5 => Capabilities(NONE),
            Cartridge::RomRamMbc5 => Capabilities(RAM),
            Cartridge::RomRamMbc5Batt => Capabilities(RAM | BATT),
            Cartridge::RomMbc5Mr => Capabilities(MR),
            Cartridge::RomMbc5MrSram => Capabilities(MR | SRAM),
            Cartridge::RomMbc5MrSramBatt => Capabilities(MR | SRAM | BATT),
            Cartridge::UNSUPPORTED => Capabilities(NONE)
        }
    }

    pub fn mbc(&self) -> Mbc {
        match self {
            Cartridge::Rom => Mbc::MBC0,
            Cartridge::RomRamMbc1 => Mbc::MBC1,
            Cartridge::RomMbc1 => Mbc::MBC1,
            Cartridge::RomRamMbc1Batt => Mbc::MBC1,
            Cartridge::RomMbc2 => Mbc::MBC2,
            Cartridge::RomMbc2Batt => Mbc::MBC2,
            Cartridge::RomRam => Mbc::MBC0,
            Cartridge::RomRamBatt => Mbc::MBC0,
            Cartridge::RomMmm01 => Mbc::Unknown,
            Cartridge::RomMmm01Sram => Mbc::Unknown,
            Cartridge::RomMmm01SramBatt => Mbc::Unknown,
            Cartridge::RomMbc3TmrBatt => Mbc::MBC3,
            Cartridge::RomRamMbc3TmrBatt => Mbc::MBC3,
            Cartridge::RomMbc3 => Mbc::MBC3,
            Cartridge::RomRamMbc3 => Mbc::MBC3,
            Cartridge::RomRamMbc3Batt => Mbc::MBC3,
            Cartridge::RomMbc5 => Mbc::MBC5,
            Cartridge::RomRamMbc5 => Mbc::MBC5,
            Cartridge::RomRamMbc5Batt => Mbc::MBC5,
            Cartridge::RomMbc5Mr => Mbc::MBC5,
            Cartridge::RomMbc5MrSram => Mbc::MBC5,
            Cartridge::RomMbc5MrSramBatt => Mbc::MBC5,
            Cartridge::UNSUPPORTED => Mbc::Unknown
        }
    }
}

impl From<u8> for Cartridge {
    fn from(value: u8) -> Self {
        match value {
            0x0 => Cartridge::Rom,
            0x1 => Cartridge::RomMbc1,
            0x2 => Cartridge::RomRamMbc1,
            0x3 => Cartridge::RomRamMbc1Batt,
            0x5 => Cartridge::RomMbc2,
            0x6 => Cartridge::RomMbc2Batt,
            0x8 => Cartridge::RomRam,
            0x9 => Cartridge::RomRamBatt,
            0xB => Cartridge::RomMmm01,
            0xC => Cartridge::RomMmm01Sram,
            0xD => Cartridge::RomMmm01SramBatt,
            0xF => Cartridge::RomMbc3TmrBatt,
            0x10 => Cartridge::RomRamMbc3TmrBatt,
            0x11 => Cartridge::RomMbc3,
            0x12 => Cartridge::RomRamMbc3,
            0x13 => Cartridge::RomRamMbc3Batt,
            0x19 => Cartridge::RomMbc5,
            0x1A => Cartridge::RomRamMbc5,
            0x1B => Cartridge::RomRamMbc5Batt,
            0x1C => Cartridge::RomMbc5Mr,
            0x1D => Cartridge::RomMbc5MrSram,
            0x1E => Cartridge::RomMbc5MrSramBatt,
            _ => Cartridge::UNSUPPORTED
        }
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct RomSize(u8);

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct RamSize(u8);

impl RomSize {
    pub const BANK_SIZE: u16 = 16384;

    pub fn new(v: u8) -> Self { Self(v) }
    pub fn banks(&self) -> usize {
        match self.0 {
            n @ 0..=8 => 2usize.pow((n + 1) as u32),
            0x52 => 72,
            0x53 => 80,
            0x54 => 96,
            n => {
                warn!("invalid rom size {n}");
                0
            }
        }
    }

    pub fn mask(&self) -> usize {
        match self.0 {
            n @ 0..=8 => 2usize.pow((n + 1) as u32) - 1,
            0x52 | 0x53 | 0x54 => 0x7F,
            n => {
                warn!("invalid rom size {n}");
                0
            }
        }
    }

    pub fn size(&self) -> usize {
        self.banks() as usize * RomSize::BANK_SIZE as usize
    }
}

impl RamSize {
    pub const BANK_SIZE: u16 = 8192;

    pub fn new(v: u8) -> Self { Self(v) }

    pub fn banks(&self) -> usize {
        match self.0 {
            2 => 1,
            3 => 4,
            4 => 16,
            5 => 8,
            n => {
                warn!("invalid ram size {n}");
                0
            }
        }
    }

    pub fn mask(&self) -> usize {
        match self.0 {
            2 => 0x0,
            3 => 0x3,
            4 => 0xF,
            5 => 0x7,
            n => {
                warn!("invalid ram size {n}");
                0
            }
        }
    }

    pub fn size(&self) -> usize {
        self.banks() as usize * RamSize::BANK_SIZE as usize
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
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
    pub checksum: u16,
}

impl Header {
    pub fn new(mem: &[u8]) -> Self {
        let cs = &mem[0x14E..=0x14F];
        Self {
            logo: mem[0x104..0x134].to_vec(),
            title: String::from_utf8_lossy(&mem[0x134..=0x142]).to_string().replace(char::from(0), ""),
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
            checksum: u16::from_le_bytes([cs[1], cs[0]]),
        }
    }
}
