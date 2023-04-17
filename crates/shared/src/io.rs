use serde::{Serialize, Deserialize};
use super::mem::Mem;

mod io_regs;
mod devices;

pub use io_regs::IORegs;
pub use devices::IODevice;

pub const CGB_MODE: u8 = 0x80;
pub const DMG_MODE: u8 = 0x04;

/// 7	LCD and PPU enable	0=Off, 1=On
// 6	Window tile map area	0=9800-9BFF, 1=9C00-9FFF
// 5	Window enable	0=Off, 1=On
// 4	BG and Window tile data area	0=8800-97FF, 1=8000-8FFF
// 3	BG tile map area	0=9800-9BFF, 1=9C00-9FFF
// 2	OBJ size	0=8x8, 1=8x16
// 1	OBJ enable	0=Off, 1=On
// 0	BG and Window enable/priority	0=Off, 1=On

pub trait LCDC {
    fn enabled(&self) -> bool;
    fn win_area(&self) -> bool;
    fn win_enable(&self) -> bool;
    fn relative_addr(&self) -> bool;
    fn bg_area(&self) -> bool;
    fn obj_size(&self) -> u8;
    fn obj_tall(&self) -> bool;
    fn obj_enable(&self) -> bool;
    fn priority(&self) -> bool;

}

impl LCDC for u8 {
    fn enabled(&self) -> bool {
        (self & 0x80) != 0
    }
    fn win_area(&self) -> bool {
        (self & 0x40) != 0
    }

    fn win_enable(&self) -> bool {
        (self & 0x20) != 0
    }

    fn relative_addr(&self) -> bool {
        (self & 0x10) == 0
    }

    fn bg_area(&self) -> bool {
        (self & 0x08) != 0
    }

    fn obj_size(&self) -> u8 {
        if (self & 0x4) == 0 { 0x8 } else { 0x10 }
    }

    fn obj_tall(&self) -> bool {
        (self & 0x4) != 0
    }

    fn obj_enable(&self) -> bool {
        (self & 0x2) != 0
    }

