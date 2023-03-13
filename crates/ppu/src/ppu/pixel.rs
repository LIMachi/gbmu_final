#[derive(Default, Copy, Clone, Debug)]
pub struct Attributes(pub u8);

impl Attributes {
    pub fn priority(&self) -> bool { self.0 & 0x80 != 0 }
    pub fn flip_y(&self) -> bool { self.0 & 0x40 != 0 }
    pub fn flip_x(&self) -> bool { self.0 & 0x20 != 0 }
    pub fn obp1(&self) -> bool { self.0 & 0x10 != 0 }
    pub fn bank(&self) -> usize { ((self.0 >> 3) & 0x1) as usize }
    pub fn palette(&self) -> usize { (self.0 & 0x7) as usize }
}

#[derive(Copy, Clone)]
pub struct Pixel {
    pub color: u8,
    pub index: Option<u8>,
    pub attrs: Attributes,
    pub sprite: bool
}

impl Pixel {

    pub fn sprite(color: u8, index: u8, attrs: Attributes) -> Self {
        Self {
            color,
            index: Some(index),
            attrs,
            sprite: true
        }
    }

    pub fn bg(color: u8, attrs: Attributes) -> Self {
        Self {
            color,
            index: None,
            attrs,
            sprite: false
        }
    }

    /// sprite priority mix
    pub fn mix(&mut self, rhs: Pixel) {
        *self = match (self.color, rhs.color, self.index, rhs.index) {
            (_, _, None, Some(_)) => rhs,
            (_, _, Some(_), None ) => *self,
            (_, 0, ..) => *self,
            (0, ..) => rhs,
            (_, _, Some(x1), Some(x2) ) if x1 < x2 => *self,
            (_, _, Some(x1), Some(x2) ) if x1 > x2 => rhs,
            _ => *self,
        }
    }
}
