use crate::la::{Color, Point3, Ray, Vec3};
use std::rc::Rc;
use std::vec::Vec;

pub struct HitRecord {
    pub t: f64,
    pub p: Point3,
    pub normal: Vec3,
    pub mat: Rc<dyn Material>,
    pub front_face: bool,
}

impl HitRecord {
    pub fn new(
        r: &Ray,
        t: f64,
        p: Point3,
        mat: Rc<dyn Material>,
        outward_normal: Vec3,
    ) -> HitRecord {
        let front_face = Vec3::dot(&r.direction(), &outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };

        HitRecord {
            t,
            p,
            normal,
            mat,
            front_face,
        }
    }
}

pub trait Hittable {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}

pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
    pub mat: Rc<dyn Material>,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, mat: Rc<dyn Material>) -> Sphere {
        Sphere {
            center,
            radius,
            mat,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = r.origin() - self.center;
        let a = Vec3::dot(&r.direction(), &r.direction());
        let hb = Vec3::dot(&oc, &r.direction());
        let c = Vec3::dot(&oc, &oc) - self.radius.powi(2);
        let discriminant = hb.powi(2) - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();
        let mut t = (-hb - sqrtd) / a;
        // Check for closest hit
        if (t < t_min || t_max < t) {
            t = (-hb + sqrtd) / a;
            if (t < t_min || t_max < t) {
                return None;
            }
        }

        let p = r.at(t);
        let outward_normal = (p - self.center) / self.radius;
        let mut rec = HitRecord::new(r, t, p, self.mat.clone(), outward_normal);

        return Some(rec);
    }
}

#[derive(Clone)]
pub struct HittableList {
    objects: Vec<Rc<dyn Hittable>>,
}

impl HittableList {
    pub fn new() -> HittableList {
        HittableList {
            objects: Vec::<Rc<dyn Hittable>>::new(),
        }
    }

    pub fn add(&mut self, object: Rc<dyn Hittable>) -> () {
        self.objects.push(object);
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut hit_anything = false;
        let mut closest = t_max;

        let mut closest_rec: Option<HitRecord> = None;
        for object in &self.objects {
            match object.hit(r, t_min, closest) {
                Some(rec) => {
                    closest = rec.t;
                    closest_rec = Some(rec);
                }
                None => (),
            }
        }

        return closest_rec;
    }
}

pub trait Material {
    fn scatter(&self, r: &Ray, rec: &HitRecord) -> Option<(Ray, Color)>;
}

pub struct Lambertian {
    albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Lambertian {
        Lambertian { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, r: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        let mut scatter_dir = rec.normal + Vec3::rand_unit_vector();

        if (scatter_dir.is_near_zero()) {
            scatter_dir = rec.normal;
        }

        let r_scattered = Ray::new(rec.p, scatter_dir);
        let attenuation = self.albedo;
        return Some((r_scattered, attenuation));
    }
}

pub struct Metal {
    albedo: Color,
    roughness: f64,
}

impl Metal {
    pub fn new(albedo: Color, roughness: f64) -> Metal {
        Metal {
            albedo,
            roughness: roughness.clamp(0.0, 1.0),
        }
    }
}

impl Material for Metal {
    fn scatter(&self, r: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        let reflected_dir = Vec3::reflect(&r.direction(), &rec.normal);

        let r_scattered = Ray::new(
            rec.p,
            reflected_dir + self.roughness * Vec3::rand_unit_vector(),
        );
        let attenuation = self.albedo;
        if (Vec3::dot(&r_scattered.direction(), &rec.normal) > 0.0) {
            return Some((r_scattered, attenuation));
        } else {
            return None;
        }
    }
}

pub struct Dielectric {
    pub ior: f64,
}

impl Dielectric {
    pub fn new(ior: f64) -> Self {
        Dielectric { ior }
    }

    pub fn reflectance(cosine: f64, ior: f64) -> f64 {
        // Schlick's approximation
        let r0 = ((1.0 - ior) / (1.0 + ior)).powi(2);
        return r0 + (1.0 - r0) * (1.0 - cosine).powi(5);
    }
}

impl Material for Dielectric {
    fn scatter(&self, r: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        let attenuation = Color::new(1.0, 1.0, 1.0);
        let refraction_ratio = if rec.front_face {
            1.0 / self.ior
        } else {
            self.ior
        };

        let cos_theta = Vec3::dot(&(-r.direction()), &rec.normal).min(1.0);
        let sin_theta = f64::sqrt(1.0 - cos_theta.powi(2));

        let total_internal_reflection = refraction_ratio * sin_theta > 1.0;

        let r_direction = if (total_internal_reflection
            || (Dielectric::reflectance(cos_theta, refraction_ratio)) > rand::random::<f64>())
        {
            r.direction().reflect(&rec.normal)
        } else {
            r.direction().refract(&rec.normal, refraction_ratio)
        };

        let r_scattered = Ray::new(rec.p, r_direction);

        return Some((r_scattered, attenuation));
    }
}
