use core::f32;
use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::Vec3;
use std::time::Duration;

use std::f32::consts::PI;
use std::f32::INFINITY;

mod framebuffer;
use framebuffer::Framebuffer;

mod sphere;
use sphere::Sphere;

mod ray_intersect;
use ray_intersect::{Intersect, RayIntersect};

mod color;
use color::Color;

mod camera;
use camera::Camera;

mod material;
use material::Material;

mod light;
use light::Light;

mod texture;
use std::sync::Arc;
use texture::Texture;

mod cube;
use cube::Cube;

const BIAS: f32 = 0.001;
const SKYBOX_COLOR: Color = Color::new(69, 142, 228);

fn offset_point(intersect: &Intersect, direction: &Vec3) -> Vec3 {
    let offset = intersect.normal * BIAS;
    intersect.point + offset
}

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

fn refract(incident: &Vec3, normal: &Vec3, eta_t: f32) -> Vec3 {
    let cosi = -incident.dot(normal).max(-1.0).min(1.0);

    let n_cosi: f32;
    let eta: f32;
    let n_normal: Vec3;

    if cosi < 0.0 {
        // Entering
        n_cosi = -cosi;
        eta = 1.0 / eta_t;
        n_normal = -normal;
    } else {
        // Leaving
        n_cosi = cosi;
        eta = eta_t;
        n_normal = *normal;
    }

    let k = 1.0 - eta * eta * (1.0 - n_cosi * n_cosi);

    if k > 0.0 {
        // Total internal reflection
        reflect(incident, &n_normal)
    } else {
        incident * eta + (eta * n_cosi - k.sqrt()) * n_normal
    }
}

