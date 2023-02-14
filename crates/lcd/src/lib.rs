use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use pixels::wgpu::PresentMode;
use shared::winit as winit;

#[derive(Default)]
pub struct Lcd {
    pixels: Option<Pixels>
}

impl Lcd {
    pub const WIDTH: u32 = 160;
    pub const HEIGHT: u32 = 144;

    pub fn init(&mut self, window: &winit::window::Window) {
        let sz = window.inner_size();
        let surf = SurfaceTexture::new(sz.width, sz.height, window);
        self.pixels = PixelsBuilder::new(Lcd::WIDTH, Lcd::HEIGHT, surf)
            .build()
            .ok();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(ref mut pixels) = self.pixels {
            pixels.resize_surface(width, height).ok();
        }
    }

    pub fn render(&mut self) {
        if let Some(ref mut pixels) = self.pixels {
            for y in 0..Lcd::HEIGHT as usize {
                pixels.get_frame_mut()[y * 4 * Lcd::WIDTH as usize + 0] = 0xFF;
                pixels.get_frame_mut()[y * 4 * Lcd::WIDTH as usize + 1] = 0xFF;
                pixels.get_frame_mut()[y * 4 * Lcd::WIDTH as usize + 2] = 0x00;
                pixels.get_frame_mut()[y * 4 * Lcd::WIDTH as usize + 3] = 0xFF;
            }
            for x in 0..Lcd::WIDTH as usize {
                pixels.get_frame_mut()[x * 4 + 0] = 0xFF;
                pixels.get_frame_mut()[x * 4 + 1] = 0xFF;
                pixels.get_frame_mut()[x * 4 + 2] = 0x00;
                pixels.get_frame_mut()[x * 4 + 3] = 0xFF;
            }
            pixels.render().ok();
        }
    }
}
