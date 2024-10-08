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

use rayon::prelude::*;

const BIAS: f32 = 0.001;
const SKYBOX_COLOR: Color = Color::new(69, 142, 228);

const AMBIENT_LIGHT_COLOR: Color = Color::new(50, 50, 50);
const AMBIENT_INTENSITY: f32 = 0.3; // Intensidad de la luz ambiental

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

fn get_skybox_color(ray_direction: &Vec3, skybox: &Texture) -> Color {
    // Normaliza la dirección del rayo
    let dir = ray_direction.normalize();

    // Mapear la dirección a coordenadas UV
    let u = 0.5 + (dir.x.atan2(dir.z) / (2.0 * PI));
    let v = 0.5 - (dir.y.asin() / PI);

    // Obtener el color de la textura
    skybox.get_color_at_uv(u, v)
}

fn clamp_color(color: Color) -> Color {
    Color::new(
        color.r().min(255).max(0),
        color.g().min(255).max(0),
        color.b().min(255).max(0),
    )
}

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Cube],
    lights: &[Light], // Cambiamos de light a lights
    skybox: &Texture, 
    depth: u32,
) -> Color {
    if depth >= 3 {
        return SKYBOX_COLOR;
    }

    let mut intersect = Intersect::empty();
    let mut zbuffer = INFINITY;

    // Encontrar la intersección más cercana
    for object in objects {
        let i = object.ray_intersect(ray_origin, ray_direction);

        if i.is_intersecting && i.distance < zbuffer {
            zbuffer = i.distance;
            intersect = i;
        }
    }

    if !intersect.is_intersecting {
        return get_skybox_color(ray_direction, skybox);
    }

    // Luz ambiental
    let ambient_light = AMBIENT_LIGHT_COLOR * AMBIENT_INTENSITY;
    let mut total_light = ambient_light;

    // Procesar la contribución de cada fuente de luz
    for light in lights {
        let light_dir = (light.position - intersect.point).normalize();
        let view_dir = (ray_origin - intersect.point).normalize();
        let reflect_dir = reflect(&-light_dir, &intersect.normal).normalize();

        // Calcular la intensidad de la sombra
        let shadow_intensity = cast_shadow(&intersect, light, objects);
        let light_intensity = light.intensity * (1.0 - shadow_intensity);

        // Calcular componentes difusos y especulares
        let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
        let diffuse_color = intersect.material.get_diffuse_color(intersect.u, intersect.v);
        let diffuse = diffuse_color * intersect.material.albedo[0] * diffuse_intensity * light_intensity;

        let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(intersect.material.specular);
        let specular = light.color * intersect.material.albedo[1] * specular_intensity * light_intensity;

        total_light = total_light + diffuse + specular;
    }

    // Si el material es emisivo, añadir su contribución
    let mut emission = intersect.material.emission * 0.5;

    // Si el material es emisivo
    if emission.r() > 0 || emission.g() > 0 || emission.b() > 0 {
        // Aquí va el código actualizado
        let warm_tone = Color::new(255, 180, 100); // Un tono cálido más anaranjado

        // Calcula la distancia desde el punto de intersección al origen del rayo para atenuar la emisión
        let distance = (ray_origin - intersect.point).magnitude();
        let attenuation = 1.0 / (1.0 + 0.09 * distance + 0.032 * distance * distance); // Fórmula de atenuación

        // Mezclar la emisión del glowstone con el tono cálido y reducir la intensidad
        emission = (emission * 0.2 + warm_tone * 0.5) * attenuation; // Reduciendo la intensidad y aumentando el tono cálido

        // Agregar la emisión al total de la luz reflejada
        total_light = total_light + emission;
    }


    // Cálculo del factor de Fresnel
    let cos_theta = -intersect.normal.dot(ray_direction).max(-1.0).min(1.0);
    let r0 = ((1.0 - intersect.material.refractive_index) / (1.0 + intersect.material.refractive_index)).powi(2);
    let fresnel_reflectance = (r0 + (1.0 - r0) * (1.0 - cos_theta).powi(5)).clamp(0.0, 1.0);

    // Ajustar reflectividad con Fresnel
    let mut reflect_color = Color::black();
    if intersect.material.albedo[2] > 0.0 {
        let reflect_dir = reflect(&ray_direction, &intersect.normal).normalize();
        let reflect_origin = offset_point(&intersect, &reflect_dir);
        reflect_color = cast_ray(&reflect_origin, &reflect_dir, objects, lights, skybox, depth + 1);
    }

    // Ajustar transparencia con Fresnel
    let mut refract_color = Color::black();
    if intersect.material.albedo[3] > 0.0 {
        let refract_dir = refract(&ray_direction, &intersect.normal, intersect.material.refractive_index);
        let refract_origin = offset_point(&intersect, &refract_dir);
        refract_color = cast_ray(&refract_origin, &refract_dir, objects, lights, skybox, depth + 1);
    }

    // Incorporar Fresnel en reflectividad y transparencia
    let final_reflectivity = fresnel_reflectance * intersect.material.albedo[2];
    let final_transparency = (1.0 - fresnel_reflectance) * intersect.material.albedo[3];

    // Ajustar los valores de reflectividad y transparencia para asegurar que no excedan el rango permitido
    let scaling_factor = 1.0 / (final_reflectivity + final_transparency + (1.0 - intersect.material.albedo[2] - intersect.material.albedo[3]));

    let final_color = (total_light) * (1.0 - final_reflectivity - final_transparency) * scaling_factor +
        (reflect_color * final_reflectivity) * scaling_factor +
        (refract_color * final_transparency) * scaling_factor;

    return clamp_color(final_color);
}


