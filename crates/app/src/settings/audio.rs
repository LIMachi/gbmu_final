use shared::egui;
use shared::egui::{Response, ScrollArea, Sense, Ui, Vec2, Widget};
use shared::widgets::section::Section;

use crate::emulator::Emulator;
use crate::settings::Settings;

pub struct Audio<'a> {
    settings: &'a Settings,
    emu: &'a mut Emulator,
}

impl<'a> Audio<'a> {
    pub fn new(settings: &'a Settings, emu: &'a mut Emulator) -> Self {
        Self {
            settings,
            emu,
        }
    }
}

impl<'a> Widget for Audio<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.section("Channels", |ui| {
                ui.add(egui::Slider::new(&mut self.emu.audio_settings.volume, 0f32..=1f32).text("Volume")) |
                    ui.checkbox(&mut self.emu.audio_settings.channels[0], "Channel 1 - Sweep") |
                    ui.checkbox(&mut self.emu.audio_settings.channels[1], "Channel 2 - Square") |
                    ui.checkbox(&mut self.emu.audio_settings.channels[2], "Channel 3 - Wave") |
                    ui.checkbox(&mut self.emu.audio_settings.channels[3], "Channel 4 - Noise")
            }) |
                ui.section("Devices", |ui| {
                    let scroll = ScrollArea::vertical();
                    let mut device = &self.emu.audio.device();
                    let devices: Vec<&String> = self.settings.devices.iter().collect();
                    let res = scroll.max_height(120.)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            let mut res = ui.allocate_response(Vec2::ZERO, Sense::hover());
                            for dev in &devices {
                                res |= ui.radio_value(&mut device, dev, *dev);
                            }
                            res
                        });
                    if device != &self.emu.audio.device() {
                        let device = device.clone().clone();
                        self.emu.audio.switch(device, &mut self.emu.console.gb.apu);
                    }
                    res.inner
                })
        }).response
    }
}
