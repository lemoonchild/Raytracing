use crate::color::Color;
use crate::texture::Texture;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Material {
    pub diffuse: Color,
    pub specular: f32,
    pub albedo: [f32; 4],
    pub refractive_index: f32,
    pub has_texture: bool,
    pub texture: Option<Arc<Texture>>,
}

impl Material {
    pub fn new(diffuse: Color, specular: f32, albedo: [f32; 4], refractive_index: f32) -> Self {
        Material {
            diffuse,
            specular,
            albedo,
            refractive_index,
            has_texture: false,
            texture: None,
        }
    }

    pub fn new_with_texture(
        specular: f32,
        albedo: [f32; 4],
        refractive_index: f32,
        texture: Arc<Texture>,
    ) -> Self {
        Material {
            diffuse: Color::new(255, 0, 0),
            specular,
            albedo,
            refractive_index,
            has_texture: true,
            texture: Some(texture),
        }
    }

    pub fn get_diffuse_color(&self, u: f32, v: f32) -> Color {
        if self.has_texture {
            if let Some(tex) = &self.texture {
                let u = u.clamp(0.0, 1.0);
                let v = v.clamp(0.0, 1.0);
                let x = (u * (tex.width as f32)).round() as usize;
                let y = ((1.0 - v) * (tex.height as f32)).round() as usize;
                return tex.get_color(x.min(tex.width - 1), y.min(tex.height - 1));
            }
        }
        self.diffuse
    }

    pub fn black() -> Self {
        Material {
            diffuse: Color::new(0, 0, 0),
            specular: 0.0,
            albedo: [0.0, 0.0, 0.0, 0.0],
            refractive_index: 0.0,
            has_texture: false,
            texture: None,
        }
    }
}
