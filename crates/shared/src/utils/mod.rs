use std::cell::RefCell;
use std::rc::Rc;

use egui::Color32;

pub mod image;
pub mod clock;
pub mod rtc;
pub mod convert;
pub mod palette;

pub const DARK_BLACK: Color32 = Color32::from_rgb(0x23, 0x27, 0x2A);

pub trait ToBox {
    fn boxed(&self) -> Box<&Self> where Self: Sized;
}

impl<T: Sized> ToBox for T {
    fn boxed(&self) -> Box<&Self> where Self: Sized {
        Box::new(self)
    }
}

pub trait Cell {
    fn cell(self) -> Rc<RefCell<Self>>;
}

impl<T> Cell for T {
    fn cell(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }
}

#[derive(Default)]
pub struct FEdge {
    old: bool,
}

impl FEdge {
    pub fn tick(&mut self, v: bool) -> bool {
        let r = self.old && !v;
        self.old = v;
        r
    }
}
