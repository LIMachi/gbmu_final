use std::cell::RefMut;
use std::collections::HashMap;
use std::ops::{Range, RangeBounds};
use egui_extras::{Column, TableBuilder};
use shared::breakpoints::{Break, Breakpoint, Breakpoints};
use shared::cpu::{dbg, Opcode, Reg};
use shared::egui;
use shared::egui::{Align, Color32, Rounding, Sense, Ui, Vec2};
use shared::mem::*;
use crate::Emulator;

#[derive(Clone)]
pub struct Op {
    pub offset: u16,
    pub size: usize,
    pub instruction: String,
    data: Vec<u8>
}

impl Op {
    fn new(offset: u16, size: usize, instruction: String, data: Vec<u8>) -> Self {
        Self { offset, size, instruction, data }
    }

    pub fn parse(pc: u16, range: &[u8]) -> Self {
        let opcode = range[0];
        let op = match (opcode, false).try_into() {
            Ok(Opcode::PrefixCB) => { (range[1], true).try_into().unwrap() },
            Ok(opcode) => opcode,
            Err(_e) => Opcode::Nop
        };
        let (sz, info) = dbg::dbg_opcodes(op);
        Self::new(pc, sz, info.to_string(), if range.len() < sz {
            vec![0xFF, range.len() as u8]
        } else {
            range[0..sz].to_vec()
        })
    }

    pub fn is_call(&self) -> bool {
        self.instruction.contains("CALL") || self.instruction.contains("RST")
    }
}

