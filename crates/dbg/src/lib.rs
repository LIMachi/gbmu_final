use std::cell::{Ref, RefCell};
use std::rc::Rc;
use shared::{egui::Context, Ui, cpu::*};

mod render;
mod dbg_opcodes;

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

pub trait Emulator: ReadAccess { }

impl<E: ReadAccess> Emulator for E { }

pub trait ReadAccess {
    fn cpu_register(&self, reg: Reg) -> Value;
    fn get_range(&self, st: u16, len: u16) -> Vec<u8>;
}

pub struct Disassembly {
    // store disassembly info, range, etc..
    // TODO maybe implement rolling disassembly ?
}

/// Ninja: Debugger internal code name.
struct Ninja<E: Emulator> {
    emu: E,
    disassembly: Disassembly
}

impl<E: Emulator> Ninja<E> {
    pub fn new(emu: E) -> Self {
        Self {
            emu,
            disassembly: Disassembly { }
        }
    }
}

#[derive(Clone)]
pub struct Debugger<E: Emulator> {
    inner: Rc<RefCell<Ninja<E>>>
}

impl<E:Emulator> Ui for Debugger<E> {
    fn draw(&mut self, ctx: &Context) {
        self.inner.borrow_mut().draw(ctx)
    }
}

impl<E: Emulator> Debugger<E> {
    pub fn new(emu: E) -> Self {
        Self {
            inner: Rc::new(RefCell::new(Ninja::new(emu)))
        }
    }

}

