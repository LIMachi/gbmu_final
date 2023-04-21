use std::cell::RefCell;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

pub mod image;
pub mod clock;
pub mod rtc;
pub mod convert;
pub mod palette;

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
