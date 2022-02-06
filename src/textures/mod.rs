use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use glium::{Display, Texture2d};
use glium::texture::{RawImage2d, SrgbTexture2d};
use image::{ColorType, GenericImageView};

pub fn load<N>(display: &Display, name: N) -> Arc<SrgbTexture2d> where N: AsRef<Path> {
    let image = image::open(name).expect("unable to open image");
    let size = image.dimensions();
    let raw = RawImage2d::from_raw_rgba_reversed(&image.into_rgba8(), size);
    let texture = SrgbTexture2d::new(display, raw).expect("failed to allocate texture");
    Arc::new(texture)
}

pub struct TextureManager {
    pub display: Display,
    pub textures: HashMap<String, Rc<Box<SrgbTexture2d>>>
}

#[macro_export]
macro_rules! texture {
    ($manager:expr, $name:literal) => {{
        use crate::image::{self, GenericImageView};
        static IMAGE_BUF: &'static [u8] = include_bytes!(concat!("resources/", $name, ".png"));
        let manager = $manager;
        let image = image::load_from_memory_with_format(&IMAGE_BUF, image::ImageFormat::Png)
            .expect("Image loading failed");
        let size = image.dimensions();
        let image = glium::texture::RawImage2d::from_raw_rgba(image.raw_pixels(), size);
        let texture = glium::texture::SrgbTexture2d::new(manager.display, image).expect("Texture allocation failed");
        manager.textures.insert($name.into(), std::rc::Rc::new(Box::new(texture)));
    }};
}

impl TextureManager {
    pub fn new(display: &Display) -> TextureManager {
        TextureManager {
            display: display.clone(),
            textures: HashMap::new()
        }
    }

    pub fn get<T>(&self, name: T) -> Rc<Box<SrgbTexture2d>> where T: AsRef<str> {
        self.textures.get(name.as_ref()).cloned().expect(&format!("Missing texture: {}", name.as_ref()))
    }

    pub fn get_or_load<P>(&mut self, name: String, path: P) -> Option<Rc<Box<SrgbTexture2d>>> where P: AsRef<Path> {
        if !self.textures.contains_key(&name) {
            let image = image::open(path.as_ref())
                .expect(&format!("Image loading failed: {}", name));

            let size = image.dimensions();
            let has_alpha = image.color().has_alpha();
            let image: RawImage2d<u8> = if has_alpha {
                RawImage2d::from_raw_rgba(image.into_rgba8().into_raw(), size)
            } else {
                RawImage2d::from_raw_rgb(image.into_rgb8().into_raw(), size)
            };
            let texture = SrgbTexture2d::new(&self.display, image).expect("Texture allocation failed");
            self.textures.insert(name.clone(), Rc::new(Box::new(texture)));
        }
        self.textures.get(&name).cloned()
    }
}