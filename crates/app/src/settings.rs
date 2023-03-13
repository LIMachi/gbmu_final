use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use shared::egui::{CentralPanel, Context, TextureHandle, Ui};
use crate::Keybindings;

#[derive(Copy, Clone, Hash, PartialOrd, PartialEq, Eq)]
pub enum Texture {
    GameBoy,
    ColorGameBoy
}

pub struct Settings {
    bindings: Rc<RefCell<Keybindings>>,
    cgb: Rc<RefCell<Mode>>,
    textures: HashMap<Texture, TextureHandle>
}

impl Settings {
    pub fn new(bindings: Rc<RefCell<Keybindings>>, cgb: Rc<RefCell<Mode>>) -> Self {
        Self {
            bindings,
            cgb,
            textures: Default::default(),
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
    fn init(&mut self, ctx: &mut Context) {
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