    fn priority(&self) -> bool {
        (self & 0x1) != 0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum IO {
    ///0x100
    CGB              = 0x100,
    ///0xFF00
    JOYP             = 0xFF00,
    ///0xFF01
    SB               = 0xFF01,
    ///0xFF02
    SC               = 0xFF02,
    ///0xFF04
    DIV              = 0xFF04,
    ///0xFF05
    TIMA             = 0xFF05,
    ///0xFF06
    TMA              = 0xFF06,
    ///0xFF07
    TAC              = 0xFF07,
    ///0xFF0F
    IF               = 0xFF0F,
    ///0xFF10
    NR10             = 0xFF10,
    ///0xFF11
    NR11             = 0xFF11,
    ///0xFF12
    NR12             = 0xFF12,
    ///0xFF13
    NR13             = 0xFF13,
    ///0xFF14
    NR14             = 0xFF14,
    ///0xFF16
    NR21             = 0xFF16,
    ///0xFF17
    NR22             = 0xFF17,
    ///0xFF18
    NR23             = 0xFF18,
    ///0xFF19
    NR24             = 0xFF19,
    ///0xFF1A
    NR30             = 0xFF1A,
    ///0xFF1B
    NR31             = 0xFF1B,
    ///0xFF1C
    NR32             = 0xFF1C,
    ///0xFF1D
    NR33             = 0xFF1D,
    ///0xFF1E
    NR34             = 0xFF1E,
    ///0xFF20
    NR41             = 0xFF20,
    ///0xFF21
    NR42             = 0xFF21,
    ///0xFF22
    NR43             = 0xFF22,
    ///0xFF23
    NR44             = 0xFF23,
    ///0xFF24
    NR50             = 0xFF24,
    ///0xFF25
    NR51             = 0xFF25,
    ///0xFF26
    NR52             = 0xFF26,
    ///0xFF30
    WaveRam0         = 0xFF30,
    ///0xFF31
    WaveRam1         = 0xFF31,
    ///0xFF32
    WaveRam2         = 0xFF32,
    ///0xFF33
    WaveRam3         = 0xFF33,
    ///0xFF34
    WaveRam4         = 0xFF34,
    ///0xFF35
    WaveRam5         = 0xFF35,
    ///0xFF36
    WaveRam6         = 0xFF36,
    ///0xFF37
    WaveRam7         = 0xFF37,
    ///0xFF38
    WaveRam8         = 0xFF38,
    ///0xFF39
    WaveRam9         = 0xFF39,
    ///0xFF3A
    WaveRamA         = 0xFF3A,
    ///0xFF3B
    WaveRamB         = 0xFF3B,
    ///0xFF3C
    WaveRamC         = 0xFF3C,
    ///0xFF3D
    WaveRamD         = 0xFF3D,
    ///0xFF3E
    WaveRamE         = 0xFF3E,
    ///0xFF3F
    WaveRamF         = 0xFF3F,
    ///0xFF40
    LCDC             = 0xFF40,
    ///0xFF41
    STAT             = 0xFF41,
    ///0xFF42
    SCY              = 0xFF42,
    ///0xFF43
    SCX              = 0xFF43,
    ///0xFF44
    LY               = 0xFF44,
    ///0xFF45
    LYC              = 0xFF45,
    ///0xFF46
    DMA              = 0xFF46,
    ///0xFF47
    BGP              = 0xFF47,
    ///0xFF48
    OBP0             = 0xFF48,
    ///0xFF49
    OBP1             = 0xFF49,
    ///0xFF4A
    WY               = 0xFF4A,
    ///0xFF4B
    WX               = 0xFF4B,
    ///0xFF4C
    KEY0             = 0xFF4C,
    ///0xFF4D
    KEY1             = 0xFF4D,
    ///0xFF4F
    VBK              = 0xFF4F,
    ///0xFF50
    POST             = 0xFF50,
    ///0xFF51
    HDMA1            = 0xFF51,
    ///0xFF52
    HDMA2            = 0xFF52,
    ///0xFF53
    HDMA3            = 0xFF53,
    ///0xFF54
    HDMA4            = 0xFF54,
    ///0xFF55
    HDMA5            = 0xFF55,
    ///0xFF56
    RP               = 0xFF56,
    ///0xFF68
    BCPS             = 0xFF68,
    ///0xFF69
    BCPD             = 0xFF69,
    ///0xFF6A
    OCPS             = 0xFF6A,
    ///0xFF6B
    OCPD             = 0xFF6B,
    ///0xFF6C
    OPRI             = 0xFF6C,
    ///0xFF70
    SVBK             = 0xFF70,
    ///0xFF76
    PCM12            = 0xFF76,
    ///0xFF77
    PCM34            = 0xFF77,
    ///0xFFFF
    IE               = 0xFFFF
}

impl IO {
    pub const fn name(&self) -> &str {
        match self {
            IO::CGB => "CGB",
            IO::JOYP => "JOY",
            IO::SB => "SB",
            IO::SC => "SC",
            IO::DIV => "DIV",
            IO::TIMA => "TIMA",
            IO::TMA => "TMA",
            IO::TAC => "TAC",
            IO::IF => "IF",
            IO::NR10 => "NR10",
            IO::NR11 => "NR11",
            IO::NR12 => "NR12",
            IO::NR13 => "NR13",
            IO::NR14 => "NR14",
            IO::NR21 => "NR21",
            IO::NR22 => "NR22",
            IO::NR23 => "NR23",
            IO::NR24 => "NR24",
            IO::NR30 => "NR30",
            IO::NR31 => "NR31",
            IO::NR32 => "NR32",
            IO::NR33 => "NR33",
            IO::NR34 => "NR34",
            IO::NR41 => "NR41",
            IO::NR42 => "NR42",
            IO::NR43 => "NR43",
            IO::NR44 => "NR44",
            IO::NR50 => "NR50",
            IO::NR51 => "NR51",
            IO::NR52 => "NR52",
            IO::WaveRam0 => "WaveRam0",
            IO::WaveRam1 => "WaveRam1",
            IO::WaveRam2 => "WaveRam2",
            IO::WaveRam3 => "WaveRam3",
            IO::WaveRam4 => "WaveRam4",
            IO::WaveRam5 => "WaveRam5",
            IO::WaveRam6 => "WaveRam6",
            IO::WaveRam7 => "WaveRam7",
            IO::WaveRam8 => "WaveRam8",
            IO::WaveRam9 => "WaveRam9",
            IO::WaveRamA => "WaveRamA",
            IO::WaveRamB => "WaveRamB",
            IO::WaveRamC => "WaveRamC",
            IO::WaveRamD => "WaveRamD",
            IO::WaveRamE => "WaveRamE",
            IO::WaveRamF => "WaveRamF",
            IO::LCDC => "LCDC",
            IO::STAT => "STAT",
            IO::SCY => "SCY",
            IO::SCX => "SCX",
            IO::LY => "LY",
            IO::LYC => "LYC",
            IO::DMA => "DMA",
            IO::BGP => "BGP",
            IO::OBP0 => "OBP0",
            IO::OBP1 => "OBP1",
            IO::WY => "WY",
            IO::WX => "WX",
            IO::KEY0 => "KEY0",
            IO::KEY1 => "KEY1",
            IO::VBK => "VBK",
            IO::POST => "POST",
            IO::HDMA1 => "HDMA1",
            IO::HDMA2 => "HDMA2",
            IO::HDMA3 => "HDMA3",
            IO::HDMA4 => "HDMA4",
            IO::HDMA5 => "HDMA5",
            IO::RP => "RP",
            IO::BCPS => "BCPS",
            IO::BCPD => "BCPD",
            IO::OCPS => "OCPS",
            IO::OCPD => "OCPD",
            IO::OPRI => "OPRI",
            IO::SVBK => "SVBK",
            IO::PCM12 => "PCM12",
            IO::PCM34 => "PCM34",
            IO::IE => "IE",
        }
    }

    pub fn tooltip(&self, reg: u8) -> Option<String> {
        let panning = ["Off", "Right", "Left", "Both"];
        match self {
            IO::PCM12 => Some(format!("Only active in CGB, output of channels 1 & 2 (digital)\nChannel 1: {}\nChannel 2: {}", reg & 0xF, (reg >> 4) & 0xF)),
            IO::PCM34 => Some(format!("Only active in CGB, output of channels 3 & 4 (digital)\nChannel 3: {}\nChannel 4: {}", reg & 0xF, (reg >> 4) & 0xF)),
            IO::NR52 => Some(format!("APU active: {}\nActive channels:\n- pulse1 {}\n- pulse2 {}\n- wave {}\n- noise {}", reg & 0x80 != 0, reg & 1 != 0, reg & 2 != 0, reg & 4 != 0, reg & 8 != 0)),
            IO::NR51 => Some(format!("Channel panning:\n- pulse1 {}\n- pulse2 {}\n- wave {}\n- noise {}", panning[((reg & 0x1) | ((reg & 0x10) >> 3)) as usize], panning[((reg & 0x2) | ((reg & 0x20) >> 3)) as usize >> 1], panning[((reg & 0x4) | ((reg & 0x40) >> 3)) as usize >> 2], panning[((reg & 0x8) | ((reg & 0x80) >> 3)) as usize >> 3])),
            IO::NR50 => Some(format!("Master volume:\n- left {} %\n- right {} %\n VIN mix: {}", ((0x70 & reg) >> 4) as f32 / 7. * 100., (0x7 & reg) as f32 / 7. * 100., panning[((reg & 0x80) >> 6 | (reg & 0x8) >> 3) as usize])),
            IO::NR10 => Some(
                if reg & 0x70 != 0 {
                    format!("Channel 1 Sweep\nPace {} Hz\n{}\nSlope (divider): {}", ((reg & 0x70) >> 4) * 128, if reg & 0x8 != 0 { "Subtraction" } else { "Addition" }, 1 << (reg & 0x7))
                } else {
                    format!("Channel 1 Sweep\nDisabled (bits 4-6 disabled)")
                }
            ),
            IO::NR11 => Some(format!("Channel 1 Duty & Length\nWave duty {} %\nLength: {} Hz", 12.5 * (1 + ((reg & 0xC0) >> 6)) as f32, 256 * (reg as usize & 0x3F))),
            IO::NR12 => Some(format!("Channel 1 Volume & Envelope\nVolume {} %\n{}\nPace {} Hz", ((reg & 0xF0) >> 4) as f32 / 15. * 100., if reg & 0x8 != 0 { "Increase" } else { "Decrease" }, (reg & 0x7) as usize * 64)),
            IO::NR13 => Some(format!("Channel 1 Wavelength (low): {reg:#04X}")),
            IO::NR14 => Some(format!("Channel 1 Wavelength (high) & Control\nWave high: {:#04X}\nStop on NR11 timer finish {}\nTrigger {}", reg & 0x7, reg & 0x40 != 0, reg & 0x80 != 0)),
            IO::NR21 => Some(format!("Channel 2 Duty & Length\nWave duty {} %\nLength: {} Hz", 12.5 * (1 + ((reg & 0xC0) >> 6)) as f32, 256 * (reg as usize & 0x3F))),
            IO::NR22 => Some(format!("Channel 2 Volume & Envelope\nVolume {} %\n{}\nPace {} Hz", ((reg & 0xF0) >> 4) as f32 / 15. * 100., if reg & 0x8 != 0 { "Increase" } else { "Decrease" }, (reg & 0x7) as usize * 64)),
            IO::NR23 => Some(format!("Channel 2 Wavelength (low): {reg:#04X}")),
            IO::NR24 => Some(format!("Channel 2 Wavelength (high) & Control\nWave high: {:#04X}\nStop on NR21 timer finish {}\nTrigger {}", reg & 0x7, reg & 0x40 != 0, reg & 0x80 != 0)),
            IO::NR30 => Some(format!("Channel 3 DAC Setting: {}", if reg & 0x80 != 0 { "On" } else { "Off" })),
            IO::NR31 => Some(format!("Channel 3 Length: {} Hz", 256 * (reg as usize))),
            IO::NR32 => Some(format!("Channel 3 Volume: {}", if reg & 0x60 == 0 { "Mute".to_string() } else { format!("{} %", 100 >> (((reg & 0x60) >> 5) - 1)) })),
            IO::NR33 => Some(format!("Channel 3 Wavelength (low): {reg:#04X}")),
            IO::NR34 => Some(format!("Channel 3 Wavelength (high) & Control\nWave high: {:#04X}\nStop on NR31 timer finish {}\nTrigger {}", reg & 0x7, reg & 0x40 != 0, reg & 0x80 != 0)),
            IO::NR41 => Some(format!("Channel 4 Length: {} Hz", 256 * (reg & 0x3F) as usize)),
            IO::NR42 => Some(format!("Channel 4 Volume & Envelope\nVolume {} %\n{}\nPace {} Hz", ((reg & 0xF0) >> 4) as f32 / 15. * 100., if reg & 0x8 != 0 { "Increase" } else { "Decrease" }, (reg & 0x7) as usize * 64)),
            IO::NR43 => Some(format!("Channel 4 Frequency & LSFR width\nFrequency: {} Hz\nLSFR width (bits): {}", 262144. / (if reg & 7 == 0 { 0.5 } else { (reg & 7) as f32 }) * 2f32.powf((reg >> 4) as f32), 15 >> ((reg & 0x8) >> 3))),
            IO::NR44 => Some(format!("Channel 4 Control\nStop on NR41 timer finish {}\nTrigger {}", reg & 0x40 != 0, reg & 0x80 != 0)),
            IO::JOYP => Some(format!("Joypad Query/Status\nQuery {}\nRight / A {}\nLeft / B {}\nUp / Select {}\nDown / Start {}", ["None", "Direction", "Action", "All"][(reg & 0x30) as usize >> 4], reg & 0x1 == 0, reg & 0x2 == 0, reg & 0x4 == 0, reg & 0x8 == 0)),
            IO::IE => Some(format!("Enable interrupts:\nVBlank (@0x0040): {}\nLCD Stat (@0x0048): {}\nTimer (@0x0050): {}\nSerial (@0x0058): {}\nJoypad (@0x0060): {}", reg & 0x1 != 0, reg & 0x2 != 0, reg & 0x4 != 0, reg & 0x8 != 0, reg & 0x10 != 0)),
            IO::IF => Some(format!("Interrupt flag:\nVBlank (@0x0040): {}\nLCD Stat (@0x0048): {}\nTimer (@0x0050): {}\nSerial (@0x0058): {}\nJoypad (@0x0060): {}", reg & 0x1 != 0, reg & 0x2 != 0, reg & 0x4 != 0, reg & 0x8 != 0, reg & 0x10 != 0)),
            IO::SB => Some(format!("Serial byte: holds a byte that will (or is being) replaced by another coming from a connected gameboy (bit by bit every cycle)")),
            _ => None
        }
    }
}

impl TryFrom<u16> for IO {
    type Error = String;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            0xFF00 => IO::JOYP,
            0xFF01 => IO::SB,
            0xFF02 => IO::SC,
            0xFF04 => IO::DIV,
            0xFF05 => IO::TIMA,
            0xFF06 => IO::TMA,
            0xFF07 => IO::TAC,
            0xFF0F => IO::IF,
            0xFF10 => IO::NR10,
            0xFF11 => IO::NR11,
            0xFF12 => IO::NR12,
            0xFF13 => IO::NR13,
            0xFF14 => IO::NR14,
            0xFF16 => IO::NR21,
            0xFF17 => IO::NR22,
            0xFF18 => IO::NR23,
            0xFF19 => IO::NR24,
            0xFF1A => IO::NR30,
            0xFF1B => IO::NR31,
            0xFF1C => IO::NR32,
            0xFF1D => IO::NR33,
            0xFF1E => IO::NR34,
            0xFF20 => IO::NR41,
            0xFF21 => IO::NR42,
            0xFF22 => IO::NR43,
            0xFF23 => IO::NR44,
            0xFF24 => IO::NR50,
            0xFF25 => IO::NR51,
            0xFF26 => IO::NR52,
            0xFF30 => IO::WaveRam0,
            0xFF31 => IO::WaveRam1,
            0xFF32 => IO::WaveRam2,
            0xFF33 => IO::WaveRam3,
            0xFF34 => IO::WaveRam4,
            0xFF35 => IO::WaveRam5,
            0xFF36 => IO::WaveRam6,
            0xFF37 => IO::WaveRam7,
            0xFF38 => IO::WaveRam8,
            0xFF39 => IO::WaveRam9,
            0xFF3A => IO::WaveRamA,
            0xFF3B => IO::WaveRamB,
            0xFF3C => IO::WaveRamC,
            0xFF3D => IO::WaveRamD,
            0xFF3E => IO::WaveRamE,
            0xFF3F => IO::WaveRamF,
            0xFF40 => IO::LCDC,
            0xFF41 => IO::STAT,
            0xFF42 => IO::SCY,
            0xFF43 => IO::SCX,
            0xFF44 => IO::LY,
            0xFF45 => IO::LYC,
            0xFF46 => IO::DMA,
            0xFF47 => IO::BGP,
            0xFF48 => IO::OBP0,
            0xFF49 => IO::OBP1,
            0xFF4A => IO::WY,
            0xFF4B => IO::WX,
            0xFF4C => IO::KEY0,
            0xFF4D => IO::KEY1,
            0xFF4F => IO::VBK,
            0xFF50 => IO::POST,
            0xFF51 => IO::HDMA1,
            0xFF52 => IO::HDMA2,
            0xFF53 => IO::HDMA3,
            0xFF54 => IO::HDMA4,
            0xFF55 => IO::HDMA5,
            0xFF56 => IO::RP,
            0xFF68 => IO::BCPS,
            0xFF69 => IO::BCPD,
            0xFF6A => IO::OCPS,
            0xFF6B => IO::OCPD,
            0xFF6C => IO::OPRI,
            0xFF70 => IO::SVBK,
            0xFF76 => IO::PCM12,
            0xFF77 => IO::PCM34,
            0xFFFF => IO::IE,
            _ => return Err("HREG not in use".to_string())
        })
    }
}

