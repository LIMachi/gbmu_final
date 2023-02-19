use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::rc::Rc;
use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use shared::utils::Cell;
use shared::winit as winit;

#[derive(Default)]
pub struct Buffer {
    enabled: bool,
    pixels: Option<Pixels>
}

#[derive(Clone)]
pub struct Lcd {
    inner: Rc<RefCell<Buffer>>
}

impl LCD for Lcd {
    fn set(&mut self, x: usize, y: usize, pixel: [u8; 3]) {
        let mut lcd = self.inner.as_ref().borrow_mut();
        if !lcd.enabled {
            return ;
        }
        if let Some(mut pixels) = lcd.pixels.as_mut() {
            let frame = pixels.get_frame_mut();
            let f = (Lcd::WIDTH * 4) as usize;
            frame[x * 4 + 0 + y * f] = pixel[0];
            frame[x * 4 + 1 + y * f] = pixel[1];
            frame[x * 4 + 2 + y * f] = pixel[2];
            frame[x * 4 + 3 + y * f] = 0xFF;
        }
    }

    fn enable(&mut self) {
        self.inner.as_ref().borrow_mut().enabled = true;
    }

    fn disable(&mut self) {
        let mut lcd = self.inner.as_ref().borrow_mut();
        lcd.enabled = false;
        if let Some(mut pixels) = lcd.pixels.as_mut() {
            let pixels = pixels.get_frame_mut();
            for i in 0..(4 * Lcd::WIDTH * Lcd::HEIGHT) as usize {
                pixels[i] = if i % 4 == 3 { 0xFF } else { 0xAA };
            }
        }
    }
}

impl Lcd {
    pub const WIDTH: u32 = 160;
    pub const HEIGHT: u32 = 144;

    pub fn new() -> Self {
        Self {
            inner: Buffer::default().cell()
        }
    }

    pub fn init(&mut self, window: &winit::window::Window) {
        let sz = window.inner_size();
        let surf = SurfaceTexture::new(sz.width, sz.height, window);
        self.inner.as_ref().borrow_mut().pixels.replace( PixelsBuilder::new(Lcd::WIDTH, Lcd::HEIGHT, surf).build().unwrap());
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(pixels) = self.inner.as_ref().borrow_mut().pixels.as_mut() {
            pixels.resize_surface(width, height).ok();
        }
    }

    pub fn render(&mut self) {
        if let Some(pixels) = self.inner.as_ref().borrow_mut().pixels.as_mut() {
            pixels.render().ok();
        }
    }
}

pub trait LCD {

    fn set(&mut self, x: usize, y: usize, pixel: [u8; 3]);
    fn enable(&mut self);
    fn disable(&mut self);
}
