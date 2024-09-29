extern crate image;
use crate::color::Color;
use image::{DynamicImage, GenericImageView, ImageReader, Pixel};
use std::fmt;

#[derive(Clone)]
pub struct Texture {
    pub id: String,
    image: DynamicImage,
    pub width: usize,
    pub height: usize,
    color_array: Vec<Color>,
}

impl Texture {
    pub fn new(file_path: &str) -> Texture {
        let img = ImageReader::open(file_path)
            .expect("Failed to open file")
            .decode()
            .expect("Failed to decode image");

        let width = img.width() as usize;
        let height = img.height() as usize;

        // Verificar que la imagen no tenga dimensiones nulas
        assert!(
            width > 0 && height > 0,
            "Image is empty or not loaded correctly"
        );

        let mut texture = Texture {
            id: file_path.to_string(),
            image: img,
            width,
            height,
            color_array: vec![Color::black(); width * height],
        };
        texture.load_color_array();
        texture
    }

    fn load_color_array(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {
                let pixel = self.image.get_pixel(x as u32, y as u32).to_rgb();
                let color =
                    ((pixel[0] as u32) << 16) | ((pixel[1] as u32) << 8) | (pixel[2] as u32);
                self.color_array[y * self.width + x] = Color::from_hex(color);
            }
        }
    }

    pub fn get_color(&self, x: usize, y: usize) -> Color {
        if x < self.width && y < self.height {
            self.color_array[y * self.width + x]
        } else {
            Color::from_hex(0xFF00FF) // Magenta para indicar error de coordenadas
        }
    }

    pub fn get_color_at_uv(&self, u: f32, v: f32) -> Color {
        // Asegúrate de que u y v estén en el rango [0, 1]
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        // Convertir las coordenadas UV a coordenadas de la textura
        let x = (u * (self.width as f32 - 1.0)) as usize;
        let y = ((1.0 - v) * (self.height as f32 - 1.0)) as usize; // invertimos v ya que las coordenadas de la textura generalmente comienzan desde la parte superior

        // Retorna el color de la textura
        self.get_color(x, y)
    }
}

impl fmt::Debug for Texture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Texture")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}