fn cast_shadow(intersect: &Intersect, light: &Light, objects: &[Cube]) -> f32 {
    let light_dir = (light.position - intersect.point).normalize();
    let light_distance = (light.position - intersect.point).magnitude();

    let shadow_ray_origin = offset_point(intersect, &light_dir);
    let mut shadow_intensity = 0.0;

    for object in objects {
        let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_dir);
        if shadow_intersect.is_intersecting && shadow_intersect.distance < light_distance {
            let distance_ratio = shadow_intersect.distance / light_distance;
            shadow_intensity = 0.50 - distance_ratio.powf(2.0).min(1.0);

            break;
        }
    }

    shadow_intensity
}

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Cube],
    light: &Light,
    depth: u32,
) -> Color {
    if depth >= 3 {
        return SKYBOX_COLOR;
    }

    let mut intersect = Intersect::empty();
    let mut zbuffer = INFINITY;

    for object in objects {
        let i = object.ray_intersect(ray_origin, ray_direction);

        if i.is_intersecting && i.distance < zbuffer {
            zbuffer = i.distance;
            intersect = i;
        }
    }

    if !intersect.is_intersecting {
        return SKYBOX_COLOR;
    }

    let light_dir = (light.position - intersect.point).normalize();
    let view_dir = (ray_origin - intersect.point).normalize();

    let reflect_dir = reflect(&-light_dir, &intersect.normal).normalize();

    let shadow_intensity = cast_shadow(&intersect, light, objects);
    let light_intensity = light.intensity * (1.0 - shadow_intensity);

    let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
    let diffuse_color = intersect
        .material
        .get_diffuse_color(intersect.u, intersect.v);
    let diffuse =
        diffuse_color * intersect.material.albedo[0] * diffuse_intensity * light_intensity;

    let specular_intensity = view_dir
        .dot(&reflect_dir)
        .max(0.0)
        .powf(intersect.material.specular);
    let specular =
        light.color * intersect.material.albedo[1] * specular_intensity * light_intensity;

    let mut reflect_color = Color::black();
    let reflectivity = intersect.material.albedo[2];

    if reflectivity > 0.0 {
        let reflect_dir = reflect(&ray_direction, &intersect.normal).normalize();
        let reflect_origin = offset_point(&intersect, &ray_direction);
        reflect_color = cast_ray(&reflect_origin, &reflect_dir, objects, light, depth + 1)
    }

    let mut refract_color = Color::black();
    let transparency = intersect.material.albedo[3];

    if transparency > 0.0 {
        let refract_dir = refract(
            &ray_direction,
            &intersect.normal,
            intersect.material.refractive_index,
        );
        let refract_origin = offset_point(&intersect, &refract_dir);
        refract_color = cast_ray(&refract_origin, &refract_dir, objects, light, depth + 1);
    }

    (diffuse + specular) * (1.0 - reflectivity - transparency)
        + (reflect_color * reflectivity)
        + (refract_color * transparency)
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Cube], camera: &Camera, light: &Light) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov / 2.0).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            // Map the pixel coordinate to screen space [-1, 1]
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            // Adjust for aspect ratio
            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            // Calculate the direction of the ray for this pixel
            let ray_direction = &Vec3::new(screen_x, screen_y, -1.0).normalize();

            let rotated_direction = camera.basis_change(&ray_direction);

            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light, 0);

            // Draw the pixel on screen with the returned color
            framebuffer.set_current_color(pixel_color.to_hex());
            framebuffer.point(x, y);
        }
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;

    let framebuffer_width = 800;
    let framebuffer_height = 600;

    let frame_delay = Duration::from_millis(0);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);

    let mut window = Window::new(
        "Gráficas - Diorama Minecraft",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    window.set_position(100, 100);
    window.update();

    framebuffer.set_background_color(0x333355);

    let grass_texture = Arc::new(Texture::new("assets\\grass.png"));
    let grass = Material::new_with_texture(1.0, [1.0, 0.05, 0.0, 0.0], 0.0, grass_texture);

    let ivory: Material = Material::new(Color::new(100, 100, 80), 50.0, [0.6, 0.3, 0.6, 0.0], 0.0);

    let glass = Material::new(
        Color::new(255, 255, 255),
        1425.0,
        [0.0, 10.0, 0.5, 0.5],
        0.3,
    );

    let objects = [Cube {
        // Base o terreno
        min: Vec3::new(0.0, 1.0, 0.0),
        max: Vec3::new(1.0, 2.0, 1.0),
        material: grass,
    }];

    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let light = Light::new(Vec3::new(3.0, -3.0, 3.0), Color::new(255, 255, 255), 1.0);

    let rotation_speed = PI / 50.0;
    let movement_speed = 0.1;
    let zoom_speed = 0.5;

    while window.is_open() {
        // listen to inputs
        if window.is_key_down(Key::Escape) {
            break;
        }

        //  camera orbit controls
        if window.is_key_down(Key::Left) {
            camera.orbit(rotation_speed, 0.0);
        }
        if window.is_key_down(Key::Right) {
            camera.orbit(-rotation_speed, 0.0);
        }
        if window.is_key_down(Key::W) {
            camera.orbit(0.0, -rotation_speed);
        }
        if window.is_key_down(Key::S) {
            camera.orbit(0.0, rotation_speed);
        }

        // Camera movement controls
        let mut movement = Vec3::new(0.0, 0.0, 0.0);
        if window.is_key_down(Key::A) {
            movement.x -= movement_speed;
        }
        if window.is_key_down(Key::D) {
            movement.x += movement_speed;
        }
        if window.is_key_down(Key::Q) {
            movement.y += movement_speed;
        }
        if window.is_key_down(Key::E) {
            movement.y -= movement_speed;
        }
        if movement.magnitude() > 0.0 {
            camera.move_center(movement);
        }

        // Camera zoom controls
        if window.is_key_down(Key::Up) {
            camera.zoom(zoom_speed);
        }
        if window.is_key_down(Key::Down) {
            camera.zoom(-zoom_speed);
        }

        framebuffer.clear();
        render(&mut framebuffer, &objects, &mut camera, &light);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}
