use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod dbg;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid opcode")]
    Invalid
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u16)]
pub enum Opcode {
    Nop             = 0x0,
    LdBCd16         = 0x1,
    LdIndBCA        = 0x2,
    IncBC           = 0x3,
    IncB            = 0x4,
    DecB            = 0x5,
    LdBd8           = 0x6,
    Rlca            = 0x7,
    LdInda16SP      = 0x8,
    AddHLBC         = 0x9,
    LdAIndBC        = 0xa,
    DecBC           = 0xb,
    IncC            = 0xc,
    DecC            = 0xd,
    LdCd8           = 0xe,
    Rrca            = 0xf,
    Stop0           = 0x10,
    LdDEd16         = 0x11,
    LdIndDEA        = 0x12,
    IncDE           = 0x13,
    IncD            = 0x14,
    DecD            = 0x15,
    LdDd8           = 0x16,
    Rla             = 0x17,
    Jrr8            = 0x18,
    AddHLDE         = 0x19,
    LdAIndDE        = 0x1a,
    DecDE           = 0x1b,
    IncE            = 0x1c,
    DecE            = 0x1d,
    LdEd8           = 0x1e,
    Rra             = 0x1f,
    JrNZr8          = 0x20,
    LdHLd16         = 0x21,
    LdIndHLIncA     = 0x22,
    IncHL           = 0x23,
    IncH            = 0x24,
    DecH            = 0x25,
    LdHd8           = 0x26,
    Daa             = 0x27,
    JrZr8           = 0x28,
    AddHLHL         = 0x29,
    LdAIndHLInc     = 0x2a,
    DecHL           = 0x2b,
    IncL            = 0x2c,
    DecL            = 0x2d,
    LdLd8           = 0x2e,
    Cpl             = 0x2f,
    JrNCr8          = 0x30,
    LdSPd16         = 0x31,
    LdIndHLDecA     = 0x32,
    IncSP           = 0x33,
    IncIndHL        = 0x34,
    DecIndHL        = 0x35,
    LdIndHLd8       = 0x36,
    Scf             = 0x37,
    JrCr8           = 0x38,
    AddHLSP         = 0x39,
    LdAIndHLDec     = 0x3a,
    DecSP           = 0x3b,
    IncA            = 0x3c,
    DecA            = 0x3d,
    LdAd8           = 0x3e,
    Ccf             = 0x3f,
    LdBB            = 0x40,
    LdBC            = 0x41,
    LdBD            = 0x42,
    LdBE            = 0x43,
    LdBH            = 0x44,
    LdBL            = 0x45,
    LdBIndHL        = 0x46,
    LdBA            = 0x47,
    LdCB            = 0x48,
    LdCC            = 0x49,
    LdCD            = 0x4a,
    LdCE            = 0x4b,
    LdCH            = 0x4c,
    LdCL            = 0x4d,
    LdCIndHL        = 0x4e,
    LdCA            = 0x4f,
    LdDB            = 0x50,
    LdDC            = 0x51,
    LdDD            = 0x52,
    LdDE            = 0x53,
    LdDH            = 0x54,
    LdDL            = 0x55,
    LdDIndHL        = 0x56,
    LdDA            = 0x57,
    LdEB            = 0x58,
    LdEC            = 0x59,
    LdED            = 0x5a,
    LdEE            = 0x5b,
    LdEH            = 0x5c,
    LdEL            = 0x5d,
    LdEIndHL        = 0x5e,
    LdEA            = 0x5f,
    LdHB            = 0x60,
    LdHC            = 0x61,
    LdHD            = 0x62,
    LdHE            = 0x63,
    LdHH            = 0x64,
    LdHL            = 0x65,
    LdHIndHL        = 0x66,
    LdHA            = 0x67,
    LdLB            = 0x68,
    LdLC            = 0x69,
    LdLD            = 0x6a,
    LdLE            = 0x6b,
    LdLH            = 0x6c,
    LdLL            = 0x6d,
    LdLIndHL        = 0x6e,
    LdLA            = 0x6f,
    LdIndHLB        = 0x70,
    LdIndHLC        = 0x71,
    LdIndHLD        = 0x72,
    LdIndHLE        = 0x73,
    LdIndHLH        = 0x74,
    LdIndHLL        = 0x75,
    Halt            = 0x76,
    LdIndHLA        = 0x77,
    LdAB            = 0x78,
    LdAC            = 0x79,
    LdAD            = 0x7a,
    LdAE            = 0x7b,
    LdAH            = 0x7c,
    LdAL            = 0x7d,
    LdAIndHL        = 0x7e,
    LdAA            = 0x7f,
    AddAB           = 0x80,
    AddAC           = 0x81,
    AddAD           = 0x82,
    AddAE           = 0x83,
    AddAH           = 0x84,
    AddAL           = 0x85,
    AddAIndHL       = 0x86,
    AddAA           = 0x87,
    AdcAB           = 0x88,
    AdcAC           = 0x89,
    AdcAD           = 0x8a,
    AdcAE           = 0x8b,
    AdcAH           = 0x8c,
    AdcAL           = 0x8d,
    AdcAIndHL       = 0x8e,
    AdcAA           = 0x8f,
    SubB            = 0x90,
    SubC            = 0x91,
    SubD            = 0x92,
    SubE            = 0x93,
    SubH            = 0x94,
    SubL            = 0x95,
    SubIndHL        = 0x96,
    SubA            = 0x97,
    SbcAB           = 0x98,
    SbcAC           = 0x99,
    SbcAD           = 0x9a,
    SbcAE           = 0x9b,
    SbcAH           = 0x9c,
    SbcAL           = 0x9d,
    SbcAIndHL       = 0x9e,
    SbcAA           = 0x9f,
    AndB            = 0xa0,
    AndC            = 0xa1,
    AndD            = 0xa2,
    AndE            = 0xa3,
    AndH            = 0xa4,
    AndL            = 0xa5,
    AndIndHL        = 0xa6,
    AndA            = 0xa7,
    XorB            = 0xa8,
    XorC            = 0xa9,
    XorD            = 0xaa,
    XorE            = 0xab,
    XorH            = 0xac,
    XorL            = 0xad,
    XorIndHL        = 0xae,
    XorA            = 0xaf,
    OrB             = 0xb0,
    OrC             = 0xb1,
    OrD             = 0xb2,
    OrE             = 0xb3,
    OrH             = 0xb4,
    OrL             = 0xb5,
    OrIndHL         = 0xb6,
    OrA             = 0xb7,
    CpB             = 0xb8,
    CpC             = 0xb9,
    CpD             = 0xba,
    CpE             = 0xbb,
    CpH             = 0xbc,
    CpL             = 0xbd,
    CpIndHL         = 0xbe,
    CpA             = 0xbf,
    RetNZ           = 0xc0,
    PopBC           = 0xc1,
    JpNZa16         = 0xc2,
    Jpa16           = 0xc3,
    CallNZa16       = 0xc4,
    PushBC          = 0xc5,
    AddAd8          = 0xc6,
    Rst00H          = 0xc7,
    RetZ            = 0xc8,
    Ret             = 0xc9,
    JpZa16          = 0xca,
    PrefixCB        = 0xcb,
    CallZa16        = 0xcc,
    Calla16         = 0xcd,
    AdcAd8          = 0xce,
    Rst08H          = 0xcf,
    RetNC           = 0xd0,
    PopDE           = 0xd1,
    JpNCa16         = 0xd2,
    CallNCa16       = 0xd4,
    PushDE          = 0xd5,
    Subd8           = 0xd6,
    Rst10H          = 0xd7,
    RetC            = 0xd8,
    Reti            = 0xd9,
    JpCa16          = 0xda,
    CallCa16        = 0xdc,
    SbcAd8          = 0xde,
    Rst18H          = 0xdf,
    LdhInda8A       = 0xe0,
    PopHL           = 0xe1,
    LdIndCA         = 0xe2,
    PushHL          = 0xe5,
    Andd8           = 0xe6,
    Rst20H          = 0xe7,
    AddSPr8         = 0xe8,
    JpHL            = 0xe9,
    LdInda16A       = 0xea,
    Xord8           = 0xee,
    Rst28H          = 0xef,
    LdhAInda8       = 0xf0,
    PopAF           = 0xf1,
    LdAIndC         = 0xf2,
    Di              = 0xf3,
    PushAF          = 0xf5,
    Ord8            = 0xf6,
    Rst30H          = 0xf7,
    LdHLSPaddr8     = 0xf8,
    LdSPHL          = 0xf9,
    LdAInda16       = 0xfa,
    Ei              = 0xfb,
    Cpd8            = 0xfe,
    Rst38H          = 0xff,
    CB(CBOpcode)    = 0x100
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum CBOpcode {
    RlcB            = 0x0,
    RlcC            = 0x1,
    RlcD            = 0x2,
    RlcE            = 0x3,
    RlcH            = 0x4,
    RlcL            = 0x5,
    RlcIndHL        = 0x6,
    RlcA            = 0x7,
    RrcB            = 0x8,
    RrcC            = 0x9,
    RrcD            = 0xa,
    RrcE            = 0xb,
    RrcH            = 0xc,
    RrcL            = 0xd,
    RrcIndHL        = 0xe,
    RrcA            = 0xf,
    RlB             = 0x10,
    RlC             = 0x11,
    RlD             = 0x12,
    RlE             = 0x13,
    RlH             = 0x14,
    RlL             = 0x15,
    RlIndHL         = 0x16,
    RlA             = 0x17,
    RrB             = 0x18,
    RrC             = 0x19,
    RrD             = 0x1a,
    RrE             = 0x1b,
    RrH             = 0x1c,
    RrL             = 0x1d,
    RrIndHL         = 0x1e,
    RrA             = 0x1f,
    SlaB            = 0x20,
    SlaC            = 0x21,
    SlaD            = 0x22,
    SlaE            = 0x23,
    SlaH            = 0x24,
    SlaL            = 0x25,
    SlaIndHL        = 0x26,
    SlaA            = 0x27,
    SraB            = 0x28,
    SraC            = 0x29,
    SraD            = 0x2a,
    SraE            = 0x2b,
    SraH            = 0x2c,
    SraL            = 0x2d,
    SraIndHL        = 0x2e,
    SraA            = 0x2f,
    SwapB           = 0x30,
    SwapC           = 0x31,
    SwapD           = 0x32,
    SwapE           = 0x33,
    SwapH           = 0x34,
    SwapL           = 0x35,
    SwapIndHL       = 0x36,
    SwapA           = 0x37,
    SrlB            = 0x38,
    SrlC            = 0x39,
    SrlD            = 0x3a,
    SrlE            = 0x3b,
    SrlH            = 0x3c,
    SrlL            = 0x3d,
    SrlIndHL        = 0x3e,
    SrlA            = 0x3f,
    Bit0B           = 0x40,
    Bit0C           = 0x41,
    Bit0D           = 0x42,
    Bit0E           = 0x43,
    Bit0H           = 0x44,
    Bit0L           = 0x45,
    Bit0IndHL       = 0x46,
    Bit0A           = 0x47,
    Bit1B           = 0x48,
    Bit1C           = 0x49,
    Bit1D           = 0x4a,
    Bit1E           = 0x4b,
    Bit1H           = 0x4c,
    Bit1L           = 0x4d,
    Bit1IndHL       = 0x4e,
    Bit1A           = 0x4f,
    Bit2B           = 0x50,
    Bit2C           = 0x51,
    Bit2D           = 0x52,
    Bit2E           = 0x53,
    Bit2H           = 0x54,
    Bit2L           = 0x55,
    Bit2IndHL       = 0x56,
    Bit2A           = 0x57,
    Bit3B           = 0x58,
    Bit3C           = 0x59,
    Bit3D           = 0x5a,
    Bit3E           = 0x5b,
    Bit3H           = 0x5c,
    Bit3L           = 0x5d,
    Bit3IndHL       = 0x5e,
    Bit3A           = 0x5f,
    Bit4B           = 0x60,
    Bit4C           = 0x61,
    Bit4D           = 0x62,
    Bit4E           = 0x63,
    Bit4H           = 0x64,
    Bit4L           = 0x65,
    Bit4IndHL       = 0x66,
    Bit4A           = 0x67,
    Bit5B           = 0x68,
    Bit5C           = 0x69,
    Bit5D           = 0x6a,
    Bit5E           = 0x6b,
    Bit5H           = 0x6c,
    Bit5L           = 0x6d,
    Bit5IndHL       = 0x6e,
    Bit5A           = 0x6f,
    Bit6B           = 0x70,
    Bit6C           = 0x71,
    Bit6D           = 0x72,
    Bit6E           = 0x73,
    Bit6H           = 0x74,
    Bit6L           = 0x75,
    Bit6IndHL       = 0x76,
    Bit6A           = 0x77,
    Bit7B           = 0x78,
    Bit7C           = 0x79,
    Bit7D           = 0x7a,
    Bit7E           = 0x7b,
    Bit7H           = 0x7c,
    Bit7L           = 0x7d,
    Bit7IndHL       = 0x7e,
    Bit7A           = 0x7f,
    Res0B           = 0x80,
    Res0C           = 0x81,
    Res0D           = 0x82,
    Res0E           = 0x83,
    Res0H           = 0x84,
    Res0L           = 0x85,
    Res0IndHL       = 0x86,
    Res0A           = 0x87,
    Res1B           = 0x88,
    Res1C           = 0x89,
    Res1D           = 0x8a,
    Res1E           = 0x8b,
    Res1H           = 0x8c,
    Res1L           = 0x8d,
    Res1IndHL       = 0x8e,
    Res1A           = 0x8f,
    Res2B           = 0x90,
    Res2C           = 0x91,
    Res2D           = 0x92,
    Res2E           = 0x93,
    Res2H           = 0x94,
    Res2L           = 0x95,
    Res2IndHL       = 0x96,
    Res2A           = 0x97,
    Res3B           = 0x98,
    Res3C           = 0x99,
    Res3D           = 0x9a,
    Res3E           = 0x9b,
    Res3H           = 0x9c,
    Res3L           = 0x9d,
    Res3IndHL       = 0x9e,
    Res3A           = 0x9f,
    Res4B           = 0xa0,
    Res4C           = 0xa1,
    Res4D           = 0xa2,
    Res4E           = 0xa3,
    Res4H           = 0xa4,
    Res4L           = 0xa5,
    Res4IndHL       = 0xa6,
    Res4A           = 0xa7,
    Res5B           = 0xa8,
    Res5C           = 0xa9,
    Res5D           = 0xaa,
    Res5E           = 0xab,
    Res5H           = 0xac,
    Res5L           = 0xad,
    Res5IndHL       = 0xae,
    Res5A           = 0xaf,
    Res6B           = 0xb0,
    Res6C           = 0xb1,
    Res6D           = 0xb2,
    Res6E           = 0xb3,
    Res6H           = 0xb4,
    Res6L           = 0xb5,
    Res6IndHL       = 0xb6,
    Res6A           = 0xb7,
    Res7B           = 0xb8,
    Res7C           = 0xb9,
    Res7D           = 0xba,
    Res7E           = 0xbb,
    Res7H           = 0xbc,
    Res7L           = 0xbd,
    Res7IndHL       = 0xbe,
    Res7A           = 0xbf,
    Set0B           = 0xc0,
    Set0C           = 0xc1,
    Set0D           = 0xc2,
    Set0E           = 0xc3,
    Set0H           = 0xc4,
    Set0L           = 0xc5,
    Set0IndHL       = 0xc6,
    Set0A           = 0xc7,
    Set1B           = 0xc8,
    Set1C           = 0xc9,
    Set1D           = 0xca,
    Set1E           = 0xcb,
    Set1H           = 0xcc,
    Set1L           = 0xcd,
    Set1IndHL       = 0xce,
    Set1A           = 0xcf,
    Set2B           = 0xd0,
    Set2C           = 0xd1,
    Set2D           = 0xd2,
    Set2E           = 0xd3,
    Set2H           = 0xd4,
    Set2L           = 0xd5,
    Set2IndHL       = 0xd6,
    Set2A           = 0xd7,
    Set3B           = 0xd8,
    Set3C           = 0xd9,
    Set3D           = 0xda,
    Set3E           = 0xdb,
    Set3H           = 0xdc,
    Set3L           = 0xdd,
    Set3IndHL       = 0xde,
    Set3A           = 0xdf,
    Set4B           = 0xe0,
    Set4C           = 0xe1,
    Set4D           = 0xe2,
    Set4E           = 0xe3,
    Set4H           = 0xe4,
    Set4L           = 0xe5,
    Set4IndHL       = 0xe6,
    Set4A           = 0xe7,
    Set5B           = 0xe8,
    Set5C           = 0xe9,
    Set5D           = 0xea,
    Set5E           = 0xeb,
    Set5H           = 0xec,
    Set5L           = 0xed,
    Set5IndHL       = 0xee,
    Set5A           = 0xef,
    Set6B           = 0xf0,
    Set6C           = 0xf1,
    Set6D           = 0xf2,
    Set6E           = 0xf3,
    Set6H           = 0xf4,
    Set6L           = 0xf5,
    Set6IndHL       = 0xf6,
    Set6A           = 0xf7,
    Set7B           = 0xf8,
    Set7C           = 0xf9,
    Set7D           = 0xfa,
    Set7E           = 0xfb,
    Set7H           = 0xfc,
    Set7L           = 0xfd,
    Set7IndHL       = 0xfe,
    Set7A           = 0xff
}

impl TryFrom<(u8, bool)> for Opcode {
    type Error = Error;

