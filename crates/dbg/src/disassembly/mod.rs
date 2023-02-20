use std::collections::{HashMap, VecDeque};
use std::ops::RangeBounds;
use egui_extras::{Column, TableBuilder};
use shared::cpu::{CBOpcode, Opcode, Reg};
use shared::egui;
use shared::egui::{Color32, Ui, Widget};
use shared::mem::{ROM, SROM};
use shared::utils::Cell;
use crate::Emulator;

mod dbg_opcodes;

#[derive(Clone)]
pub(crate) struct Op {
    pub size: usize,
    instruction: String,
    data: Vec<u8>
}

impl Op {
    fn new(size: usize, instruction: String, data: Vec<u8>) -> Self {
        Self { size, instruction, data }
    }

    pub fn parse(range: &[u8]) -> Self {
        let opcode = range[0];
        let op = match (opcode, false).try_into() {
            Ok(Opcode::PrefixCB) => { (range[1], true).try_into().unwrap() },
            Ok(opcode) => opcode,
            Err(e) => Opcode::Nop
        };
        let (sz, info) = dbg_opcodes::dbg_opcodes(op);
        Self::new(sz, info.to_string(), range[0..sz].to_vec())
    }

    pub fn is_call(&self) -> bool {
        self.instruction.contains("CALL") || self.instruction.contains("RST")
    }
}

impl Default for Op {
    fn default() -> Self {
        Op::new(1, dbg_opcodes::dbg_opcodes(Opcode::Nop).1.to_string(), vec![])
    }
}

struct OpRange {
    pub ops: Vec<Op>
}


impl OpRange {
    pub fn new() -> Self { Self { ops: vec![] } }

    // ignore range: start + skip || need to match start exactly to skip
    pub fn parse(&mut self, input: Vec<u8>, ignore: Vec<(usize, usize)>) {
        let mut st = 0;
        while st < input.len() {
            if let Some((_, len)) = ignore.iter().find(|(x, _)| x == st) {
                st += len;
                continue;
            }
            let op = Op::parse(&input[st..]);
            st += op.size;
            self.ops.push(op);
        }
    }
}

struct RomRange {
    ops: OpRange
}

impl RomRange {
    fn new() -> Self {
        Self { ops: OpRange::new() }
    }
    fn update(&mut self) {
        if !self.ops.ops.is_empty() { return }

    }
}

struct SromRange {
    banks: HashMap<u16, OpRange>
}

impl MemRange for RomRange {
    fn reload(&mut self) { self.ops.ops.clear(); }
    fn range(&self) -> std::ops::Range<u16> { (0..0x4000) }
    fn update<E: Emulator>(&mut self, emu: &E) {

    }
}

impl MemRange for SromRange {
    fn reload(&mut self) { self.banks.clear() }
    fn range(&self) -> std::ops::Range<u16> { (0x4000..0x8000) }
    fn update<E: Emulator>(&mut self, emu: &E) {
        let bank = emu.mbc()
    }
}

trait MemRange {
    fn reload(&mut self);
    fn range(&self) -> std::ops::Range<u16>;
    fn update<E: Emulator>(&mut self, emu: &E);
}

pub struct Disassembly {
    ranges: Vec<Box<dyn MemRange>>
}

impl Disassembly {
    pub fn new() -> Self {
        Self {
            ranges: vec![

            ]
        }
    }

    pub(crate) fn next(&self, emu: &impl Emulator) -> Option<(u16, Op)> {
        let pc = emu.cpu_register(Reg::PC).u16();
        if !self.range.contains(pc) {
            return None;
        }
        let mut st = self.range.st;
        for op in &self.range.ops {
            if st == pc { return Some((st, op.clone())); }
            st += op.size as u16;
        }
        None
    }

    pub fn reload(&mut self, emu: &impl Emulator) {

    }

    pub fn render<E: Emulator>(&mut self, emu: &E, ui: &mut Ui) {
        let pc = emu.cpu_register(Reg::PC).u16();
       if !self.range.contains(pc) {
            self.range.replace(pc, emu.get_range(pc, 32));
        }
        let mut table = TableBuilder::new(ui)
            .columns(Column::remainder(), 3)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center));
        table
            .header(20., |mut header| {
                header.col(|ui| {
                    ui.strong(egui::RichText::new("Address").color(Color32::GOLD));
                });
                header.col(|ui| {
                    ui.strong(egui::RichText::new("Instruction").color(Color32::GOLD));
                });
                header.col(|ui| {
                    ui.strong(egui::RichText::new("Parameters").color(Color32::GOLD));
                });
            })
            .body(|mut body| {
                let mut addr = self.range.st;
                for op in &self.range.ops {
                    let color = if pc >= addr && pc < addr + op.size as u16 { Color32::WHITE } else { Color32::DARK_GRAY };
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.label(egui::RichText::new(format!("{:#06X}", addr)).color(color));
                        });
                        row.col(|ui| {
                            ui.label(egui::RichText::new(&op.instruction).color(color));
                        });
                        row.col(|ui| {
                            let mut code = String::new();
                            for o in &op.data { code.push_str(format!(" {o:02X}").as_str()); }
                            ui.label(egui::RichText::new(code).color(color));
                        });
                        addr += op.size as u16;
                    });
                }
            });
    }
}
