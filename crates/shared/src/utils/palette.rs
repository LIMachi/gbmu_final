use serde::{Deserialize, Serialize};

const GRAY_SCALE: [[u8; 3]; 4] = [[0xBF; 3], [0x7F; 3], [0x3F; 3], [0; 3]];
const ORIGINAL: [[u8; 3]; 4] = [[224, 248, 208], [136, 192, 112], [52, 104, 86], [8, 24, 32]];

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub enum Palette {
    GrayScale,
    Original,
    Custom([[u8; 3]; 4]),
}

impl PartialEq for Palette {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..3 {
            if self.color(i) != other.color(i) { return false; }
        }
        true
    }
}

impl Default for Palette {
    fn default() -> Self { Self::GrayScale }
}

impl Palette {
    pub fn color(&self, index: u8) -> [u8; 3] {
        match self {
            Palette::GrayScale => GRAY_SCALE[index as usize],
            Palette::Original => ORIGINAL[index as usize],
            Palette::Custom(v) => v[index as usize],
        }
    }

    pub fn to_f32(&self) -> [[f32; 3]; 4] {
        let mut o = [[0f32; 3]; 4];
        for i in 0..4 {
            let t = self.color(i as u8);
            for j in 0..3 {
                o[i][j] = t[j] as f32 / 255f32;
            }
        }
        o
    }

    pub fn from_f32(colors: [[f32; 3]; 4]) -> Self {
        let mut t = [[0u8; 3]; 4];
        for i in 0..4 {
            for j in 0..3 {
                t[i][j] = (colors[i][j] * 255f32) as u8;
            }
        }
        Palette::Custom(t)
    }
}
