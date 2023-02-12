use std::cell::RefCell;
use std::rc::Rc;

pub mod image;
pub mod clock;

pub trait Cell {
    fn cell(self) -> Rc<RefCell<Self>>;
}

impl<T> Cell for T {
    fn cell(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }
}
