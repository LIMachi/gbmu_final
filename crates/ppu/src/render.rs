use std::collections::HashSet;

use shared::egui::*;
use shared::egui::epaint::ImageDelta;
use shared::Events;
use shared::events::WindowEvent;
use shared::utils::serde_arrays;
use shared::widgets::tabs;

use crate::ppu::Ppu;

use super::*;

mod tilemap;
mod oam;
mod bgmap;

pub struct VramViewer<E> {
    tab: Tabs,
    storage: HashMap<Textures, TextureHandle>,
    bg_data: Option<bgmap::TileData>,
    tile_data: Vec<PixelBuffer>,
    draw_cache: HashSet<usize>,
    emu: PhantomData<E>,
}

impl<E> Default for VramViewer<E> {
    fn default() -> Self {
        VramViewer {
            draw_cache: HashSet::with_capacity(768),
            tile_data: vec![PixelBuffer::new(); 768],
            tab: Tabs::Oam,
            storage: Default::default(),
            emu: Default::default(),
            bg_data: None,
        }
    }
}

pub trait VramAccess {
    fn vram(&self) -> &Vram;
    fn vram_mut(&mut self) -> &mut Vram;

    fn oam(&self) -> &Oam;
    fn oam_mut(&mut self) -> &mut Oam;
}

pub trait PpuAccess: VramAccess {
    fn ppu(&self) -> &Ppu;
    fn ppu_mut(&mut self) -> &mut Ppu;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) enum Textures {
    None,
    Blank,
    Placeholder,
    Tile(usize),
    Draw(usize),
    Miniature,
}

const BUF_W: usize = 8;
const BUF_H: usize = 8;
const BUF_SZ: usize = 64;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct PixelBuffer {
    #[serde(with = "serde_arrays")]
    pixels: [u8; BUF_SZ],
}

#[derive(Serialize, Deserialize)]
pub struct ColorBuffer {
    #[serde(with = "serde_arrays")]
    pixels: [Option<[u8; 4]>; BUF_SZ],
}

impl ColorBuffer {
    pub fn new() -> Self { Self { pixels: [None; BUF_SZ] } }

    pub fn color(&mut self, x: usize, y: usize, color: [u8; 3]) {
        let [r, g, b] = color;
        self.pixels[x + y * BUF_W] = Some([r, g, b, 255]);
    }

    pub(crate) fn image<const IW: usize, const IH: usize, const IS: usize>(&self, source: &PixelBuffer) -> ColorImage where [(); IS]: Sized {
        let sw = IW / BUF_W; // 8
        let sh = IH / BUF_H; // 8
        assert_eq!(IW % BUF_W, 0);
        assert_eq!(IH % BUF_H, 0);
        let mut buf = [0; IS]; // 64 * 4
        for w in 0..BUF_W { // 0..8
            for h in 0..BUF_H { // 0..8
                let color = self.pixels[w + h * BUF_W].unwrap_or_else(|| source.at(w, h));
                for i in 0..sw { // 0..8
                    for j in 0..sh { // 0..8
                        let x = w * 4 * sw + i * 4; // w * 8 + i * 4
                        let y = h * sh + j;
                        buf[x + y * 4 * IW] = color[0];
                        buf[x + y * 4 * IW + 1] = color[1];
                        buf[x + y * 4 * IW + 2] = color[2];
                        buf[x + y * 4 * IW + 3] = color[3];
                    }
                }
            }
        }
        ColorImage::from_rgba_unmultiplied([IW, IH], &buf)
    }
}

impl PixelBuffer {
    pub fn new() -> Self {
        Self { pixels: [0; BUF_SZ] }
    }

    pub fn blit(&mut self, data: [u8; BUF_SZ]) {
        self.pixels = data;
    }

    pub fn at(&self, x: usize, y: usize) -> [u8; 4] {
        match self.pixels[x + y * BUF_W] { // w + 8 * h
            0 => [255, 255, 255, 255],
            1 => [192, 192, 192, 255],
            2 => [128, 128, 128, 255],
            3 => [64, 64, 64, 255],
            _ => [0, 0, 0, 255]
        }
    }