    fn try_from((value, prefix): (u8, bool)) -> Result<Self, Self::Error> {
        if prefix { return Ok(Opcode::CB(CBOpcode::from(value))) }
        Ok(match value {
            0x0 => Opcode::Nop,
            0x1 => Opcode::LdBCd16,
            0x2 => Opcode::LdIndBCA,
            0x3 => Opcode::IncBC,
            0x4 => Opcode::IncB,
            0x5 => Opcode::DecB,
            0x6 => Opcode::LdBd8,
            0x7 => Opcode::Rlca,
            0x8 => Opcode::LdInda16SP,
            0x9 => Opcode::AddHLBC,
            0xa => Opcode::LdAIndBC,
            0xb => Opcode::DecBC,
            0xc => Opcode::IncC,
            0xd => Opcode::DecC,
            0xe => Opcode::LdCd8,
            0xf => Opcode::Rrca,
            0x10 => Opcode::Stop0,
            0x11 => Opcode::LdDEd16,
            0x12 => Opcode::LdIndDEA,
            0x13 => Opcode::IncDE,
            0x14 => Opcode::IncD,
            0x15 => Opcode::DecD,
            0x16 => Opcode::LdDd8,
            0x17 => Opcode::Rla,
            0x18 => Opcode::Jrr8,
            0x19 => Opcode::AddHLDE,
            0x1a => Opcode::LdAIndDE,
            0x1b => Opcode::DecDE,
            0x1c => Opcode::IncE,
            0x1d => Opcode::DecE,
            0x1e => Opcode::LdEd8,
            0x1f => Opcode::Rra,
            0x20 => Opcode::JrNZr8,
            0x21 => Opcode::LdHLd16,
            0x22 => Opcode::LdIndHLIncA,
            0x23 => Opcode::IncHL,
            0x24 => Opcode::IncH,
            0x25 => Opcode::DecH,
            0x26 => Opcode::LdHd8,
            0x27 => Opcode::Daa,
            0x28 => Opcode::JrZr8,
            0x29 => Opcode::AddHLHL,
            0x2a => Opcode::LdAIndHLInc,
            0x2b => Opcode::DecHL,
            0x2c => Opcode::IncL,
            0x2d => Opcode::DecL,
            0x2e => Opcode::LdLd8,
            0x2f => Opcode::Cpl,
            0x30 => Opcode::JrNCr8,
            0x31 => Opcode::LdSPd16,
            0x32 => Opcode::LdIndHLDecA,
            0x33 => Opcode::IncSP,
            0x34 => Opcode::IncIndHL,
            0x35 => Opcode::DecIndHL,
            0x36 => Opcode::LdIndHLd8,
            0x37 => Opcode::Scf,
            0x38 => Opcode::JrCr8,
            0x39 => Opcode::AddHLSP,
            0x3a => Opcode::LdAIndHLDec,
            0x3b => Opcode::DecSP,
            0x3c => Opcode::IncA,
            0x3d => Opcode::DecA,
            0x3e => Opcode::LdAd8,
            0x3f => Opcode::Ccf,
            0x40 => Opcode::LdBB,
            0x41 => Opcode::LdBC,
            0x42 => Opcode::LdBD,
            0x43 => Opcode::LdBE,
            0x44 => Opcode::LdBH,
            0x45 => Opcode::LdBL,
            0x46 => Opcode::LdBIndHL,
            0x47 => Opcode::LdBA,
            0x48 => Opcode::LdCB,
            0x49 => Opcode::LdCC,
            0x4a => Opcode::LdCD,
            0x4b => Opcode::LdCE,
            0x4c => Opcode::LdCH,
            0x4d => Opcode::LdCL,
            0x4e => Opcode::LdCIndHL,
            0x4f => Opcode::LdCA,
            0x50 => Opcode::LdDB,
            0x51 => Opcode::LdDC,
            0x52 => Opcode::LdDD,
            0x53 => Opcode::LdDE,
            0x54 => Opcode::LdDH,
            0x55 => Opcode::LdDL,
            0x56 => Opcode::LdDIndHL,
            0x57 => Opcode::LdDA,
            0x58 => Opcode::LdEB,
            0x59 => Opcode::LdEC,
            0x5a => Opcode::LdED,
            0x5b => Opcode::LdEE,
            0x5c => Opcode::LdEH,
            0x5d => Opcode::LdEL,
            0x5e => Opcode::LdEIndHL,
            0x5f => Opcode::LdEA,
            0x60 => Opcode::LdHB,
            0x61 => Opcode::LdHC,
            0x62 => Opcode::LdHD,
            0x63 => Opcode::LdHE,
            0x64 => Opcode::LdHH,
            0x65 => Opcode::LdHL,
            0x66 => Opcode::LdHIndHL,
            0x67 => Opcode::LdHA,
            0x68 => Opcode::LdLB,
            0x69 => Opcode::LdLC,
            0x6a => Opcode::LdLD,
            0x6b => Opcode::LdLE,
            0x6c => Opcode::LdLH,
            0x6d => Opcode::LdLL,
            0x6e => Opcode::LdLIndHL,
            0x6f => Opcode::LdLA,
            0x70 => Opcode::LdIndHLB,
            0x71 => Opcode::LdIndHLC,
            0x72 => Opcode::LdIndHLD,
            0x73 => Opcode::LdIndHLE,
            0x74 => Opcode::LdIndHLH,
            0x75 => Opcode::LdIndHLL,
            0x76 => Opcode::Halt,
            0x77 => Opcode::LdIndHLA,
            0x78 => Opcode::LdAB,
            0x79 => Opcode::LdAC,
            0x7a => Opcode::LdAD,
            0x7b => Opcode::LdAE,
            0x7c => Opcode::LdAH,
            0x7d => Opcode::LdAL,
            0x7e => Opcode::LdAIndHL,
            0x7f => Opcode::LdAA,
            0x80 => Opcode::AddAB,
            0x81 => Opcode::AddAC,
            0x82 => Opcode::AddAD,
            0x83 => Opcode::AddAE,
            0x84 => Opcode::AddAH,
            0x85 => Opcode::AddAL,
            0x86 => Opcode::AddAIndHL,
            0x87 => Opcode::AddAA,
            0x88 => Opcode::AdcAB,
            0x89 => Opcode::AdcAC,
            0x8a => Opcode::AdcAD,
            0x8b => Opcode::AdcAE,
            0x8c => Opcode::AdcAH,
            0x8d => Opcode::AdcAL,
            0x8e => Opcode::AdcAIndHL,
            0x8f => Opcode::AdcAA,
            0x90 => Opcode::SubB,
            0x91 => Opcode::SubC,
            0x92 => Opcode::SubD,
            0x93 => Opcode::SubE,
            0x94 => Opcode::SubH,
            0x95 => Opcode::SubL,
            0x96 => Opcode::SubIndHL,
            0x97 => Opcode::SubA,
            0x98 => Opcode::SbcAB,
            0x99 => Opcode::SbcAC,
            0x9a => Opcode::SbcAD,
            0x9b => Opcode::SbcAE,
            0x9c => Opcode::SbcAH,
            0x9d => Opcode::SbcAL,
            0x9e => Opcode::SbcAIndHL,
            0x9f => Opcode::SbcAA,
            0xa0 => Opcode::AndB,
            0xa1 => Opcode::AndC,
            0xa2 => Opcode::AndD,
            0xa3 => Opcode::AndE,
            0xa4 => Opcode::AndH,
            0xa5 => Opcode::AndL,
            0xa6 => Opcode::AndIndHL,
            0xa7 => Opcode::AndA,
            0xa8 => Opcode::XorB,
            0xa9 => Opcode::XorC,
            0xaa => Opcode::XorD,
            0xab => Opcode::XorE,
            0xac => Opcode::XorH,
            0xad => Opcode::XorL,
            0xae => Opcode::XorIndHL,
            0xaf => Opcode::XorA,
            0xb0 => Opcode::OrB,
            0xb1 => Opcode::OrC,
            0xb2 => Opcode::OrD,
            0xb3 => Opcode::OrE,
            0xb4 => Opcode::OrH,
            0xb5 => Opcode::OrL,
            0xb6 => Opcode::OrIndHL,
            0xb7 => Opcode::OrA,
            0xb8 => Opcode::CpB,
            0xb9 => Opcode::CpC,
            0xba => Opcode::CpD,
            0xbb => Opcode::CpE,
            0xbc => Opcode::CpH,
            0xbd => Opcode::CpL,
            0xbe => Opcode::CpIndHL,
            0xbf => Opcode::CpA,
            0xc0 => Opcode::RetNZ,
            0xc1 => Opcode::PopBC,
            0xc2 => Opcode::JpNZa16,
            0xc3 => Opcode::Jpa16,
            0xc4 => Opcode::CallNZa16,
            0xc5 => Opcode::PushBC,
            0xc6 => Opcode::AddAd8,
            0xc7 => Opcode::Rst00H,
            0xc8 => Opcode::RetZ,
            0xc9 => Opcode::Ret,
            0xca => Opcode::JpZa16,
            0xcb => Opcode::PrefixCB,
            0xcc => Opcode::CallZa16,
            0xcd => Opcode::Calla16,
            0xce => Opcode::AdcAd8,
            0xcf => Opcode::Rst08H,
            0xd0 => Opcode::RetNC,
            0xd1 => Opcode::PopDE,
            0xd2 => Opcode::JpNCa16,
            0xd4 => Opcode::CallNCa16,
            0xd5 => Opcode::PushDE,
            0xd6 => Opcode::Subd8,
            0xd7 => Opcode::Rst10H,
            0xd8 => Opcode::RetC,
            0xd9 => Opcode::Reti,
            0xda => Opcode::JpCa16,
            0xdc => Opcode::CallCa16,
            0xde => Opcode::SbcAd8,
            0xdf => Opcode::Rst18H,
            0xe0 => Opcode::LdhInda8A,
            0xe1 => Opcode::PopHL,
            0xe2 => Opcode::LdIndCA,
            0xe5 => Opcode::PushHL,
            0xe6 => Opcode::Andd8,
            0xe7 => Opcode::Rst20H,
            0xe8 => Opcode::AddSPr8,
            0xe9 => Opcode::JpHL,
            0xea => Opcode::LdInda16A,
            0xee => Opcode::Xord8,
            0xef => Opcode::Rst28H,
            0xf0 => Opcode::LdhAInda8,
            0xf1 => Opcode::PopAF,
            0xf2 => Opcode::LdAIndC,
            0xf3 => Opcode::Di,
            0xf5 => Opcode::PushAF,
            0xf6 => Opcode::Ord8,
            0xf7 => Opcode::Rst30H,
            0xf8 => Opcode::LdHLSPaddr8,
            0xf9 => Opcode::LdSPHL,
            0xfa => Opcode::LdAInda16,
            0xfb => Opcode::Ei,
            0xfe => Opcode::Cpd8,
            0xff => Opcode::Rst38H,
            _ => return Err(Error::Invalid)
        })
    }
}

impl From<u8> for CBOpcode {
    fn from(value: u8) -> Self {
        match value {
            0x0 => CBOpcode::RlcB,
            0x1 => CBOpcode::RlcC,
            0x2 => CBOpcode::RlcD,
            0x3 => CBOpcode::RlcE,
            0x4 => CBOpcode::RlcH,
            0x5 => CBOpcode::RlcL,
            0x6 => CBOpcode::RlcIndHL,
            0x7 => CBOpcode::RlcA,
            0x8 => CBOpcode::RrcB,
            0x9 => CBOpcode::RrcC,
            0xa => CBOpcode::RrcD,
            0xb => CBOpcode::RrcE,
            0xc => CBOpcode::RrcH,
            0xd => CBOpcode::RrcL,
            0xe => CBOpcode::RrcIndHL,
            0xf => CBOpcode::RrcA,
            0x10 => CBOpcode::RlB,
            0x11 => CBOpcode::RlC,
            0x12 => CBOpcode::RlD,
            0x13 => CBOpcode::RlE,
            0x14 => CBOpcode::RlH,
            0x15 => CBOpcode::RlL,
            0x16 => CBOpcode::RlIndHL,
            0x17 => CBOpcode::RlA,
            0x18 => CBOpcode::RrB,
            0x19 => CBOpcode::RrC,
            0x1a => CBOpcode::RrD,
            0x1b => CBOpcode::RrE,
            0x1c => CBOpcode::RrH,
            0x1d => CBOpcode::RrL,
            0x1e => CBOpcode::RrIndHL,
            0x1f => CBOpcode::RrA,
            0x20 => CBOpcode::SlaB,
            0x21 => CBOpcode::SlaC,
            0x22 => CBOpcode::SlaD,
            0x23 => CBOpcode::SlaE,
            0x24 => CBOpcode::SlaH,
            0x25 => CBOpcode::SlaL,
            0x26 => CBOpcode::SlaIndHL,
            0x27 => CBOpcode::SlaA,
            0x28 => CBOpcode::SraB,
            0x29 => CBOpcode::SraC,
            0x2a => CBOpcode::SraD,
            0x2b => CBOpcode::SraE,
            0x2c => CBOpcode::SraH,
            0x2d => CBOpcode::SraL,
            0x2e => CBOpcode::SraIndHL,
            0x2f => CBOpcode::SraA,
            0x30 => CBOpcode::SwapB,
            0x31 => CBOpcode::SwapC,
            0x32 => CBOpcode::SwapD,
            0x33 => CBOpcode::SwapE,
            0x34 => CBOpcode::SwapH,
            0x35 => CBOpcode::SwapL,
            0x36 => CBOpcode::SwapIndHL,
            0x37 => CBOpcode::SwapA,
            0x38 => CBOpcode::SrlB,
            0x39 => CBOpcode::SrlC,
            0x3a => CBOpcode::SrlD,
            0x3b => CBOpcode::SrlE,
            0x3c => CBOpcode::SrlH,
            0x3d => CBOpcode::SrlL,
            0x3e => CBOpcode::SrlIndHL,
            0x3f => CBOpcode::SrlA,
            0x40 => CBOpcode::Bit0B,
            0x41 => CBOpcode::Bit0C,
            0x42 => CBOpcode::Bit0D,
            0x43 => CBOpcode::Bit0E,
            0x44 => CBOpcode::Bit0H,
            0x45 => CBOpcode::Bit0L,
            0x46 => CBOpcode::Bit0IndHL,
            0x47 => CBOpcode::Bit0A,
            0x48 => CBOpcode::Bit1B,
            0x49 => CBOpcode::Bit1C,
            0x4a => CBOpcode::Bit1D,
            0x4b => CBOpcode::Bit1E,
            0x4c => CBOpcode::Bit1H,
            0x4d => CBOpcode::Bit1L,
            0x4e => CBOpcode::Bit1IndHL,
            0x4f => CBOpcode::Bit1A,
            0x50 => CBOpcode::Bit2B,
            0x51 => CBOpcode::Bit2C,
            0x52 => CBOpcode::Bit2D,
            0x53 => CBOpcode::Bit2E,
            0x54 => CBOpcode::Bit2H,
            0x55 => CBOpcode::Bit2L,
            0x56 => CBOpcode::Bit2IndHL,
            0x57 => CBOpcode::Bit2A,
            0x58 => CBOpcode::Bit3B,
            0x59 => CBOpcode::Bit3C,
            0x5a => CBOpcode::Bit3D,
            0x5b => CBOpcode::Bit3E,
            0x5c => CBOpcode::Bit3H,
            0x5d => CBOpcode::Bit3L,
            0x5e => CBOpcode::Bit3IndHL,
            0x5f => CBOpcode::Bit3A,
            0x60 => CBOpcode::Bit4B,
            0x61 => CBOpcode::Bit4C,
            0x62 => CBOpcode::Bit4D,
            0x63 => CBOpcode::Bit4E,
            0x64 => CBOpcode::Bit4H,
            0x65 => CBOpcode::Bit4L,
            0x66 => CBOpcode::Bit4IndHL,
            0x67 => CBOpcode::Bit4A,
            0x68 => CBOpcode::Bit5B,
            0x69 => CBOpcode::Bit5C,
            0x6a => CBOpcode::Bit5D,
            0x6b => CBOpcode::Bit5E,
            0x6c => CBOpcode::Bit5H,
            0x6d => CBOpcode::Bit5L,
            0x6e => CBOpcode::Bit5IndHL,
            0x6f => CBOpcode::Bit5A,
            0x70 => CBOpcode::Bit6B,
            0x71 => CBOpcode::Bit6C,
            0x72 => CBOpcode::Bit6D,
            0x73 => CBOpcode::Bit6E,
            0x74 => CBOpcode::Bit6H,
            0x75 => CBOpcode::Bit6L,
            0x76 => CBOpcode::Bit6IndHL,
            0x77 => CBOpcode::Bit6A,
            0x78 => CBOpcode::Bit7B,
            0x79 => CBOpcode::Bit7C,
            0x7a => CBOpcode::Bit7D,
            0x7b => CBOpcode::Bit7E,
            0x7c => CBOpcode::Bit7H,
            0x7d => CBOpcode::Bit7L,
            0x7e => CBOpcode::Bit7IndHL,
            0x7f => CBOpcode::Bit7A,
            0x80 => CBOpcode::Res0B,
            0x81 => CBOpcode::Res0C,
            0x82 => CBOpcode::Res0D,
            0x83 => CBOpcode::Res0E,
            0x84 => CBOpcode::Res0H,
            0x85 => CBOpcode::Res0L,
            0x86 => CBOpcode::Res0IndHL,
            0x87 => CBOpcode::Res0A,
            0x88 => CBOpcode::Res1B,
            0x89 => CBOpcode::Res1C,
            0x8a => CBOpcode::Res1D,
            0x8b => CBOpcode::Res1E,
            0x8c => CBOpcode::Res1H,
            0x8d => CBOpcode::Res1L,
            0x8e => CBOpcode::Res1IndHL,
            0x8f => CBOpcode::Res1A,
            0x90 => CBOpcode::Res2B,
            0x91 => CBOpcode::Res2C,
            0x92 => CBOpcode::Res2D,
            0x93 => CBOpcode::Res2E,
            0x94 => CBOpcode::Res2H,
            0x95 => CBOpcode::Res2L,
            0x96 => CBOpcode::Res2IndHL,
            0x97 => CBOpcode::Res2A,
            0x98 => CBOpcode::Res3B,
            0x99 => CBOpcode::Res3C,
            0x9a => CBOpcode::Res3D,
            0x9b => CBOpcode::Res3E,
            0x9c => CBOpcode::Res3H,
            0x9d => CBOpcode::Res3L,
            0x9e => CBOpcode::Res3IndHL,
            0x9f => CBOpcode::Res3A,
            0xa0 => CBOpcode::Res4B,
            0xa1 => CBOpcode::Res4C,
            0xa2 => CBOpcode::Res4D,
            0xa3 => CBOpcode::Res4E,
            0xa4 => CBOpcode::Res4H,
            0xa5 => CBOpcode::Res4L,
            0xa6 => CBOpcode::Res4IndHL,
            0xa7 => CBOpcode::Res4A,
            0xa8 => CBOpcode::Res5B,
            0xa9 => CBOpcode::Res5C,
            0xaa => CBOpcode::Res5D,
            0xab => CBOpcode::Res5E,
            0xac => CBOpcode::Res5H,
            0xad => CBOpcode::Res5L,
            0xae => CBOpcode::Res5IndHL,
            0xaf => CBOpcode::Res5A,
            0xb0 => CBOpcode::Res6B,
            0xb1 => CBOpcode::Res6C,
            0xb2 => CBOpcode::Res6D,
            0xb3 => CBOpcode::Res6E,
            0xb4 => CBOpcode::Res6H,
            0xb5 => CBOpcode::Res6L,
            0xb6 => CBOpcode::Res6IndHL,
            0xb7 => CBOpcode::Res6A,
            0xb8 => CBOpcode::Res7B,
            0xb9 => CBOpcode::Res7C,
            0xba => CBOpcode::Res7D,
            0xbb => CBOpcode::Res7E,
            0xbc => CBOpcode::Res7H,
            0xbd => CBOpcode::Res7L,
            0xbe => CBOpcode::Res7IndHL,
            0xbf => CBOpcode::Res7A,
            0xc0 => CBOpcode::Set0B,
            0xc1 => CBOpcode::Set0C,
            0xc2 => CBOpcode::Set0D,
            0xc3 => CBOpcode::Set0E,
            0xc4 => CBOpcode::Set0H,
            0xc5 => CBOpcode::Set0L,
            0xc6 => CBOpcode::Set0IndHL,
            0xc7 => CBOpcode::Set0A,
            0xc8 => CBOpcode::Set1B,
            0xc9 => CBOpcode::Set1C,
            0xca => CBOpcode::Set1D,
            0xcb => CBOpcode::Set1E,
            0xcc => CBOpcode::Set1H,
            0xcd => CBOpcode::Set1L,
            0xce => CBOpcode::Set1IndHL,
            0xcf => CBOpcode::Set1A,
            0xd0 => CBOpcode::Set2B,
            0xd1 => CBOpcode::Set2C,
            0xd2 => CBOpcode::Set2D,
            0xd3 => CBOpcode::Set2E,
            0xd4 => CBOpcode::Set2H,
            0xd5 => CBOpcode::Set2L,
            0xd6 => CBOpcode::Set2IndHL,
            0xd7 => CBOpcode::Set2A,
            0xd8 => CBOpcode::Set3B,
            0xd9 => CBOpcode::Set3C,
            0xda => CBOpcode::Set3D,
            0xdb => CBOpcode::Set3E,
            0xdc => CBOpcode::Set3H,
            0xdd => CBOpcode::Set3L,
            0xde => CBOpcode::Set3IndHL,
            0xdf => CBOpcode::Set3A,
            0xe0 => CBOpcode::Set4B,
            0xe1 => CBOpcode::Set4C,
            0xe2 => CBOpcode::Set4D,
            0xe3 => CBOpcode::Set4E,
            0xe4 => CBOpcode::Set4H,
            0xe5 => CBOpcode::Set4L,
            0xe6 => CBOpcode::Set4IndHL,
            0xe7 => CBOpcode::Set4A,
            0xe8 => CBOpcode::Set5B,
            0xe9 => CBOpcode::Set5C,
            0xea => CBOpcode::Set5D,
            0xeb => CBOpcode::Set5E,
            0xec => CBOpcode::Set5H,
            0xed => CBOpcode::Set5L,
            0xee => CBOpcode::Set5IndHL,
            0xef => CBOpcode::Set5A,
            0xf0 => CBOpcode::Set6B,
            0xf1 => CBOpcode::Set6C,
            0xf2 => CBOpcode::Set6D,
            0xf3 => CBOpcode::Set6E,
            0xf4 => CBOpcode::Set6H,
            0xf5 => CBOpcode::Set6L,
            0xf6 => CBOpcode::Set6IndHL,
            0xf7 => CBOpcode::Set6A,
            0xf8 => CBOpcode::Set7B,
            0xf9 => CBOpcode::Set7C,
            0xfa => CBOpcode::Set7D,
            0xfb => CBOpcode::Set7E,
            0xfc => CBOpcode::Set7H,
            0xfd => CBOpcode::Set7L,
            0xfe => CBOpcode::Set7IndHL,
            0xff => CBOpcode::Set7A,
            _ => unreachable!() // can't get anything else than 0..0xFF from an u8
        }
    }
}
