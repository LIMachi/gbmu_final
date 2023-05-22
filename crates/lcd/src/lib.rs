use pixels::{Pixels, PixelsBuilder, SurfaceTexture};

use shared::io::{IO, IORegs};
use shared::serde::{Deserialize, Serialize};
use shared::winit as winit;

#[derive(Serialize, Deserialize)]
pub struct Lcd {
    enabled: bool,
    frame: Vec<u8>,
    #[serde(default, skip)]
    pub pixels: Option<Pixels>, //TODO serde: rebind using previous pixels
}

impl Default for Lcd {
    fn default() -> Self {
        Self {
            enabled: false,
            frame: vec![0; (4 * Lcd::WIDTH * Lcd::HEIGHT) as usize],
            pixels: None,
        }
    }
}

impl LCD for Lcd {
    fn set(&mut self, x: usize, y: usize, color: [u8; 3]) {
        if !self.enabled {
            return;
        }
        let f = (Lcd::WIDTH * 4) as usize;
        self.frame[x * 4 + 0 + y * f] = color[0];
        self.frame[x * 4 + 1 + y * f] = color[1];
        self.frame[x * 4 + 2 + y * f] = color[2];
        self.frame[x * 4 + 3 + y * f] = 0xFF;
    }

    fn enable(&mut self) {
        self.enabled = true;
    }

    fn disable(&mut self, io: &IORegs) {
        self.enabled = false;
        let white = if io.io(IO::CGB).bit(0) != 0 { [0xFF; 3] } else { io.palette().color(0) };
        if let Some(pixels) = self.pixels.as_mut() {
            let pixels = pixels.get_frame_mut();
            for i in 0..(4 * Lcd::WIDTH * Lcd::HEIGHT) as usize {
                pixels[i] = if i % 4 == 3 { 0xFF } else { white[i % 4] };
            }
        }
    }

    fn vblank(&mut self) {
        if let Some(pixels) = self.pixels.as_mut() {
            pixels.get_frame_mut()
                .iter_mut()
                .zip(self.frame.iter_mut())
                .for_each(|(d, s)| *d = *s);
        }
    }
}

impl Lcd {
    pub const WIDTH: u32 = 160;
    pub const HEIGHT: u32 = 144;

    pub fn reload(self, load: Self) -> Self {
        let mut t = load;
        t.pixels = self.pixels;
        t
    }

    pub fn init(&mut self, window: &winit::window::Window) {
        let sz = window.inner_size();
        let surf = SurfaceTexture::new(sz.width, sz.height, window);
        let pixels = PixelsBuilder::new(Lcd::WIDTH, Lcd::HEIGHT, surf)
            .enable_vsync(false).build().unwrap();
        self.pixels.replace(pixels);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(pixels) = self.pixels.as_mut() {
            pixels.resize_surface(width, height).ok();
        }
    }

    pub fn render(&mut self) {
        if let Some(pixels) = self.pixels.as_mut() { pixels.render().ok(); }
    }
}

pub trait LCD {
    fn set(&mut self, x: usize, y: usize, pixel: [u8; 3]);
    fn enable(&mut self);
    fn disable(&mut self, io: &IORegs);
    fn vblank(&mut self);
}
