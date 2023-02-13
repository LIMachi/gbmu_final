use std::io::Read;
use std::path::Path;
use egui::{Context, TextureHandle, TextureOptions};
use egui_extras::image::{FitTo, load_svg_bytes_with_size};

pub trait ImageLoader {
    fn load_image<S: Into<String>, P: AsRef<std::path::Path>>(&mut self, name: S, path: P) -> TextureHandle;
    fn load_svg<const W: u32, const H: u32>(&mut self, name: impl Into<String>, path: impl AsRef<Path>) -> TextureHandle;
}

impl ImageLoader for Context {
    fn load_image<S: Into<String>, P: AsRef<Path>>(&mut self, name: S, path: P) -> TextureHandle {
        let img = load_image_from_path(path.as_ref()).unwrap();
        self.load_texture(name, img, TextureOptions::LINEAR)
    }

    fn load_svg<const W: u32, const H: u32>(&mut self, name: impl Into<String>, path: impl AsRef<Path>) -> TextureHandle {
        let img = load_svg_from_path::<W, H>(path.as_ref()).unwrap();
        self.load_texture(name, img, TextureOptions::LINEAR)
    }
}

pub fn load_image_from_path(path: &Path) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::io::Reader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

pub fn load_svg_from_path<const W: u32, const H: u32>(path: &Path) -> Result<egui::ColorImage, String> {
    let buf = { let mut buf = vec![]; std::fs::File::open(path).unwrap().read_to_end(&mut buf).ok(); buf };
    load_svg_bytes_with_size(&buf, FitTo::Size(W, H))
}
