use std::net::Ipv4Addr;
use shared::egui;
use shared::egui::{Align, Response, TextEdit, Ui, Widget};
use crate::emulator::Emulator;
use crate::settings::Mode;

pub struct Device<'a> {
    emu: &'a mut Emulator,
}

impl<'a> Device<'a> {
    pub fn new(emu: &'a mut Emulator) -> Self {
        Self {
            emu,
        }
    }
}

impl<'a> Widget for Device<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            let model = &mut self.emu.cgb;
            ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                ui.label("MODEL");
            });
            ui.radio_value(model, Mode::Dmg, format!("{:?}", Mode::Dmg));
            ui.radio_value(model, Mode::Cgb, format!("{:?}", Mode::Cgb));
            ui.checkbox(&mut self.emu.bios, "enable boot rom");
            ui.separator();

            ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                ui.label(if self.emu.link_do(|x| x.connected()) { "SERIAL - (Connected)" } else { "SERIAL" });
            });
            ui.label(format!("server listening on port {}", self.emu.link_port));

            ui.horizontal(|ui| {
                let host = TextEdit::singleline(&mut self.emu.settings.host).desired_width(120.);
                ui.label("Host: ");
                ui.add(host);
            });
            ui.horizontal(|ui| {
                let port = TextEdit::singleline(&mut self.emu.settings.port).desired_width(48.);
                ui.label(" Port: ");
                ui.add(port);
            });
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
            };
            ui.separator();
        }).response
    }
}