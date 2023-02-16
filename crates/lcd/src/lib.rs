use std::cell::RefCell;
use std::rc::Rc;
use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use shared::utils::Cell;
use shared::winit as winit;

#[derive(Default)]
pub struct Lcd {
    pixels: Rc<RefCell<Option<Pixels>>>
}

impl Lcd {
    pub const WIDTH: u32 = 160;
    pub const HEIGHT: u32 = 144;

    pub fn init(&mut self, window: &winit::window::Window) {
        let sz = window.inner_size();
        let surf = SurfaceTexture::new(sz.width, sz.height, window);
        self.pixels.replace( PixelsBuilder::new(Lcd::WIDTH, Lcd::HEIGHT, surf).build().ok());
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(pixels) = self.pixels.as_ref().borrow_mut().as_mut() {
            pixels.resize_surface(width, height).ok();
        }
    }

    pub fn render(&mut self) {
        if let Some(pixels) = self.pixels.as_ref().borrow_mut().as_mut() {
            pixels.render().ok();
        }
    }

    pub fn framebuffer(&self) -> Rc<RefCell<dyn Framebuffer>> {
        self.pixels.clone()
    }
}

pub trait Framebuffer {
    fn set(&mut self, x: usize, y: usize, pixel: [u8; 3]);
}

impl Framebuffer for Option<Pixels> {
    fn set(&mut self, x: usize, y: usize, pixel: [u8; 3]) {
        if let Some(mut pixels) = self.as_mut() {
            let frame = pixels.get_frame_mut();
            let f = (Lcd::WIDTH * 4) as usize;
            frame[x * 4 + 0 + y * f] = pixel[0];
            frame[x * 4 + 1 + y * f] = pixel[1];
            frame[x * 4 + 2 + y * f] = pixel[2];
            frame[x * 4 + 3 + y * f] = 0xFF;
        }
    }
}
