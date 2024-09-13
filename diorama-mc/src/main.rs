use minifb::{Key, Window, WindowOptions};
use core::f32;
use std::time::Duration;
use nalgebra_glm::Vec3;

use std::f32::consts::PI; 
use std::f32::INFINITY; 

mod framebuffer;
use framebuffer::Framebuffer; 

mod sphere;
use sphere::Sphere; 

mod ray_intersect;
use ray_intersect::{RayIntersect, Intersect};

mod color;
use color::Color; 

mod camera;
use camera::Camera; 

mod material;
use material::Material; 

mod light;
use light::Light; 

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

fn cast_shadow(
    intersect: &Intersect,
    light: &Light,
    objects: &[Sphere],
) -> f32 {
    let light_dir = (light.position - intersect.point).normalize();
    let shadow_ray_origin = intersect.point;
    let mut shadow_intensity = 0.0;

    for object in objects {
        let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_dir);
        if shadow_intersect.is_intersecting {
            shadow_intensity = 0.7;
            break;
        }
    }

    shadow_intensity
}

pub fn cast_ray(ray_origin: &Vec3, ray_direction: &Vec3, objects: &[Sphere], light: &Light) -> Color {

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
        return Color::new(4, 12, 36); 
    }

    let light_dir = (light.position - intersect.point).normalize(); 
    let view_dir = (ray_origin - intersect.point).normalize();

    let reflect_dir = reflect(&-light_dir, &intersect.normal).normalize();

    let shadow_intensity = cast_shadow(&intersect, light, objects);
    let light_intensity = light.intensity * (1.0 - shadow_intensity); 

    let diffuse_intensity = intersect.normal.dot(&light_dir); 
    let diffuse = intersect.material.diffuse * intersect.material.albedo[0] * diffuse_intensity * light_intensity; 

    let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(intersect.material.specular); 
    let specular = light.color * intersect.material.albedo[1] * specular_intensity * light_intensity; 

    diffuse + specular
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Sphere], camera: &Camera, light: &Light) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI/3.0; 
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

            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light);

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

    let rubber = Material::new(
        Color::new(80, 0, 0),
        10.0,
        [0.9, 0.1],
    ); 

    let ivory = Material::new(
        Color::new(100, 100, 100),
        50.0,
        [0.6, 0.3],
    );

    let objects = [
        Sphere {
            center: Vec3::new(0.0, 0.0, 3.5),
            radius: 0.1, 
            material: ivory,
        },
        Sphere {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: 1.0,
            material: rubber,
        }
    ];


    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let light = Light::new(
      Vec3::new(0.0, 0.0, 5.0),
      Color::new(255, 255, 255),
      1.0,
    );

    let rotation_speed = PI/50.0; 

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        // camera orbit controls
        if window.is_key_down(Key::Left) {
            camera.orbit(rotation_speed, 0.0);
        }
        if window.is_key_down(Key::Right) {
            camera.orbit(-rotation_speed, 0.0);
        }
        if window.is_key_down(Key::Up) {
            camera.orbit(0.0, -rotation_speed);
        }
        if window.is_key_down(Key::Down) {
            camera.orbit(0.0, rotation_speed);
        }

        framebuffer.clear();
        render(&mut framebuffer, &objects, &mut camera, &light);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}