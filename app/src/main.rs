use std::fs::File;
use std::io::Read;

pub struct FakeBus {
    rom: Vec<u8>,
    status: core::MemStatus
}

impl FakeBus {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            rom,
            status: core::MemStatus::ReqRead(0x100u16)
        }
    }
}


fn main() {
    let mut v = Vec::new();
    let mut file = File::open("roms/29459/29459.gbc").expect("not found");
    file.read_to_end(&mut v);
}
