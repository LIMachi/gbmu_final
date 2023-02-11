use std::io::Read;
use egui_extras::image::{FitTo, load_svg_bytes_with_size};

pub fn load_image_from_path(path: &std::path::Path) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::io::Reader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

pub fn load_svg_from_path<const W: u32, const H: u32>(path: &std::path::Path) -> Result<egui::ColorImage, String> {
    let buf = { let mut buf = vec![]; std::fs::File::open(path).unwrap().read_to_end(&mut buf).ok(); buf };
    load_svg_bytes_with_size(&buf, FitTo::Size(W, H))
}
