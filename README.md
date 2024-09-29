# Diorama of Minecraft - Project 02 

### Description
This project is a 3D diorama inspired by Minecraft, developed as part of a Computer Graphics course. It uses ray tracing techniques to simulate realistic lighting, reflections, and material properties, creating a visually engaging miniature world. The diorama includes several blocks, each with unique textures, properties, and even light emission, to closely mimic the look and feel of a Minecraft scene.

### Features
- **Camera Constrols:** 
  - You can freely explore the diorama using the following controls:
    - **Orbit:** 
      - **Left Arrow (←):** Rotate the camera left.
      - **Right Arrow (→):** Rotate the camera right.
      - **W:** Rotate the camera up.
      - **S:** Rotate the camera down.
    - **Movement:** 
      - **A:** Move the camera left.
      - **D:** Move the camera right.
      - **Q:** Move the camera up.
      - **E:** Move the camera down.
    - **Zoom:** 
      - **Up Arrow (↑)** Zoom in.
      - **Down Arrow (↓):** Zoom out.

- **Material Properties:** 
  - Each block has distinct materials with unique properties:
    - **Specular Reflection:**  Controls the shininess of the material.
    - **Albedo:** Includes diffuse, specular, reflectivity, and transparency
    - **Refraction:** Determines how light bends through transparent materials.
    - Fresnel effect is used for calculating transparency and reflectivity, providing a more realistic representation of how light interacts with surfaces.

- **Lighting and Shadows:** 
    - Utilizes Fresnel calculations to determine transparency and reflectivity for realistic light interactions.
    - Supports multiple light sources with varying colors and intensities.
    - Includes support for emissive materials that act as their own light sources.

- **Performance Optimization:** 
    - The project employs multi-threading with Rayon to enhance rendering performance, allowing for faster and more efficient image generation.

### Technologies and Libraries Used
The diorama has been developed using various Rust libraries, including:
- `image`: Provides functionality for image processing, allowing textures to be loaded and mapped onto 3D objects in the diorama.
- `minifb`: A lightweight library used to create a window and display the rendered image in real-time, facilitating interaction with the diorama.
- `nalgebra-glm`: A linear algebra library used for 3D vector and matrix operations, crucial for camera manipulation, lighting calculations, and ray tracing.
- `once_cell`: Offers a way to define and initialize global static variables safely and efficiently, used in this project to manage textures and other reusable resources.
- `rand`: Provides random number generation capabilities, which can be utilized for adding variations in lighting or generating random positions/materials in the diorama.
- `rayon`: A parallelism library that significantly improves the performance of the ray tracing process by leveraging multi-threading to render the scene more quickly.

### Installation
To run the diorama, you will need to have Rust and Cargo installed on your system. Follow these steps to get started:

1. Clone this repository:

`git clone https://github.com/lemoonchild/Raytracing.git`

2. Navigate to the game directory:
`cd diorama-mc`

3. Compile and run the project:
  - if you are on Windows, you can use: `PowerShell -ExecutionPolicy Bypass -File .\run.ps1` 
  - Or use `cargo run --release`

### License
This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

We hope you enjoy the diorama as much as I enjoyed creating it!

### Demostration

Video of the diorama: https://youtu.be/nEGaVrgGx0U 