impl IO {
    pub const fn access(&self) -> AccessMode {
        use Access::*;
        use AccessMode::*;
        match self {
            IO::CGB      => Generic(R),
            IO::JOYP     => Custom([R, R, R, R, RW, RW, U, U]),
            IO::SB       => Generic(RW),
            IO::SC       => Generic(RW),
            IO::DIV      => Generic(RW),
            IO::TIMA     => Generic(RW),
            IO::TMA      => Generic(RW),
            IO::TAC      => Custom([RW, RW, RW, U, U, U, U, U]),
            IO::IF       => Custom([RW, RW, RW, RW, RW, U, U, U]),
            IO::NR10     => Generic(RW),
            IO::NR11     => Custom([W, W, W, W, W, W, RW, RW]),
            IO::NR12     => Generic(RW),
            IO::NR13     => Generic(W),
            IO::NR14     => Custom([W, W, W, U, U, U, RW, W]),
            IO::NR21     => Custom([W, W, W, W, W, W, RW, RW]),
            IO::NR22     => Generic(RW),
            IO::NR23     => Generic(W),
            IO::NR24     => Custom([W, W, W, U, U, U, RW, W]),
            IO::NR30     => Generic(RW),
            IO::NR31     => Generic(W),
            IO::NR32     => Generic(RW),
            IO::NR33     => Generic(W),
            IO::NR34     => Custom([W, W, W, U, U, U, RW, W]),
            IO::NR41     => Generic(W),
            IO::NR42     => Generic(RW),
            IO::NR43     => Generic(RW),
            IO::NR44     => Custom([W, W, W, U, U, U, RW, W]),
            IO::NR50     => Generic(RW),
            IO::NR51     => Generic(RW),
            IO::NR52     => Custom([R, R, R, R, U, U, U, RW]),
            IO::WaveRam0 => Generic(RW),
            IO::WaveRam1 => Generic(RW),
            IO::WaveRam2 => Generic(RW),
            IO::WaveRam3 => Generic(RW),
            IO::WaveRam4 => Generic(RW),
            IO::WaveRam5 => Generic(RW),
            IO::WaveRam6 => Generic(RW),
            IO::WaveRam7 => Generic(RW),
            IO::WaveRam8 => Generic(RW),
            IO::WaveRam9 => Generic(RW),
            IO::WaveRamA => Generic(RW),
            IO::WaveRamB => Generic(RW),
            IO::WaveRamC => Generic(RW),
            IO::WaveRamD => Generic(RW),
            IO::WaveRamE => Generic(RW),
            IO::WaveRamF => Generic(RW),
            IO::LCDC => Generic(RW),
            IO::STAT => Custom([R, R, R, RW, RW, RW, RW, U]),
            IO::SCY => Generic(RW),
            IO::SCX => Generic(RW),
            IO::LY => Generic(R),
            IO::LYC => Generic(RW),
            IO::DMA => Generic(RW),
            IO::BGP => Generic(RW),
            IO::OBP0 => Generic(RW),
            IO::OBP1 => Generic(RW),
            IO::WY => Generic(RW),
            IO::WX => Generic(RW),
            IO::KEY0 => Generic(RW),
            IO::KEY1 => Custom([RW, U, U, U, U, U, U, R]),
            IO::VBK => Generic(RW),
            IO::POST => Generic(RW),
            IO::HDMA1 => Generic(W),
            IO::HDMA2 => Generic(W),
            IO::HDMA3 => Generic(W),
            IO::HDMA4 => Generic(W),
            IO::HDMA5 => Generic(RW),
            IO::RP => Custom([RW, R, U, U, U, U, RW, RW]),
            IO::BCPS => Generic(RW),
            IO::BCPD => Generic(RW),
            IO::OCPS => Generic(RW),
            IO::OCPD => Generic(RW),
            IO::OPRI => Generic(RW),
            IO::SVBK => Generic(RW),
            IO::PCM12 => Generic(R),
            IO::PCM34 => Generic(R),
            IO::IE => Custom([RW, RW, RW, RW, RW, U, U, U])
        }
    }
    pub const fn default(&self, cgb: bool) -> u8 {
        match self {
            IO::JOYP => 0xFF,
            IO::KEY0 if cgb => 0x80,
            _ => 0
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
pub enum Access { W, R, RW, U }
#[derive(Debug, Copy, Clone)]
pub enum AccessMode { Generic(Access), Custom([Access; 8]) }

impl Access {
    pub const fn format(&self) -> &'static str {
        match self {
            Access::R => "R",
            Access::W => "W",
            Access::RW => "R/W",
            Access::U => "U"
        }
    }
}

impl Default for AccessMode {
    fn default() -> Self { Self::Generic(Access::U) }
}

impl AccessMode {
    pub fn rmask(&self) -> u8 {
        match self {
            AccessMode::Generic(acc) => acc.read_mask(),
            AccessMode::Custom(bits) => {
                let mut mask = 0;
                for b in 0..8 {
                    mask |= match bits[b] {
                        Access::R | Access::RW | Access::U => 0,
                        Access::W => 1
                    } << b;
                }
                mask
            }
        }
    }

    pub fn wmask(&self) -> u8 {
        match self {
            AccessMode::Generic(acc) => acc.write_mask(),
            AccessMode::Custom(bits) => {
                let mut mask = 0;
                for b in 0..8 {
                    mask |= match bits[b] {
                        Access::W | Access::RW => 1,
                        Access::R | Access::U => 0
                    } << b;
                }
                mask
            }
        }
    }

    pub const fn wronly() -> Self { Self::Generic(Access::W) }
    pub const fn unused() -> Self { Self::Generic(Access::U) }
    pub const fn rdonly() -> Self { Self::Generic(Access::R) }
    pub const fn rw() -> Self { Self::Generic(Access::RW) }
}

impl Access {
    pub const fn read_mask(&self) -> u8 {
        match self {
            Access::R | Access::RW | Access::U => 0x00,
            Access::W => 0xFF,
        }
    }

    pub const fn write_mask(&self) -> u8 {
        match self {
            Access::W | Access::RW => 0xFF,
            Access::R | Access::U => 0x00,
        }
    }
}

#[derive(Clone)]
pub struct IOReg {
    pub(crate) v: u8,
    dirty: bool,
    rmask: u8,
    wmask: u8
}

impl Default for IOReg {
    fn default() -> Self { IOReg::new(AccessMode::unused()) }
}

impl Mem for IOReg {
    fn read(&self, addr: u16, _: u16) -> u8 {
        if addr != 0 { panic!("IO reg is only 1 byte") }
        self.read()
    }

    fn value(&self, addr: u16, _: u16) -> u8 {
        if addr != 0 { panic!("IO reg is only 1 byte") }
        self.value()
    }

    fn write(&mut self, addr: u16, value: u8, _absolute: u16) {
        if addr != 0 { panic!("IO reg is only 1 byte") }
        self.v = (self.v & !self.wmask) | (value & self.wmask);
        self.dirty = true;
    }
}

impl IOReg {
    pub fn new(access: AccessMode) -> Self {
        IOReg {
            v: 0,
            dirty: false,
            rmask: access.rmask(),
            wmask: access.wmask()
        }
    }

    pub fn direct_write(&mut self, value: u8) -> &mut Self {
        self.v = value; self
    }
    pub fn reset_dirty(&mut self) { self.dirty = false; }

    pub fn rdonly() -> Self { IOReg::new(AccessMode::rdonly()) }
    pub fn wronly() -> Self { IOReg::new(AccessMode::wronly()) }
    pub fn rw() -> Self { IOReg::new(AccessMode::rw()) }
    pub fn custom(bits: [Access; 8]) -> Self { IOReg::new(AccessMode::Custom(bits)) }
    pub fn with_access(mode: AccessMode) -> Self { IOReg::new(mode) }
    pub fn with_value(mut self, value: u8) -> Self { self.direct_write(value); self }
    pub fn unset() -> Self { IOReg::new(AccessMode::Generic(Access::U)) }

    pub const fn value(&self) -> u8 { self.v }

    pub fn reset(&mut self, bit: u8) {
        self.direct_write(self.value() & !(1 << bit));
    }
    pub fn set(&mut self, bit: u8) {
        self.direct_write(self.value() | (1 << bit));
    }
    pub const fn bit(&self, bit: u8) -> u8 {
        (self.value() >> bit) & 0x1
    }

    pub fn read(&self) -> u8 {
        self.v | self.rmask
    }

    pub fn dirty(&self) -> bool { self.dirty }

    pub const fn writable(&self) -> bool { self.wmask != 0 }

    pub fn set_access(&mut self, mode: AccessMode) -> &Self {
        self.wmask = mode.wmask();
        self.rmask = mode.rmask();
        self
    }
}