pub fn render(framebuffer: &mut Framebuffer, objects: &[Cube], camera: &Camera, lights: &[Light]) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov / 2.0).tan();
    let skybox_texture = Arc::new(Texture::new("assets\\sky.png"));

    let pixels: Vec<_> = (0..framebuffer.height).flat_map(|y| {
        (0..framebuffer.width).map(move |x| (x, y))
    }).collect();

    // Calcula los colores de los píxeles en paralelo
    let pixel_colors: Vec<(usize, usize, u32)> = pixels.par_iter().map(|&(x, y)| {
        let screen_x = (2.0 * x as f32) / width - 1.0;
        let screen_y = -(2.0 * y as f32) / height + 1.0;
        let screen_x = screen_x * aspect_ratio * perspective_scale;
        let screen_y = screen_y * perspective_scale;
        let ray_direction = Vec3::new(screen_x, screen_y, -1.0).normalize();
        let rotated_direction = camera.basis_change(&ray_direction);
        let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, lights, &skybox_texture, 0);
        (x, y, pixel_color.to_hex())
    }).collect();

    // Aplica los colores de los píxeles en una operación secuencial
    for (x, y, color) in pixel_colors {
        framebuffer.set_current_color(color);
        framebuffer.point(x, y);
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

    let mut objects: Vec<Cube> = Vec::new();

    //Emisiones
    let glowstone_emission = Color::new(255, 223, 127); // Color de la luz que emite

    //Texturas
    let grass_texture = Arc::new(Texture::new("assets\\grass.png"));
    let dirt_texture = Arc::new(Texture::new("assets\\dirt.png"));
    let iron_texture = Arc::new(Texture::new("assets\\iron_ore.png"));
    let gold_texture = Arc::new(Texture::new("assets\\gold_ore.png"));
    let diamond_texture = Arc::new(Texture::new("assets\\diamond_ore.png"));
    let coal_texture = Arc::new(Texture::new("assets\\coal_ore.png"));
    let bookshelf_texture = Arc::new(Texture::new("assets\\bookshelf.png"));
    let furnance_texture = Arc::new(Texture::new("assets\\furnance.png"));
    let crafting_table_texture = Arc::new(Texture::new("assets\\crafting_table.png"));
    let crying_obsidian_texture = Arc::new(Texture::new("assets\\crying_obsidian.png"));
    let obsidian_texture = Arc::new(Texture::new("assets\\obsidian.png"));
    let chiseled_stone_texture = Arc::new(Texture::new("assets\\chiseled_stone.png"));
    let gold_block_texture = Arc::new(Texture::new("assets\\gold_block.png"));
    let magma_texture = Arc::new(Texture::new("assets\\magma.png"));
    let stone_bricks_texture = Arc::new(Texture::new("assets\\stone_bricks.png"));
    let glowstone_texture = Arc::new(Texture::new("assets\\glowstone.png"));
    let stone_texture: Arc<Texture> = Arc::new(Texture::new("assets\\stone.png"));
    let netherrack_texture: Arc<Texture> = Arc::new(Texture::new("assets\\netherrack.png"));

    let grass_material = Material::new_with_texture(0.1, [0.85, 0.1, 0.05, 0.0], 1.3, grass_texture.clone());
    let dirt_material = Material::new_with_texture(0.2, [0.9, 0.05, 0.05, 0.0], 1.0, dirt_texture);
    let iron_material = Material::new_with_texture(0.3, [0.6, 0.1, 0.0, 0.0], 1.5, iron_texture);  
    let gold_material = Material::new_with_texture(0.5, [0.6, 0.1, 0.0, 0.0], 1.3, gold_texture);  
    let diamond_material = Material::new_with_texture(0.2, [0.6, 0.05, 0.0, 0.0], 1.3, diamond_texture);  

    let coal_material = Material::new_with_texture(0.1, [0.6, 0.05, 0.0, 0.0], 1.5, coal_texture);
    let bookshelf_material = Material::new_with_texture(0.2, [0.8, 0.1, 0.0, 0.0], 1.3, bookshelf_texture);
    let furnance_material = Material::new_with_texture(0.4, [0.6, 0.3, 0.05, 0.0], 1.5, furnance_texture);
    let crafting_table_material = Material::new_with_texture(0.1, [0.85, 0.05, 0.0, 0.0], 1.3, crafting_table_texture);
    let crying_obsidian_material = Material::new_with_texture(0.1, [0.7, 0.5, 0.03, 0.0], 1.5, crying_obsidian_texture);

    let obsidian_material = Material::new_with_texture(0.1, [0.7, 0.3, 0.04, 0.0], 1.5, obsidian_texture);
    let chiseled_stone_material = Material::new_with_texture(0.1, [0.6, 0.05, 0.0, 0.0], 1.5, chiseled_stone_texture);
    let gold_block_material = Material::new_with_texture(0.1, [0.85, 0.5, 0.05, 0.0], 0.47, gold_block_texture);
    let magma_material = Material::new_with_texture(0.2, [0.7, 0.5, 0.03, 0.0], 1.5, magma_texture);
    let stone_bricks_material = Material::new_with_texture(0.1, [0.6, 0.05, 0.0, 0.0], 1.5, stone_bricks_texture);

    let glowstone_material = Material::new_with_texture_and_emission(0.2, [0.9, 0.1, 0.0, 0.0], 1.2, glowstone_emission, glowstone_texture);
    let stone_material = Material::new_with_texture(0.1, [0.6, 0.05, 0.0, 0.0], 1.5, stone_texture);
    let netherrack_material = Material::new_with_texture(0.1, [0.8, 0.1, 0.1, 0.0], 1.0, netherrack_texture);

    // Materiales al lado del portal
    let materials = [stone_material, stone_bricks_material, chiseled_stone_material];

    for i in 0..8 {
        for j in 0..8 {    

            let mut material = grass_material.clone();  // Material por defecto
            let mut place_dirt = true;  // Asumimos que se coloca tierra a menos que se especifique lo contrario

            // Especificar filas y columnas que tendrán un material diferente
            if (i == 5 && (j == 2 || j == 3 || j == 6)) || (i == 4 && (j >= 1 && j <= 6)) {
                material = netherrack_material.clone();
                place_dirt = true;  
            } else if i == 3 {
                match j {
                    1 | 6 | 5 => {
                        material = magma_material.clone();
                        place_dirt = true;  
                    },
                    2 | 7 => {
                        material = gold_block_material.clone();
                        place_dirt = true;  
                    },
                    _ => (),
                }
            } else if i == 2 {
                match j {
                    0 => {
                        material = magma_material.clone();
                        place_dirt = true;
                    },
                    _ => (),
                }
            } else if i == 1 {
                match j {
                    1 => {
                        material = magma_material.clone();
                        place_dirt = true;
                    },
                    2 => {
                        material = glowstone_material.clone();
                        place_dirt = true;
                    },
                    _ => (),
                }
            }

            // Colocar los bloques con el material especificado o el default
            objects.push(Cube {
                min: Vec3::new(i as f32, 1.0, j as f32),
                max: Vec3::new(i as f32 + 1.0, 2.0, j as f32 + 1.0),
                material: material,
            });

            // Agregar la capa de tierra debajo si es necesario
            if place_dirt {
                objects.push(Cube {
                    min: Vec3::new(i as f32, 0.0, j as f32),
                    max: Vec3::new(i as f32 + 1.0, 1.0, j as f32 + 1.0),
                    material: dirt_material.clone(),
                });
            }
    
            // Agrega bloques de material en la segunda fila en las posiciones específicas
            if i == 1 && (j == 3 || j == 4 || j == 5 || j == 6) {
                let material = match j {
                    3 => iron_material.clone(),  // Bloque de hierro
                    4 => gold_material.clone(),  // Bloque de oro
                    5 => diamond_material.clone(),  // Bloque de diamante
                    6 => coal_material.clone(),  // Bloque de carbón
                    _ => grass_material.clone(), // Este caso no debería ocurrir
                };
                objects.push(Cube {
                    min: Vec3::new(i as f32, 2.0, j as f32), // Estos bloques van encima de la grama
                    max: Vec3::new(i as f32 + 1.0, 3.0, j as f32 + 1.0),
                    material: material,
                });
            }

            if i == 6 && j == 6 {
                for k in 0..3 {  
                    objects.push(Cube {
                        min: Vec3::new(i as f32, 2.0 + k as f32, j as f32),
                        max: Vec3::new(i as f32 + 1.0, 3.0 + k as f32, j as f32 + 1.0),
                        material: bookshelf_material.clone(),
                    });
                }
            }

            if i == 5 && j == 1 {
                // Agregar el librero
                objects.push(Cube {
                    min: Vec3::new(i as f32, 2.0, j as f32),
                    max: Vec3::new(i as f32 + 1.0, 3.0 , j as f32 + 1.0),
                    material: bookshelf_material.clone(),
                });
            }
        
            // Crafting table with furnance

            if i == 5 && j == 5 {
                objects.push(Cube {
                    min: Vec3::new(i as f32, 2.0, j as f32),
                    max: Vec3::new(i as f32 + 1.0, 3.0, j as f32 + 1.0),
                    material: crafting_table_material.clone(),
                });
                
            }
            if i == 5 && j == 4 {
                objects.push(Cube {
                    min: Vec3::new(i as f32, 2.0, j as f32),
                    max: Vec3::new(i as f32 + 1.0, 3.0, j as f32 + 1.0),
                    material: furnance_material.clone(),
                });
                
            }

            // Nether portal
            if i == 6 && j == 1 {
                for k in 0..2 {  
                    objects.push(Cube {
                        min: Vec3::new(i as f32, 2.0 + k as f32, j as f32),
                        max: Vec3::new(i as f32 + 1.0, 3.0 + k as f32, j as f32 + 1.0),
                        material: crying_obsidian_material.clone(),
                    });
                }
                
            }

            if i == 6 && j == 2 {
                objects.push(Cube {
                    min: Vec3::new(i as f32, 2.0, j as f32),
                    max: Vec3::new(i as f32 + 1.0, 3.0, j as f32 + 1.0),
                    material: obsidian_material.clone(),
                });
            }

            if i == 6 && j == 3 {
                // Agregar el bloque de crying obsidian
                objects.push(Cube {
                    min: Vec3::new(i as f32, 2.0, j as f32),
                    max: Vec3::new(i as f32 + 1.0, 3.0, j as f32 + 1.0),
                    material: crying_obsidian_material.clone(),
                });

                // Agregar el glowstone encima del bloque de crying obsidian
                objects.push(Cube {
                    min: Vec3::new(i as f32, 3.0, j as f32), // Una capa arriba del crying obsidian
                    max: Vec3::new(i as f32 + 1.0, 4.0, j as f32 + 1.0),
                    material: glowstone_material.clone(), // Usar el material de glowstone
                });
            }


            if i == 6 && j == 4 {
                for k in 0..4 {  
                    objects.push(Cube {
                        min: Vec3::new(i as f32, 2.0 + k as f32, j as f32),
                        max: Vec3::new(i as f32 + 1.0, 3.0 + k as f32, j as f32 + 1.0),
                        material: obsidian_material.clone(),
                    });
                }
            }
        
            // Crear la pila de bloques
            for (index, material) in materials.iter().enumerate() {
                let k = index as f32; // Usar 'index' para incrementar la altura (k)
                objects.push(Cube {
                    min: Vec3::new(6.0, 2.0 + k, 5.0),
                    max: Vec3::new(7.0, 3.0 + k, 6.0),
                    material: material.clone(),
                });
            }
        }
    }    

    // Configuración de la cámara
    let mut camera = Camera::new(
        Vec3::new(-5.0, 5.0, -10.0), // Posición de la cámara ajustada
        Vec3::new(0.0, 0.0, 0.0),   // Punto hacia el que mira la cámara
        Vec3::new(0.0, 1.0, 0.0),   // Vector "up" de la cámara
    );

    // Crear una lista de luces que incluirá la luz principal y las luces de los bloques glowstone
    let mut lights: Vec<Light> = vec![
        Light::new(Vec3::new(-5.0, 10.0, -10.0), Color::new(255, 255, 255), 1.0), // Luz principal
    ];

    // Ahora recorremos todos los objetos y añadimos los bloques de glowstone como fuentes de luz
    for object in &objects {
        if object.material.emission.r() > 0 || object.material.emission.g() > 0 || object.material.emission.b() > 0 {
            lights.push(Light::new(
                object.min + Vec3::new(0.5, 0.5, 0.5), // Centro del bloque glowstone
                object.material.emission,
                0.8, // Ajusta la intensidad para las luces glowstone
            ));
        }
    }
    
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
        render(&mut framebuffer, &objects, &mut camera, &lights);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}

