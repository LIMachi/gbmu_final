#![feature(slice_flatten)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(if_let_guard)]

extern crate core;

use std::collections::HashMap;
use std::marker::PhantomData;
use bincode::Options;
use serde::{Deserializer, Serializer};

pub use dma::Dma;
pub use hdma::Hdma;
use lcd::Lcd;
use mem::{Oam, Vram};
pub use ppu::Ppu;
pub use render::{PpuAccess, VramAccess, VramViewer};
use shared::emulator::Emulator;
use shared::io::{IO, IODevice, IORegs};
use shared::mem::{IOBus, Lock};
use shared::serde::{Deserialize, Serialize};
use crate::ppu::states::{HState, Mode, OamState, TransferState, VState};

mod render;
mod ppu;

mod dma;
mod hdma;

#[derive(Serialize, Deserialize)]
pub struct Controller {
    ppu: Ppu,
    state: ppu::PpuState,
}

#[derive(Serialize, Deserialize)]
struct InnerPpuState {
    mode: Mode,
    raw: Vec<u8>
}

impl Serialize for ppu::PpuState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mode = self.mode();
        let raw = self.raw();
        InnerPpuState{mode, raw}.serialize(serializer)
    }
}

impl <'de> Deserialize<'de> for ppu::PpuState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        Deserialize::deserialize(deserializer).map(|InnerPpuState{mode, raw}| {
            match mode {
                Mode::Search => OamState::from_raw(raw),
                Mode::Transfer => TransferState::from_raw(raw),
                Mode::HBlank => HState::from_raw(raw),
                Mode::VBlank => VState::from_raw(raw)
            }
        })
    }
}

impl Controller {
    pub fn new() -> Self {
        Self {
            ppu: Ppu::new(),
            state: Ppu::default_state(),
        }
    }

    pub fn tick<'a>(&mut self, io: &mut IORegs, oam: &'a mut Lock<Oam>, vram: &'a mut Lock<Vram>, lcd: &mut Lcd) {
        self.ppu.claim(oam, vram);
        self.ppu.tick(&mut self.state, io, lcd);
        self.ppu.release();
    }

    pub fn inner(&self) -> &Ppu { &self.ppu }

    pub fn can_serde(&self) -> bool {
        self.state.first_tick()
    }
}

impl IODevice for Controller {
    fn write(&mut self, io: IO, v: u8, bus: &mut dyn IOBus) {
        IODevice::write(&mut self.ppu, io, v, bus);
    }
}
