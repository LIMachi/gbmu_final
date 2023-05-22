use std::fmt::{Debug, Formatter};
use std::io::Read;
use std::path::Path;

use egui::{Context, TextureHandle, TextureOptions};
use egui_extras::image::{FitTo, load_svg_bytes_with_size};
use serde::{Deserialize, Serialize};
use winit::window::Icon;

pub type Image<const W: usize, const H: usize> = [[[u32; 3]; W]; H];

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RawData {
    pub w: usize,
    pub h: usize,
    pub data: Vec<u8>,
}

impl Debug for RawData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawIcon")
            .field("width", &self.w)
            .field("height", &self.h)
            .field("data", &format!("[...]({})", self.data.len()))
            .finish()
    }
}

impl RawData {
    pub fn icon(&self) -> Option<Icon> {
        Icon::from_rgba(self.data.clone(), self.w as u32, self.h as u32).ok()
    }
}

pub trait ImageLoader {
    fn load_image<S: Into<String>, P: AsRef<std::path::Path>>(&self, name: S, path: P) -> Option<(TextureHandle, RawData)>;
    fn load_svg<const W: u32, const H: u32>(&mut self, name: impl Into<String>, path: impl AsRef<Path>) -> TextureHandle;
    fn load_svg_bytes<const W: u32, const H: u32>(&mut self, name: impl Into<String>, bytes: &[u8]) -> TextureHandle;
}

impl ImageLoader for Context {
    fn load_image<S: Into<String>, P: AsRef<Path>>(&self, name: S, path: P) -> Option<(TextureHandle, RawData)> {
        load_image_from_path(path.as_ref()).ok()
            .map(|(raw, img)| (self.load_texture(name, img, TextureOptions::LINEAR), raw))
    }

    fn load_svg<const W: u32, const H: u32>(&mut self, name: impl Into<String>, path: impl AsRef<Path>) -> TextureHandle {
        let img = load_svg_from_path::<W, H>(path.as_ref()).unwrap();
        self.load_texture(name, img, TextureOptions::LINEAR)
    }

    fn load_svg_bytes<const W: u32, const H: u32>(&mut self, name: impl Into<String>, bytes: &[u8]) -> TextureHandle {
        let name = name.into();
        let tex = load_svg_bytes_with_size(bytes, FitTo::Size(W, H)).expect(format!("could not load {}", name).as_str());
        self.load_texture(name, tex, TextureOptions::LINEAR)
    }
}

pub fn load_image_from_path(path: &Path) -> Result<(RawData, egui::ColorImage), image::ImageError> {
    let image = image::io::Reader::open(path)?.with_guessed_format()?.decode()?;
    let [w, h] = [image.width() as usize, image.height() as usize];
    let image_buffer = image.to_rgba8();
    let data = image_buffer.as_raw().clone();
    let pixels = image_buffer.as_flat_samples();
    Ok((RawData { w, h, data },
        egui::ColorImage::from_rgba_unmultiplied(
            [w, h],
            pixels.as_slice(),
        )))
}

pub fn load_svg_from_path<const W: u32, const H: u32>(path: &Path) -> Result<egui::ColorImage, String> {
    let buf = {
        let mut buf = vec![];
        std::fs::File::open(path).unwrap().read_to_end(&mut buf).ok();
        buf
    };
    load_svg_bytes_with_size(&buf, FitTo::Size(W, H))
}
