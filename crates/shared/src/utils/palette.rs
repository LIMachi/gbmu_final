use serde::{Deserialize, Serialize};

const DMG_COLORS: [[u8; 3]; 4] = [[0xBF; 3], [0x7F; 3], [0x3F; 3], [0; 3]];

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub enum Palette {
    Dmg,
    Custom([[u8; 3]; 4]),
}

impl Default for Palette {
    fn default() -> Self { Self::Dmg }
}

impl Palette {
    pub fn color(&self, index: usize) -> [u8; 3] {
        match self {
            Palette::Dmg => DMG_COLORS[index],
            Palette::Custom(v) => v[index]
        }
    }
}
