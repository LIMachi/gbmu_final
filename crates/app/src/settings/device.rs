use std::net::Ipv4Addr;
use std::time::Instant;

use shared::egui::{Response, TextEdit, Ui, Widget};
use shared::widgets::section::Section;

use crate::emulator::Emulator;
use crate::settings::Mode;

pub struct Device<'a> {
    emu: &'a mut Emulator,
    autosave: &'a mut String,
}

impl<'a> Device<'a> {
    pub fn new(emu: &'a mut Emulator, input: &'a mut String) -> Self {
        Self {
            emu,
            autosave: input,
        }
    }
}

impl<'a> Widget for Device<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            let model = &mut self.emu.cgb;
            ui.section("MODEL", |ui| {
                ui.radio_value(model, Mode::Dmg, format!("{:?}", Mode::Dmg)) |
                    ui.radio_value(model, Mode::Cgb, format!("{:?}", Mode::Cgb)) |
                    ui.checkbox(&mut self.emu.bios, "enable boot rom")
            });
            let connected = self.emu.link_do(|x| x.connected());
            ui.section(if connected { "SERIAL - (Connected)" } else { "SERIAL" }, |ui| {
                ui.label(format!("server listening on port {}", self.emu.link_port));
                let res = ui.horizontal(|ui| {
                    let host = TextEdit::singleline(&mut self.emu.settings.host).desired_width(120.);
                    ui.label("Host: ");
                    ui.add(host);
                }).response |
                    ui.horizontal(|ui| {
                        let port = TextEdit::singleline(&mut self.emu.settings.port).desired_width(48.);
                        ui.label(" Port: ");
                        ui.add(port);
                    }).response;
                ui.horizontal(|ui| {
                    if ui.button("Connect").clicked() {
                        match (self.emu.settings.host.parse(), self.emu.settings.port.parse()) {
                            (Ok(addr), Ok(port)) => {
                                let addr: Ipv4Addr = addr;
                                let port: u16 = port;
                                self.emu.link_do(|link| link.connect(addr, port));
                            }
                            (a, p) => {
                                log::warn!("failed to parse: {a:?}, {p:?}");
                            }
                        }
                    }
                    if connected && ui.button("Disconnect").clicked() {
                        self.emu.link_do(|x| x.disconnect());
                    }
                });
                res
            });
            ui.section("Game", |ui| {
                let res = ui.label("Autosave: ");
                let auto = ui.checkbox(&mut self.emu.settings.autosave, "");
                if auto.clicked() && self.emu.settings.autosave {
                    self.emu.timer = Instant::now();
                }
                let r = ui.add(TextEdit::singleline(self.autosave).interactive(self.emu.settings.autosave));
                if r.changed() {
                    self.emu.settings.timer = self.autosave.parse::<u64>()
                        .map(|x| 60 * x)
                        .unwrap_or(self.emu.settings.timer);
                }
                res | auto | r
            });
        }).response
    }
}
