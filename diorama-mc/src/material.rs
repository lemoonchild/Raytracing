use crate::color::Color;

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub diffuse: Color, 
    pub specular: f32, 
    pub albedo: [f32; 2],
    pub reflectivity: f32, 
    pub transparency: f32, 
    pub refraction_index: f32,
}

impl Material {
    pub fn new (
        diffuse: Color,
        specular: f32,
        albedo: [f32; 2],
        reflectivity: f32,
        transparency: f32, 
        refraction_index: f32,

    ) -> Self {
        Material {
            diffuse,
            specular,
            albedo,
            reflectivity,
            transparency, 
            refraction_index,
        }
    }

    pub fn black() -> Self {
        Material {
            diffuse: Color::black(),
            specular: 0.0,
            albedo: [0.0, 0.0],
            reflectivity: 0.0, 
            transparency: 0.0, 
            refraction_index: 0.0,
        }                            
    }                       
}                