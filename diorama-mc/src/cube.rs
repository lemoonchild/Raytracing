use nalgebra_glm::Vec3;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::material::Material;

pub struct Cube {
    pub min: Vec3, // La esquina mínima del cubo
    pub max: Vec3, // La esquina máxima del cubo
    pub material: Material, // Material del cubo (incluyendo albedo)
}

impl Cube {
    pub fn get_uv(&self, point: &Vec3, normal: &Vec3) -> (f32, f32) {
        // Toma el tamaño de cada segmento de la textura
        let segment_height = 1.0 / 6.0;  // Asumiendo 6 filas iguales
        let segment_width = 1.0;
    
        let mut u = 0.0;
        let mut v = 0.0;
    
        if normal.x.abs() > 0.99 {  // Caras laterales (left/right)
            u = (point.y - self.min.y) / (self.max.y - self.min.y);
            v = (point.z - self.min.z) / (self.max.z - self.min.z);
            v = segment_height * (if normal.x > 0.0 { 4.0 } else { 5.0 }) + v * segment_height;
        } else if normal.y.abs() > 0.99 {  // Caras superior/inferior (top/bottom)
            u = (point.x - self.min.x) / (self.max.x - self.min.x);
            v = (point.z - self.min.z) / (self.max.z - self.min.z);
            v = segment_height * (if normal.y > 0.0 { 0.0 } else { 1.0 }) + v * segment_height;
        } else if normal.z.abs() > 0.99 {  // Caras frontal/posterior (front/back)
            u = (point.x - self.min.x) / (self.max.x - self.min.x);
            v = (point.y - self.min.y) / (self.max.y - self.min.y);
            v = segment_height * (if normal.z > 0.0 { 2.0 } else { 3.0 }) + v * segment_height;
        }
    
        (u, v)
    }    
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        // Calcular el inverso de cada componente del vector de dirección
        let inv_dir = Vec3::new(
            1.0 / ray_direction[0],
            1.0 / ray_direction[1],
            1.0 / ray_direction[2]
        );

        let tmin = (self.min - ray_origin).component_mul(&inv_dir);
        let tmax = (self.max - ray_origin).component_mul(&inv_dir);

        let t1 = tmin[0].min(tmax[0]);
        let t2 = tmin[0].max(tmax[0]);
        let t3 = tmin[1].min(tmax[1]);
        let t4 = tmin[1].max(tmax[1]);
        let t5 = tmin[2].min(tmax[2]);
        let t6 = tmin[2].max(tmax[2]);

        let t_enter = t1.max(t3).max(t5);
        let t_exit = t2.min(t4).min(t6);

        if t_enter < t_exit && t_exit > 0.0 {
            let point = ray_origin + ray_direction * t_enter;
            let normal = if (point[0] - self.min[0]).abs() < 1e-3 {
                Vec3::new(-1.0, 0.0, 0.0)
            } else if (point[0] - self.max[0]).abs() < 1e-3 {
                Vec3::new(1.0, 0.0, 0.0)
            } else if (point[1] - self.min[1]).abs() < 1e-3 {
                Vec3::new(0.0, -1.0, 0.0)
            } else if (point[1] - self.max[1]).abs() < 1e-3 {
                Vec3::new(0.0, 1.0, 0.0)
            } else if (point[2] - self.min[2]).abs() < 1e-3 {
                Vec3::new(0.0, 0.0, -1.0)
            } else {
                Vec3::new(0.0, 0.0, 1.0)
            };

            let (u, v) = self.get_uv(&point, &normal);
            return Intersect::new(point, normal, t_enter, self.material.clone(), u, v);
        }

        Intersect::empty()
    }
}