impl Default for Op {
    fn default() -> Self {
        Op::new(0, 1, dbg::dbg_opcodes(Opcode::Nop).1.to_string(), vec![])
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Cursor {
    Fixed(usize),
    Follow
}

pub struct OpRange {
    pub ops: Vec<Op>
}

impl Default for OpRange {
    fn default() -> Self {
        Self { ops: Vec::with_capacity(8192) }
    }
}

impl OpRange {
    // ignore range: start + skip || need to match start exactly to skip
    pub fn parse(mut self, input: Vec<u8>, ignore: Vec<(usize, usize)>) -> Self {
        self.ops.clear();
        let mut st = 0;
        while st < input.len() {
            if let Some((_, len)) = ignore.iter().find(|(x, _)| *x == st).copied() {
                self.ops.push(Op {
                    offset: st as u16,
                    size: len as usize,
                    instruction: "..".to_string(),
                    data: input[st..(st + (len as usize).min(8))].to_vec()
                });
                st += len;
                continue;
            }
            let op = Op::parse(st as u16, &input[st..]);
            st += op.size;
            self.ops.push(op);
        }
        self
    }
}

#[derive(Default)]
struct RomRange(OpRange);

struct RawRange(u16, u16, OpRange);

struct DynRange(u16, u16, OpRange);

impl DynRange {
    pub fn new(st: u16, end: u16) -> Self { Self(st, end - st + 1, OpRange::default()) }
}

impl<E: Emulator> MemRange<E> for DynRange {
    fn reload(&mut self) { }
    fn range(&self) -> Range<u16> {
        self.0..self.1
    }
    fn update(&mut self, emu: &E) {
        self.2 = OpRange::default().parse(emu.get_range(self.0, self.1), vec![]);
    }
    fn ops(&self) -> &OpRange { &self.2 }
    fn count(&self) -> usize { self.2.ops.len() }
}

struct SromRange {
    current: usize,
    banks: HashMap<usize, OpRange>
}

impl Default for SromRange {
    fn default() -> Self {
        Self { current: 1, banks: HashMap::with_capacity(0x200) }
    }
}

impl<E: Emulator> MemRange<E> for RomRange {
    fn reload(&mut self) { self.0.ops.clear(); }
    fn range(&self) -> Range<u16> { 0..0x4000 }
    fn update(&mut self, emu: &E) {
        if self.0.ops.is_empty() {
            self.0 = OpRange::default().parse(emu.get_range(0, 0x4000), vec![(0x104, 0x46)]);
        }
    }
    fn ops(&self) -> &OpRange { &self.0 }
    fn count(&self) -> usize { self.0.ops.len() }
}

impl<E: Emulator> MemRange<E> for SromRange {
    fn reload(&mut self) { self.banks.clear() }
    fn range(&self) -> Range<u16> { 0x4000..0x8000 }
    fn update(&mut self, emu: &E) {
        let bank = emu.bus().mbc().rom_bank();
        self.current = bank;
        self.banks.entry(bank).or_insert_with(|| OpRange::default().parse(emu.get_range(0x4000, 0x4000), vec![]));
    }

    fn ops(&self) -> &OpRange { &self.banks[&self.current] }

    fn count(&self) -> usize { self.banks.get(&self.current).map(|x| x.ops.len()).unwrap_or(0) }
}

pub trait MemRange<E: Emulator> {
    fn reload(&mut self);
    fn range(&self) -> Range<u16>;
    fn update(&mut self, emu: &E);
    fn ops(&self) -> &OpRange;
    fn boxed(self) -> Box<dyn MemRange<E>> where Self: 'static + Sized { Box::new(self) }
    fn count(&self) -> usize;
}

pub struct Disassembly<E: Emulator> {
    ranges: Vec<Box<dyn MemRange<E>>>,
    cursor: Cursor,
    hover: Option<usize>,
}

impl<E: Emulator> Disassembly<E> {
    pub fn new() -> Self {
        Self {
            cursor: Cursor::Follow,
            ranges: vec![
                RomRange::default().boxed(),
                SromRange::default().boxed(),
                DynRange::new(RAM, RAM_END).boxed(),
                DynRange::new(SRAM, SRAM_END).boxed(),
                DynRange::new(HRAM, HRAM_END).boxed()
            ],
            hover: None
        }
    }

    pub fn follow(&mut self) {
        self.cursor = Cursor::Follow;
    }

    pub fn range(&mut self, pc: u16) -> Option<&mut Box<dyn MemRange<E>>> {
        self.ranges.iter_mut().find(|x| x.range().contains(&pc))
    }

    pub fn lines(&self) -> usize {
        self.ranges.iter().fold(0, |acc, range| acc + range.count())
    }

    pub fn row(&self, addr: u16) -> usize {
        self.ranges.iter().fold(0, |row, range| {
            let r = range.range();
            if r.contains(&addr) {
                range.ops().ops.iter().enumerate().find(|(_i, x)| x.offset + r.start >= addr)
                    .map(|x| row + x.0)
                    .unwrap_or(row)
            } else if r.start < addr {
                row + range.count()
            } else {
                row
            }
        })
    }

    pub(crate) fn next(&mut self, emu: &E) -> Option<(u16, Op)> {
        let pc = emu.cpu_register(Reg::PC).u16();
        if let Some((range, ops)) = self.range(pc)
            .map(|x| (x.range(), x.ops())) {
            let mut st = range.start;
            for op in &ops.ops {
                if st == pc { return Some((st, op.clone())); }
                st += op.size as u16;
            }
        }
        None
    }

    pub fn fixed(&mut self, emu: &E) {
        if Cursor::Follow == self.cursor {
            self.cursor = Cursor::Fixed(self.row(emu.cpu_register(Reg::PC).u16()));
        }
    }

    pub fn reload(&mut self) {
        self.ranges.iter_mut().for_each(|x| x.reload());
    }

    pub fn render(&mut self, emu: &E, ui: &mut Ui, breakpoints: &Breakpoints) {
        let pc = emu.cpu_register(Reg::PC).u16();
        let cursor = match self.cursor {
            Cursor::Follow => self.row(pc),
            Cursor::Fixed(row) => row,
        };
        ui.set_height(300.);
        let row = self.row(pc);
        self.ranges.iter_mut().for_each(|x| { x.update(emu); });
        let mut table = TableBuilder::new(ui)
            .columns(Column::remainder(), 3)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(Align::Center))
            .auto_shrink([false, false]);
        if let Cursor::Follow = self.cursor {
            table = table.scroll_to_row(row, Some(Align::Center));
        }
        table.header(20., |mut header| {
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
            .body(|body| {
            let lines = self.lines();
            let mut hover = None;
            body.rows(30., lines, |index, mut row| {
                let mut st = index;
                if let Some(current) = self.ranges.iter_mut()
                    .fold(None, |res, current| {
                        match res {
                            None if st < current.count() => Some(current),
                            None => {
                                st -= current.count();
                                None
                            },
                            v => v
                        }
                    }) {
                    let addr = current.range().start;
                    let op = &current.ops().ops[st];
                    if (row.col(|ui| {
                        let rect = ui.available_rect_before_wrap().expand2(Vec2::new(8., 0.));
                        if Some(index) == self.hover {  ui.painter().rect_filled(rect, Rounding { nw: 4., sw: 4., ne: 0., se: 0. }, Color32::from_white_alpha(20)) }
                        else if index == cursor { ui.painter().rect_filled(rect, Rounding { nw: 4., sw: 4., ne: 0., se: 0. }, Color32::DARK_GREEN); }
                        ui.label(egui::RichText::new(format!("{:#06X}", addr + op.offset)));
                    }).1 | row.col(|ui| {
                        let mut rect = ui.available_rect_before_wrap();
                        rect.max.x += 8.;
                        if Some(index) == self.hover {  ui.painter().rect_filled(rect, 0., Color32::from_white_alpha(20)) }
                        else if index == cursor { ui.painter().rect_filled(rect, 0., Color32::DARK_GREEN); }
                        ui.label(egui::RichText::new(&op.instruction));
                    }).1 | row.col(|ui| {
                        let mut rect = ui.available_rect_before_wrap();
                        rect.max.x += 8.;
                        if Some(index) == self.hover {  ui.painter().rect_filled(rect, 0., Color32::from_white_alpha(20)) }
                        else if index == cursor { ui.painter().rect_filled(rect, 0., Color32::DARK_GREEN); }
                        let mut code = String::new();
                        for o in &op.data { code.push_str(format!(" {o:02X}").as_str()); }
                        ui.label(egui::RichText::new(code));
                    }).1).context_menu(|ui| {
                        if ui.button("Run to cursor").clicked() {
                            breakpoints.schedule(Breakpoint::address(addr + op.offset).once());
                            emu.play();
                        }
                    }).hovered() {
                        hover = Some(index);
                    };
                }
            });
            self.hover = hover;
        });
    }
}
