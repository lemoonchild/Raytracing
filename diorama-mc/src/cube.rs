use crate::material::Material;
use crate::ray_intersect::{Intersect, RayIntersect};
use nalgebra_glm::Vec3;

pub struct Cube {
    pub min: Vec3,          // La esquina mínima del cubo
    pub max: Vec3,          // La esquina máxima del cubo
    pub material: Material, // Material del cubo (incluyendo albedo)
}

impl Cube {
    pub fn get_uv(&self, point: &Vec3, normal: &Vec3) -> (f32, f32) {
        let size = self.max - self.min;
        let local_point = point - self.min;

        let img_width = 375.0;
        let img_height = 500.0;
        let num_columns = 3.0;
        let num_rows = 4.0;

        let column_width = img_width / num_columns;
        let row_height = img_height / num_rows;

        let mut u = 0.0;
        let mut v = 0.0;

        let (u, v) = if normal.x > 0.0 {
            // Cara derecha (Face 6)
            (
                (local_point.y / size.y) * column_width + 2.0 * column_width, // Columna 3
                (1.0 - local_point.z / size.z) * row_height + row_height,
            ) // Fila 2
        } else if normal.x < 0.0 {
            // Cara izquierda (Face 4)
            (
                (1.0 - local_point.y / size.y) * column_width, // Columna 1
                (1.0 - local_point.z / size.z) * row_height + row_height,
            ) // Fila 2
        } else if normal.y > 0.0 {
            // Cara superior (Face 11)
            (
                (local_point.x / size.x) * column_width + column_width, // Columna 2
                (local_point.z / size.z) * row_height + 3.0 * row_height,
            ) // Fila 4
        } else if normal.y < 0.0 {
            // Cara inferior (Face 5)
            (
                (local_point.x / size.x) * column_width + column_width, // Columna 2
                (local_point.z / size.z) * row_height + row_height,
            ) // Fila 2
        } else if normal.z > 0.0 {
            // Cara frontal (Face 2)
            (
                (local_point.x / size.x) * column_width + column_width, // Columna 2
                (1.0 - local_point.y / size.y) * row_height,
            ) // Fila 1
        } else {
            // Cara trasera (Face 8)
            (
                (local_point.x / size.x) * column_width + column_width, // Columna 2
                (local_point.y / size.y) * row_height + 2.0 * row_height,
            ) // Fila 3
        };

        // Convertir coordenadas de píxeles a coordenadas UV dividiendo por las dimensiones de la imagen
        (u / img_width, 1.0 - v / img_height)
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        // Calcular el inverso de cada componente del vector de dirección
        let inv_dir = Vec3::new(
            1.0 / ray_direction[0],
            1.0 / ray_direction[1],
            1.0 / ray_direction[2],
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