    pub fn image<const IW: usize, const IH: usize, const IS: usize>(&self) -> ColorImage where
        [(); IS]: Sized
    {
        let sw = IW / BUF_W; // 8
        let sh = IH / BUF_H; // 8
        assert_eq!(IW % BUF_W, 0);
        assert_eq!(IH % BUF_H, 0);
        let mut buf = [0; IS]; // 64 * 4
        for w in 0..BUF_W { // 0..8
            for h in 0..BUF_H { // 0..8
                let color = self.at(w, h);
                for i in 0..sw { // 0..8
                    for j in 0..sh { // 0..8
                        let x = w * 4 * sw + i * 4; // w * 8 + i * 4
                        let y = h * sh + j;
                        buf[x + y * 4 * IW] = color[0];
                        buf[x + y * 4 * IW + 1] = color[1];
                        buf[x + y * 4 * IW + 2] = color[2];
                        buf[x + y * 4 * IW + 3] = color[3];
                    }
                }
            }
        }
        ColorImage::from_rgba_unmultiplied([IW, IH], &buf)
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
enum Tabs {
    Oam,
    Tiledata,
    Tilemap,
}

impl tabs::Tab for Tabs {
    fn name(&self) -> String {
        format!("{:?}", self)
    }
}

impl<E: Emulator> VramViewer<E> {
    pub(crate) fn get(&self, tile: usize) -> TextureHandle {
        self.storage.get(&Textures::Tile(tile))
            .or(self.storage.get(&Textures::Blank))
            .cloned().unwrap()
    }

    pub(crate) fn insert(&mut self, tile: Textures, tex: TextureHandle) {
        self.storage.insert(tile, tex);
    }

    pub(crate) fn tex(&self, tex: Textures) -> Option<TextureHandle> {
        self.storage.get(&tex).cloned()
    }

    pub(crate) fn draw_tex(&self, tile: usize) -> Option<TextureId> {
        self.storage.get(&if self.draw_cache.contains(&tile) {
            Textures::Draw(tile)
        } else {
            Textures::Tile(tile)
        }).map(|x| x.id())
    }
}

impl<E: Emulator + PpuAccess> shared::Ui for VramViewer<E> {
    type Ext = E;

    fn init(&mut self, ctx: &mut Context, emu: &mut E) {
        let base = ColorImage::new([64, 64], Color32::from_black_alpha(50));
        let count = if emu.bus().is_cgb() { 768 } else { 384 };
        for n in 0..count {
            let s = Textures::Tile(n);
            let d = Textures::Draw(n);
            self.insert(s, ctx.load_texture(format!("{:?}", s), base.clone(), TextureOptions::NEAREST));
            self.insert(d, ctx.load_texture(format!("{:?}", d), base.clone(), TextureOptions::NEAREST));
        }
        self.insert(Textures::Blank, ctx.load_texture("Blank", ColorImage::new([8, 8], Color32::WHITE), TextureOptions::NEAREST));
        self.insert(Textures::None, ctx.load_texture("None", ColorImage::new([8, 8], Color32::from_black_alpha(0)), TextureOptions::NEAREST));
        self.insert(Textures::Placeholder, ctx.load_texture("Placeholder", base, TextureOptions::NEAREST));
        self.insert(Textures::Miniature, ctx.load_texture("Miniature", ColorImage::new([160, 144], Color32::from_black_alpha(0)), TextureOptions::NEAREST));
    }

    fn draw(&mut self, ctx: &mut Context, emu: &mut E) {
        let mut tiles = Vec::with_capacity(768);
        let vram = emu.vram_mut();
        tiles.extend(vram.tile_cache.drain());
        for &tile in &tiles {
            self.tile_data[tile].blit(vram.tile_data(tile % 384, tile / 384));
            let id = self.tex(Textures::Tile(tile)).expect(format!("can't access tile {tile}").as_str()).id();
            let image = self.tile_data[tile].image::<64, 64, 16384>();
            ctx.tex_manager().write().set(id, ImageDelta::full(image, TextureOptions::NEAREST));
        }
        let cache = emu.ppu_mut().draw_cache.drain();
        let mut was_empty = true;
        for (tile, color) in cache {
            was_empty = false;
            self.draw_cache.insert(tile);
            let id = self.tex(Textures::Draw(tile)).expect(format!("can't access tile {tile}").as_str()).id();
            let image = color.image::<64, 64, 16384>(&self.tile_data[tile]);
            ctx.tex_manager().write().set(id, ImageDelta::full(image, TextureOptions::NEAREST));
        }
        if was_empty {
            self.draw_cache.clear();
        }
        emu.ppu_mut().sprite_debug = self.tab == Tabs::Tilemap;
        CentralPanel::default()
            .show(ctx, |ui|
                tabs::Tabs::new(&mut self.tab, ui, &[Tabs::Oam, Tabs::Tiledata, Tabs::Tilemap])
                    .with_tab(Tabs::Oam, oam::Oam(self, emu))
                    .with_tab(Tabs::Tiledata, bgmap::BgMap(self, emu, ctx))
                    .with_tab(Tabs::Tilemap, tilemap::Tilemap(self))
                    .response());
        self.draw_cache.clear();
    }

    fn handle(&mut self, event: &shared::winit::event::Event<Events>, ctx: &mut Context, ext: &mut <Self as shared::Ui>::Ext) {
        match event {
            shared::winit::event::Event::WindowEvent {
                event: WindowEvent::CloseRequested, ..
            } => {
                ext.ppu_mut().sprite_debug = false;
            }
            shared::winit::event::Event::UserEvent(Events::Reload) => {
                self.init(ctx, ext);
            }
            _ => {}
        }
    }
}
