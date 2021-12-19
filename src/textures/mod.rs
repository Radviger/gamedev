use std::path::Path;
use std::sync::Arc;
use glium::{Display, Texture2d};
use glium::texture::{RawImage2d, SrgbTexture2d};
use image::GenericImageView;

pub fn load<N>(display: &Display, name: N) -> Arc<SrgbTexture2d> where N: AsRef<Path> {
    let image = image::open(name).expect("unable to open image");
    let size = image.dimensions();
    let raw = RawImage2d::from_raw_rgba_reversed(&image.into_rgba8(), size);
    let texture = SrgbTexture2d::new(display, raw).expect("failed to allocate texture");
    Arc::new(texture)
}