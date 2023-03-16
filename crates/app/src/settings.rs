use std::cell::RefCell;
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use shared::egui::{CentralPanel, Context, Ui};
use crate::Keybindings;

pub struct Settings {
    bindings: Rc<RefCell<Keybindings>>,
    cgb: Rc<RefCell<Mode>>
}

impl Settings {
    pub fn new(bindings: Rc<RefCell<Keybindings>>, cgb: Rc<RefCell<Mode>>) -> Self {
        Self {
            bindings,
            cgb
        }
    }
}

// TODO mode auto ?
#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Mode {
    Dmg,
    Cgb
}

impl Default for Mode {
    fn default() -> Self {
        Self::Dmg
    }
}

impl Mode {
    pub fn is_cgb(&self) -> bool {
        match self {
            Mode::Dmg => false,
            Mode::Cgb => true
        }
    }
}

impl shared::Ui for Settings {
    fn init(&mut self, _ctx: &mut Context) {
    }

    fn draw(&mut self, ctx: &mut Context) {
        CentralPanel::default()
            .show(ctx, |ui: &mut Ui| {
                let mut value = *self.cgb.as_ref().borrow();
                ui.radio_value(&mut value, Mode::Dmg, format!("{:?}", Mode::Dmg));
                ui.radio_value(&mut value, Mode::Cgb, format!("{:?}", Mode::Cgb));
                self.cgb.replace(value);
            });
    }
}
