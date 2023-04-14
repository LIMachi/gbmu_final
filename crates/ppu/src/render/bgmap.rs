use shared::egui;
use shared::egui::{Color32, Image, Response, Stroke, Ui, Vec2, Widget};
use shared::emulator::Emulator;
use shared::io::{IO, LCDC};
use crate::render::PpuAccess;
use crate::VramViewer;

pub struct BgMap<'a, E: Emulator + PpuAccess>(pub(crate) &'a mut VramViewer<E>, pub(crate) &'a mut E, pub(crate) &'a egui::Context);

#[derive(Default, Copy, Clone)]
pub(crate) struct TileData {
    x: usize,
    y: usize,
    tile: usize,
    attribute: u8,
    map_addr: u16,
    tile_addr: u16,
}

impl<E: Emulator + PpuAccess> Widget for BgMap<'_, E> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.spacing_mut().item_spacing.x = 1.;
        ui.spacing_mut().item_spacing.y = 1.;
        let lcdc = self.1.bus().io(IO::LCDC).value();
        let ppu = self.1.ppu();
        egui::Area::new("scrolled_area")
            .fixed_pos([ppu.sc.x as f32 + ui.available_rect_before_wrap().min.x, ppu.sc.y as f32 + ui.available_rect_before_wrap().min.y])
            .movable(false)
            .interactable(false)
            .show(self.2, |ui| {
                egui::Frame::none()
                    .stroke(Stroke::new(2., Color32::BLACK))
                    .fill(Color32::TRANSPARENT)
                    .show(ui, |ui| {
                       ui.allocate_space(Vec2::new(500., 450.));
                    });
            });
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                for j in 0..32 {
                    ui.spacing_mut().item_spacing.y = 1.;
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.;
                        for i in 0..32 {
                            let addr = i + j * 32 + if lcdc.bg_area() { 0x1C00 } else { 0x1800 };
                            let tile = self.1.vram().read_bank(addr, 0);
                            let attribute = self.1.vram().read_bank(addr, 1);
                            let mut tile = if lcdc.relative_addr() {
                                (256 + (tile as i8) as isize) as usize
                            } else { tile as usize };
                            if attribute & 0x8 != 0 { tile += 384; }
                            let tex = self.0.get(tile).id();
                            let ret = ui.add(Image::new(tex, [24., 24.]));
                            if ret.hovered() {
                                self.0.bg_data = Some(TileData{
                                    x: i as usize,
                                    y: j as usize,
                                    tile,
                                    attribute,
                                    map_addr: addr,
                                    tile_addr: 0
                                })
                            }
                        }
                    });
                }
            });
            ui.add_space(20.);
            ui.vertical(|ui| {
                egui::Frame::none()
                    .fill(Color32::TRANSPARENT)
                    .show(ui, |ui| {
                        let data = self.0.bg_data.unwrap_or_default();
                        let tex = self.0.get(data.tile as usize).id();
                        ui.add(Image::new(tex, [256., 256.]));
                        ui.label(format!("x : {:02}", data.x));
                        ui.label(format!("Y : {:02}", data.y));
                        ui.label(format!("Tile Number: {:02X}{}", data.tile & 0xFF, if data.tile > 0xFF { "(H)" } else { "" }));
                        ui.label(format!("Attribute: {:02X}", data.attribute));
                        ui.label(format!("Tile Address: {:#04X}", 0x8000 + data.tile_addr * 16));
                        ui.label(format!("Map Address: {:#04X}", data.map_addr + 0x8000));
                    });
            });
        }).response
    }
}
