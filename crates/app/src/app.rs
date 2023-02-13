use std::collections::HashMap;
use shared::egui::{self, Align, Color32, Context, Grid, Label, Layout, Margin, Rect, Rounding, Sense, TextureHandle, TextureId, Ui};
use shared::rom::Rom;
use shared::utils::image::ImageLoader;
use crate::{Events, Handle};
use crate::render::{Proxy, EventLoop};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
enum Texture {
    Settings,
    Debug,
    Add
}

// TODO serde Deserialize/Serialize + serde_toml
// paths are OS specific paths to rom directories/files mixup
// each of these paths should be handed to Rom::find_roms for recursive dir traversal and rom retrieval
// if no roms.conf file is found (use default directories later, for now project root path)
#[derive(Default)]
pub struct RomConfig {
    paths: Vec<String>
}

#[derive(Clone)]
pub struct Menu {
    proxy: Proxy,
    textures: HashMap<Texture, TextureHandle>,
    roms: Vec<Rom>,
    conf: RomConfig
}

impl Menu {

    pub fn new(proxy: Proxy) -> Self {
        Self {
            proxy,
            textures: Default::default(),
            roms: vec![],
            conf: RomConfig::default()
        }
    }

    fn tex(&self, tex: Texture) -> TextureId {
        self.textures.get(&tex).unwrap().id()
    }
}

const MENU_HEIGHT: f32 = 32.;
const ROM_GRID: f32 = 128.;

impl shared::Ui for Menu {
    fn init(&mut self, ctx: &mut Context) {
        self.textures.insert(Texture::Add, ctx.load_svg::<40, 40>("add", "assets/icons/add.svg"));
        self.textures.insert(Texture::Debug, ctx.load_svg::<40, 40>("debug", "assets/icons/debug.svg"));
        self.textures.insert(Texture::Settings, ctx.load_svg::<40, 40>("settings", "assets/icons/settings.svg"));
    }

    fn draw(&mut self, ctx: &Context) {
        let style = ctx.style();
        let mut frame = egui::Frame::side_top_panel(&style)
            .inner_margin(Margin::symmetric(8., 8.))
            .rounding(Rounding::none());
        egui::containers::TopBottomPanel::top("menu")
            .frame(frame)
            .show(ctx, |ui| {
                ui.columns(2, |uis| {
                    let [l, r] = match uis { [l, r] => [l, r], _ => unreachable!() };
                    let debug = egui::ImageButton::new(self.tex(Texture::Debug), (28., 28.)).frame(false);
                    let setting = egui::ImageButton::new(self.tex(Texture::Settings), (24., 24.)).frame(false);
                    let add = egui::ImageButton::new(self.tex(Texture::Add), (32., 32.)).frame(false);
                    r.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.add(debug).clicked() { self.proxy.send_event(Events::Open(Handle::Debug)).ok(); };
                        ui.add(setting);
                    });
                    l.with_layout(Layout::default(), |ui| {
                        ui.add(add);
                    });
                })
            });
        egui::containers::CentralPanel::default()
            .show(ctx, |ui| {
                let w = ui.available_width();
                egui::containers::ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        let w = ui.available_width();
                        egui::Grid::new("roms").show(ui, |ui| {
                            let frame = egui::Frame::group(ui.style()).fill(Color32::DARK_GRAY);
                            let mut n = 1;
                            for i in 0..25 {
                                if n as f32 * (ROM_GRID + ui.spacing().item_spacing.x * 2.) + ui.spacing().scroll_bar_width + ui.spacing().scroll_bar_outer_margin > w { ui.end_row(); n = 1; }
                                let response = frame.show(ui, |ui: &mut egui::Ui| {
                                    ui.set_width(ROM_GRID);
                                    ui.set_height(ROM_GRID);
                                    ui.label("POKEMON");
                                }).response.interact(Sense::click());
                                if response.double_clicked() { println!("play pokemon ! {i}"); }
                                n += 1;
                            }
                        });
                    });
            });
    }
}

//
// when loading files and directories:
//  -> extensions recommended: gb, gbc
//  -> when loading directories:
//    -> recursive search
//    -> if we find a file with extension .gb / .gbc, look for {filename}.{png/jpg/jpeg}.
//    -> if we find a single file in a directory, look for cover.{png/jpg/jpeg}
//  -> use image as cover, else blank.
//
//
