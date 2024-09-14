use nalgebra_glm::Vec3; 
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::material::Material; 

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Material,
}

impl Sphere {
    fn get_uv(&self, point: &Vec3) -> (f32, f32) {
        let normalized = (point - self.center) / self.radius;
        let u = 0.5 + (normalized.z.atan2(normalized.x) / (2.0 * std::f32::consts::PI));
        let v = 0.5 - (normalized.y.asin() / std::f32::consts::PI);
        (u, v)
    }
}

impl RayIntersect for Sphere {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        // Vector from the ray origin to the center of the sphere
        let oc = *ray_origin - self.center;

        // Coefficients for the quadratic equation
        let a = ray_direction.dot(ray_direction);
        let b = 2.0 * oc.dot(ray_direction);
        let c = oc.dot(&oc) - self.radius * self.radius;

        // Calculate the discriminant
        let discriminant = b * b - 4.0 * a * c;

        // If the discriminant is greater than 0, the ray intersects the sphere
        if discriminant > 0.0 {
            // Calculate the nearest point of intersection
            let t = (-b - discriminant.sqrt()) / (2.0 * a);
            if t > 0.0 {
                // Compute intersection point, normal at the intersection, and distance from the ray origin
                let point = ray_origin + ray_direction * t;
                let normal = (point - self.center).normalize();
                let distance = t;
                let (u, v) = self.get_uv(&point);

                return Intersect::new(point, normal, distance, self.material.clone(), u, v);
            }
        }
        
        // If no intersection, return an empty intersect
        Intersect::empty()
        

    }
}
