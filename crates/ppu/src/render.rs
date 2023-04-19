use shared::egui::*;
use shared::egui::epaint::ImageDelta;
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
    init: bool,
    emu: PhantomData<E>,
}

impl<E> Default for VramViewer<E> {
    fn default() -> Self {
        VramViewer {
            tab: Tabs::Oam,
            storage: Default::default(),
            init: false,
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
}

const DARK_BLACK: Color32 = Color32::from_rgb(0x23, 0x27, 0x2A);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) enum Textures {
    None,
    Blank,
    Placeholder,
    Tile(usize),
    Miniature,
}

struct PixelBuffer<const W: usize, const H: usize> where [(); W * H]: Sized {
    pixels: [u8; W * H],
}

impl<const W: usize, const H: usize> PixelBuffer<W, H> where [(); W * H]: Sized {
    pub fn new(pixels: [u8; W * H]) -> Self {
        Self { pixels }
    }

    pub fn image<const IW: usize, const IH: usize>(&self) -> ColorImage where
        [(); IW * IH * 4]: Sized
    {
        let sw = IW / W; // 8
        let sh = IH / H; // 8
        assert_eq!(IW % W, 0);
        assert_eq!(IH % H, 0);
        let mut buf = [0; IW * IH * 4]; // 64 * 4
        for w in 0..W { // 0..8
            for h in 0..H { // 0..8
                let color = match self.pixels[w + h * W] { // w + 8 * h
                    0 => [255, 255, 255, 255],
                    1 => [192, 192, 192, 255],
                    2 => [128, 128, 128, 255],
                    3 => [64, 64, 64, 255],
                    _ => [0, 0, 0, 255]
                };
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
}

impl<E: Emulator + PpuAccess> shared::Ui for VramViewer<E> {
    type Ext = E;

    fn init(&mut self, ctx: &mut Context, emu: &mut E) {
        let base = ColorImage::new([64, 64], Color32::from_black_alpha(50));
        let count = if emu.bus().is_cgb() { 768 } else { 384 };
        for n in 0..count {
            let s = Textures::Tile(n);
            self.insert(s, ctx.load_texture(format!("{:?}", s), base.clone(), TextureOptions::NEAREST));
        }
        self.insert(Textures::Blank, ctx.load_texture("Blank", ColorImage::new([8, 8], Color32::WHITE), TextureOptions::NEAREST));
        self.insert(Textures::None, ctx.load_texture("None", ColorImage::new([8, 8], Color32::from_black_alpha(0)), TextureOptions::NEAREST));
        self.insert(Textures::Placeholder, ctx.load_texture("Placeholder", base, TextureOptions::NEAREST));
        self.insert(Textures::Miniature, ctx.load_texture("Miniature", ColorImage::new([160, 144], Color32::from_black_alpha(0)), TextureOptions::NEAREST));
        self.init = true;
    }

    fn draw(&mut self, ctx: &mut Context, emu: &mut E) {
        if !self.init {
            self.init(ctx, emu);
            self.init = true;
        }
        let tiles: Vec<usize> = emu.vram_mut().tile_cache.drain().collect();
        let vram = emu.vram();
        for n in tiles {
            let buf = PixelBuffer::<8, 8>::new(vram.tile_data(n % 384, n / 384)).image::<64, 64>();
            let id = self.tex(Textures::Tile(n)).expect(format!("can't access tile {n}").as_str()).id();
            ctx.tex_manager().write().set(id, ImageDelta::full(buf, TextureOptions::NEAREST));
        }
        CentralPanel::default()
            .show(ctx, |ui|
                tabs::Tabs::new(&mut self.tab, ui, &[Tabs::Oam, Tabs::Tiledata, Tabs::Tilemap])
                    .with_tab(Tabs::Oam, oam::Oam(self, emu))
                    .with_tab(Tabs::Tiledata, bgmap::BgMap(self, emu, ctx))
                    .with_tab(Tabs::Tilemap, tilemap::Tilemap(self))
                    .response());
    }

    fn handle(&mut self, _event: &shared::Event, _emu: &mut E) {}
}
