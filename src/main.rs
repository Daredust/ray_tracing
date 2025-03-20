#![allow(unused)]

use rand::Rng;
use std::f64::consts::PI;
use std::f64::INFINITY;
use std::rc::Rc;

use image::{ImageBuffer, Rgb, RgbImage};

mod la;
use la::*;

mod objects;
use objects::*;

mod camera;
use camera::*;

fn ray_color(r: &Ray, world: &HittableList, depth: u32) -> Color {
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    match world.hit(r, 0.001, INFINITY) {
        Some(rec) => match rec.mat.scatter(r, &rec) {
            Some((r_scattered, attenuation)) => {
                return attenuation * ray_color(&r_scattered, world, depth - 1);
            }
            None => {
                return Color::new(0.0, 0.0, 0.0);
            }
        },
        None => {
            let t = 0.5 * (r.direction().y() + 1.0);
            return (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0);
        }
    }
}

fn main() {
    let output_path = r"render.png";

    let aspect_ratio = 16.0 / 9.0;
    let image_width = 1920;
    let image_height = (image_width as f64 / aspect_ratio) as u32;
    let samples_per_pixel = 500;
    let max_depth = 50;

    let test = Color::new(0.8, 0.5, 0.2);
    dbg!(test.as_u8_color(1));

    // World
    let r = f64::cos((PI / 4.0));
    let mut world = HittableList::new();

    let ground_sphere_pos = Point3::new(0.0, -100.5, -1.0);
    let center_sphere_pos = Point3::new(0.0, 0.0, -1.0);
    let left_sphere_pos = Point3::new(-1.1, 0.0, -1.0);
    let right_sphere_pos = Point3::new(1.1, 0.0, -1.0);
    
    let ground_sphere_mat = Rc::new(Metal::new(Color::new(0.8, 0.8, 0.8), 1.0));
    let center_sphere_mat = Rc::new(Lambertian::new(Color::new(0.0, 1.0, 1.0)));
    let left_sphere_mat = Rc::new(Dielectric::new(1.5));
    let right_sphere_mat = Rc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.0));
    
    world.add(Rc::new(Sphere::new(
        ground_sphere_pos,
        100.0,
        ground_sphere_mat.clone(),
    )));
    world.add(Rc::new(Sphere::new(
        center_sphere_pos,
        0.5,
        center_sphere_mat.clone(),
    )));
    world.add(Rc::new(Sphere::new(
        left_sphere_pos,
        0.5,
        left_sphere_mat.clone(),
    )));
    // world.add(Rc::new(Sphere::new(
    //     left_sphere_pos,
    //     -0.4999,
    //     left_sphere_mat.clone(),
    // )));
    world.add(Rc::new(Sphere::new(
        right_sphere_pos,
        0.5,
        right_sphere_mat.clone(),
    )));    

    // Camera
    let look_from = Point3::new(-3.0, 1.0, 2.0);
    let look_at = Point3::new(0.0, 0.0, -1.0);
    let up = Vec3::new(0.0, 1.0, 0.0);
    
    let focus_dist = (look_from - look_at).length();
    
    let camera = Camera::new(
        &look_from,
        &look_at,
        &up,
        30.0,
        aspect_ratio,
        0.1, 
        focus_dist,
    );

    // Render
    let mut image: RgbImage = ImageBuffer::new(image_width, image_height);

    for j in (0..image_height).rev() {
        println!("Scanlines remaining: {}", j + 1);
        for i in 0..image_width {
            let mut pixel_color = Color::new(0.0, 0.0, 0.0);
            let mut rng = rand::thread_rng();
            for s in 0..samples_per_pixel {
                let u = (i as f64 + rng.gen_range(0.0..1.0)) / (image_width - 1) as f64;
                let v = (j as f64 + rng.gen_range(0.0..1.0)) / (image_height - 1) as f64;
                let r = camera.get_ray(u, v);

                pixel_color += ray_color(&r, &world, max_depth);
            }
            *image.get_pixel_mut(i, image_height - j - 1) =
                Rgb(pixel_color.as_u8_color(samples_per_pixel));
        }
    }

    image.save(output_path).unwrap();
}
