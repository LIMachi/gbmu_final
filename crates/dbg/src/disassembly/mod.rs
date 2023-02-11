use std::collections::HashMap;
use shared::cpu::{CBOpcode, Opcode, Reg};
use shared::egui;
use shared::egui::{Color32, Layout, Response, TextureHandle, TextureId, TextureOptions, Ui, Widget};
use shared::egui::WidgetText::RichText;
use crate::Emulator;

mod dbg_opcodes;

struct OpRange {
    st: u16,
    len: usize,
    ops: Vec<(u16, String)>
}
//
// fn get_dbg_range(&self, start: u16, len: u16) -> Vec<(u16, Vec<u8>, &'static str)> {
//     let mut v = Vec::new();
//     let mut bc = false;
//     let mut i = start;
//     while i < start + len {
//         let op = *self.rom.get(i as usize).unwrap_or(&0x00u8);
//         v.push(if !bc {
//             bc = op == 0xCB;
//             let (len, label) = Opcode::try_from(op).map_or_else(|_| {(1, "Invalid")}, |op|{dbg_opcodes(op)});
//             let ti = i;
//             i += len as u16;
//             (i, label)
//         } else {
//             let op = CBOpcode::try_from(op).unwrap();
//             let label = dbg_cb_opcodes(op);
//             bc = false;
//             i += 1;
//             (i, label)
//         });
//     }
//     v
// }


impl OpRange {
    pub fn new(pc: u16, range: Vec<u8>) -> Self {
        let mut off = 0;
        let mut ops = Vec::with_capacity(range.len());
        let mut prefix = false;
        while off < range.len() {
            let opcode = if prefix { range[off + 1] } else { range[off] };
            let mut op = (opcode, prefix).try_into().unwrap_or(Opcode::Nop);
            if op == Opcode::PrefixCB { prefix = true; continue; }
            prefix = false;
            let (sz, info) = dbg_opcodes::dbg_opcodes(op);
            let code = &range[off..(off + sz)];
            let mut str = format!("{:#06X}: ", pc + off as u16);
            // for o in code { str.push_str(format!("{o:02X}  ").as_str()); }
            // for s in code.len()..6 { str.push_str("    "); }
            str.push_str(format!("{info}").as_str());
            for s in info.len()..14 { str.push_str(" "); }
            for o in code { str.push_str(format!(" {o:02X}").as_str()); }
            for _ in code.len()..3 { str.push_str("   "); }
            ops.push((pc + off as u16, str));
            off += sz;
        }
        Self { st: pc, len: off, ops }
    }

    pub fn contains(&self, pc: u16) -> bool {
        pc >= self.st && pc <= (self.st + self.len as u16)
    }
}


pub struct Disassembly {
    pc: u16,
    range: Option<OpRange>
}

impl Disassembly {
    pub fn new() -> Self { Self { pc: 0x0, range: None } }

    pub fn render<E: Emulator>(&mut self, emu: &E, ui: &mut Ui) {
        let pc = emu.cpu_register(Reg::PC).u16();
        if self.range.as_ref().map(|x| !x.contains(pc)).unwrap_or(true) {
            self.range = Some(OpRange::new(pc, emu.get_range(pc, 256)));
        }
        ui.columns(3, |ui: &mut [Ui]| {
            let ops = &self.range.as_ref().unwrap().ops;
            for mut l in 0..90.min(ops.len() - 1) {
                let ui = &mut ui[l / 30];
                let text = if ops[l].0 <= pc && ops[l + 1].0 > pc {
                    egui::RichText::new(&ops[l].1)
                        .background_color(Color32::DARK_GREEN)
                } else { egui::RichText::new(&ops[l].1) };
                ui.label(text);
            }
        });
    }
}
