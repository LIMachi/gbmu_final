use std::collections::VecDeque;
use egui_extras::{Column, TableBuilder};
use shared::cpu::{CBOpcode, Opcode, Reg};
use shared::egui;
use shared::egui::{Color32, Ui, Widget};
use shared::utils::Cell;
use crate::Emulator;

mod dbg_opcodes;

#[derive(Clone)]
struct Op {
    size: usize,
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
}

impl Default for Op {
    fn default() -> Self {
        Op::new(1, dbg_opcodes::dbg_opcodes(Opcode::Nop).1.to_string(), vec![])
    }
}

struct OpRange {
    st: u16,
    len: usize,
    ops: VecDeque<Op>
}

impl OpRange {
    pub fn empty() -> Self {
        Self { st: 0xFFFF, len: 0, ops: VecDeque::with_capacity(8) }
    }

    pub fn pop(&mut self) -> Option<(u16, Op)> {
        self.ops.pop_front().map(|x| {
            let pc = self.st;
            self.st += x.size as u16;
            self.len -= x.size;
            (pc, x)
        })
    }

    pub fn push(&mut self, op: Op) {
        self.ops.push_back(op);
    }

    pub fn replace(&mut self, pc: u16, range: Vec<u8>) {
        self.ops.clear();
        let mut off = 0;
        while self.ops.len() < 8 && off < range.len() {
            let op = Op::parse(&range[off..]);
            off += op.size;
            self.ops.push_back(op);
        }
        for i in self.ops.len()..8 {
            self.ops.push_back(Op::default());
        }
        self.st = pc;
        self.len = off;
    }

    pub fn next_pc(&self) -> u16 {
        self.st + self.len as u16
    }

    pub fn contains(&self, pc: u16) -> bool {
        pc >= self.st && pc < (self.st + self.len as u16)
    }
}

pub struct Disassembly {
    range: OpRange
}

impl Disassembly {
    pub fn new() -> Self { Self { range: OpRange::empty() } }

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
