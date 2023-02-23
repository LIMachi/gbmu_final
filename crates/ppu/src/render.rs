use shared::egui::*;
use shared::egui::epaint::ImageDelta;
use shared::Events;
use super::*;

mod tilemap;
mod oam;
mod tiledata;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Textures {
    None,
    Placeholder,
    Tile(usize),
    Miniature
}

pub struct PixelBuffer<const W: usize, const H: usize> where [(); W * H]: Sized {
    pixels: [u8; W * H]
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
                let color =  match self.pixels[w + h * W] { // w + 8 * h
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

impl shared::Ui for Controller {
    fn init(&mut self, ctx: &mut Context) {
        let base = ColorImage::new([64, 64], Color32::from_black_alpha(50));
        for n in 0..768 {
            let s = Textures::Tile(n);
            self.storage.insert(s, ctx.load_texture(format!("{:?}", s), base.clone(), TextureOptions::NEAREST));
        }
        self.storage.insert(Textures::None, ctx.load_texture("None", ColorImage::new([8, 8], Color32::from_black_alpha(0)), TextureOptions::NEAREST));
        self.storage.insert(Textures::Placeholder, ctx.load_texture("Placeholder", base, TextureOptions::NEAREST));
        self.storage.insert(Textures::Miniature,ctx.load_texture("Miniature", ColorImage::new([160, 144], Color32::from_black_alpha(0)), TextureOptions::NEAREST));
        self.init = true;
    }

    fn draw(&mut self, ctx: &mut Context) {
        if !self.init {
            self.init(ctx);
            self.init = true;
        }
        let tiles: Vec<usize> = { self.ppu.as_ref().borrow_mut().tile_cache.drain().collect() };
        let mut ppu = self.ppu.as_ref().borrow();
        for n in tiles {
            let buf = PixelBuffer::<8, 8>::new(ppu.vram.tile_data(n, 0)).image::<64, 64>();
            ctx.tex_manager()
                .write()
                .set(self.storage.get(&Textures::Tile(n)).unwrap().id(), ImageDelta::full(buf, TextureOptions::NEAREST));
        }
        CentralPanel::default()
            .show(ctx, |ui| {
               ui.add(oam::Oam(&self.storage, &ppu));
            });
    }

    fn handle(&mut self, event: &Events) {
    }
}
