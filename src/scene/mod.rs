mod camera;
pub use camera::{Camera, CameraController, GpuCamera};

mod material;
pub use material::{GpuMaterial, Material, Texture};

use crate::object::{
    self, area, center_surface, rotate, scale, translate, Light, Mesh, Object, ObjectList,
    ObjectType, Sphere,
};

#[derive(Clone, Debug)]
pub struct Scene {
    pub materials: Vec<Material>,
    pub spheres: Vec<Sphere>,
    pub lights: Vec<Light>,
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub render_param: RenderParam,
    pub frame_data: FrameData,
    pub object_list: ObjectList,
}

impl PartialEq for Scene {
    fn eq(&self, other: &Self) -> bool {
        self.materials == other.materials
            && self.spheres == other.spheres
            && self.camera == other.camera
            && self.frame_data == other.frame_data
            && self.camera_controller == other.camera_controller
    }
}

impl Scene {
    #[allow(dead_code)]
    pub fn raytracing_scene_oneweek(render_param: RenderParam, frame_data: FrameData) -> Self {
        let mut spheres = Vec::new();
        let mut materials = Vec::new();
        let mut lights = Vec::new();
        let mut object_list = ObjectList::new_empty_mesh();

        let ground_material = Material::Lambertian {
            albedo: Texture::new_from_color(glm::vec3(0.5, 0.5, 0.5)),
        };

        materials.push(ground_material);
        spheres.push(Sphere::new(glm::vec3(0.0, -1000.0, 0.0), 1000.0));

        for (a, b) in (-11..11).flat_map(|a| (-11..11).map(move |b| (a, b))) {
            let choose_mat = rand::random::<f32>();
            let center = glm::vec3(
                a as f32 + 0.9 * rand::random::<f32>(),
                0.2,
                b as f32 + 0.9 * rand::random::<f32>(),
            );

            if (center - glm::vec3(4.0, 0.2, 0.0)).magnitude() > 0.9 {
                let sphere_material = if choose_mat < 0.8 {
                    Material::Lambertian {
                        albedo: Texture::new_from_color(glm::vec3(
                            rand::random::<f32>() * rand::random::<f32>(),
                            rand::random::<f32>() * rand::random::<f32>(),
                            rand::random::<f32>() * rand::random::<f32>(),
                        )),
                    }
                } else if choose_mat < 0.95 {
                    Material::Metal {
                        albedo: Texture::new_from_color(glm::vec3(
                            0.5 * (1.0 + rand::random::<f32>()),
                            0.5 * (1.0 + rand::random::<f32>()),
                            0.5 * (1.0 + rand::random::<f32>()),
                        )),
                        fuzz: rand::random::<f32>() * 0.5,
                    }
                } else {
                    Material::Dialectric { ref_idx: 1.5 }
                };

                materials.push(sphere_material);
                spheres.push(Sphere::new(center, 0.2));
            }
        }

        spheres.push(Sphere::new(glm::vec3(0.0, 1.0, 0.0), 1.0));
        materials.push(Material::Dialectric { ref_idx: 1.5 });

        spheres.push(Sphere::new(glm::vec3(-4.0, 1.0, 0.0), 1.0));
        materials.push(Material::DiffuseLight {
            emit: Texture::new_from_color(glm::vec3(10.0, 10.0, 10.0)),
        });
        lights.push(Light::new(spheres.len() as u32 - 1, ObjectType::Sphere));

        spheres.push(Sphere::new(glm::vec3(4.0, 1.0, 0.0), 1.0));
        materials.push(Material::Metal {
            albedo: Texture::new_from_color(glm::vec3(0.7, 0.6, 0.5)),
            fuzz: 0.0,
        });

        let camera = Camera {
            eye_pos: glm::vec3(-10.5, 2.73, -5.83),
            eye_dir: glm::vec3(0.9086872, -0.15932521, 0.3858796),
            up: glm::vec3(0.0, 1.0, 0.0),
            vfov: 20.0,
            aperture: 0.0,
            focus_distance: 10.0,
        };

        let objects: Vec<Object> = spheres
            .iter()
            .enumerate()
            .map(|(i, _)| Object::new(i as u32, object::ObjectType::Sphere, None, Some(i as u32)))
            .collect();

        object_list.objects = objects;

        Self {
            camera,
            // meshes: vec![Mesh::empty()],
            materials,
            spheres,
            lights,
            render_param,
            frame_data,
            camera_controller: CameraController::new(4.0, 0.4),
            object_list,
        }
    }
    #[allow(dead_code)]
    pub fn cornell_scene_without_suzanne(render_param: RenderParam, frame_data: FrameData) -> Self {
        let mut materials = Vec::new();
        let mut object_list = ObjectList::new();
        let mut spheres = Vec::new();
        let mut lights = Vec::new();

        let red = Material::Lambertian {
            albedo: Texture::new_from_color(glm::vec3(0.65, 0.05, 0.05)),
        };
        let white = Material::Lambertian {
            albedo: Texture::new_from_color(glm::vec3(0.73, 0.73, 0.73)),
        };
        let green = Material::Lambertian {
            albedo: Texture::new_from_color(glm::vec3(0.12, 0.45, 0.15)),
        };
        let light = Material::DiffuseLight {
            emit: Texture::new_from_color(glm::vec3(15.0, 15.0, 15.0)),
        };

        let metal = Material::Metal {
            albedo: Texture::new_from_color(glm::vec3(0.8, 0.85, 0.88)),
            fuzz: 0.0,
        };

        let gold_metal = Material::Metal {
            albedo: Texture::new_from_color(glm::vec3(0.8, 0.6, 0.2)),
            fuzz: 0.4,
        };

        materials.push(white.clone());
        materials.push(green);
        materials.push(red);
        materials.push(white.clone());
        materials.push(white.clone());
        materials.push(light);
        materials.push(white.clone());
        materials.push(metal.clone());
        // materials.push(gold_metal);
        materials.push(Material::Dialectric { ref_idx: 1.5 });

        let mut back_wall = Mesh::quad();
        translate(&mut back_wall, glm::vec3(0.0, 0.0, -1.0));
        object_list.add_mesh(Some(back_wall.len()), back_wall);

        let mut left_wall = Mesh::quad();
        rotate(&mut left_wall, 90., glm::vec3(0.0, 1.0, 0.0));
        translate(&mut left_wall, glm::vec3(-1.0, 0.0, 0.0));
        for v in left_wall.iter_mut() {
            v.normals = [
                glm::vec4(0.5, 0.0, 0.0, 1.0),
                glm::vec4(0.5, 0.0, 0.0, 1.0),
                glm::vec4(0.5, 0.0, 0.0, 1.0),
            ]
        }
        object_list.add_mesh(Some(left_wall.len()), left_wall);

        let mut right_wall: Vec<Mesh> = Mesh::quad();
        rotate(&mut right_wall, 90., glm::vec3(0.0, 1.0, 0.0));
        translate(&mut right_wall, glm::vec3(1.0, 0.0, 0.0));
        for v in right_wall.iter_mut() {
            v.normals = [
                glm::vec4(-0.5, 0.0, 0.0, 1.0),
                glm::vec4(-0.5, 0.0, 0.0, 1.0),
                glm::vec4(-0.5, 0.0, 0.0, 1.0),
            ]
        }
        object_list.add_mesh(Some(right_wall.len()), right_wall);

        let mut ceiling = Mesh::quad();
        rotate(&mut ceiling, 90., glm::vec3(1.0, 0.0, 0.0));
        translate(&mut ceiling, glm::vec3(0.0, 1.0, 0.0));
        for v in ceiling.iter_mut() {
            v.normals = [
                glm::vec4(0.0, -0.5, 0.0, 1.0),
                glm::vec4(0.0, -0.5, 0.0, 1.0),
                glm::vec4(0.0, -0.5, 0.0, 1.0),
            ]
        }
        object_list.add_mesh(Some(ceiling.len()), ceiling);

        let mut floor = Mesh::quad();
        rotate(&mut floor, 90., glm::vec3(1.0, 0.0, 0.0));
        translate(&mut floor, glm::vec3(0.0, -1.0, 0.0));
        for v in floor.iter_mut() {
            v.normals = [
                glm::vec4(0.0, 0.5, 0.0, 1.0),
                glm::vec4(0.0, 0.5, 0.0, 1.0),
                glm::vec4(0.0, 0.5, 0.0, 1.0),
            ]
        }
        object_list.add_mesh(Some(floor.len()), floor);

        let mut ceiling_light = Mesh::quad();
        rotate(&mut ceiling_light, 90., glm::vec3(1.0, 0.0, 0.0));
        translate(&mut ceiling_light, glm::vec3(0.0, 0.99, 0.));
        scale(&mut ceiling_light, glm::vec3(0.20, 1.0, 0.2));
        for v in ceiling_light.iter_mut() {
            v.normals = [
                glm::vec4(0.0, -0.5, 0.0, 1.0),
                glm::vec4(0.0, -0.5, 0.0, 1.0),
                glm::vec4(0.0, -0.5, 0.0, 1.0),
            ]
        }
        object_list.add_mesh(Some(ceiling_light.len()), ceiling_light);
        lights.push(Light::new(5, ObjectType::Mesh));

        let mut box1 = Mesh::cube();
        scale(&mut box1, glm::vec3(0.3, 0.3, 0.3));
        rotate(&mut box1, 70., glm::vec3(0.0, 1.0, 0.0));
        translate(&mut box1, glm::vec3(0.3, -0.699, 0.3));
        object_list.add_mesh(Some(box1.len()), box1);

        let mut rectangle_box = Mesh::cube();
        scale(&mut rectangle_box, glm::vec3(0.3, 0.6, 0.3));
        rotate(&mut rectangle_box, 15., glm::vec3(0.0, 1.0, 0.0));
        translate(&mut rectangle_box, glm::vec3(-0.3, -0.399, -0.35));
        object_list.add_mesh(Some(rectangle_box.len()), rectangle_box);

        spheres.push(Sphere::new(glm::vec3(-0.5, -0.8, 0.3), 0.2));
        object_list.add_sphere(None);

        let camera = Camera {
            eye_pos: glm::vec3(0.0, 0.0, 5.),
            eye_dir: glm::vec3(0.0, 0.0, -1.0),
            up: glm::vec3(0.0, 1.0, 0.0),
            vfov: 30.0,
            aperture: 0.0,
            focus_distance: 10.0,
        };

        Self {
            camera,
            materials,
            spheres,
            lights,
            render_param,
            frame_data,
            camera_controller: CameraController::new(4.0, 0.4),
            object_list,
        }
    }

    pub fn cornell_scene(render_param: RenderParam, frame_data: FrameData) -> Self {
        let mut materials = Vec::new();
        let mut object_list = ObjectList::new();
        let mut spheres = Vec::new();
        let mut lights = Vec::new();

        let red = Material::Lambertian {
            albedo: Texture::new_from_color(glm::vec3(0.65, 0.05, 0.05)),
        };
        let white = Material::Lambertian {
            albedo: Texture::new_from_color(glm::vec3(0.73, 0.73, 0.73)),
        };
        let green = Material::Lambertian {
            albedo: Texture::new_from_color(glm::vec3(0.12, 0.45, 0.15)),
        };
        let light = Material::DiffuseLight {
            emit: Texture::new_from_color(glm::vec3(15.0, 15.0, 15.0)),
        };

        let metal = Material::Metal {
            albedo: Texture::new_from_color(glm::vec3(0.8, 0.85, 0.88)),
            fuzz: 0.0,
        };

        let gold_metal = Material::Metal {
            albedo: Texture::new_from_color(glm::vec3(0.8, 0.6, 0.2)),
            fuzz: 0.4,
        };

        materials.push(white.clone());
        materials.push(green);
        materials.push(red);
        materials.push(white.clone());
        materials.push(white.clone());
        materials.push(light);
        materials.push(white.clone());
        // materials.push(white.clone());
        materials.push(metal.clone());
        materials.push(gold_metal);
        materials.push(Material::Dialectric { ref_idx: 1.5 });

        let mut back_wall = Mesh::quad();
        translate(&mut back_wall, glm::vec3(0.0, 0.0, -1.0));
        object_list.add_mesh(Some(back_wall.len()), back_wall);

        let mut left_wall = Mesh::quad();
        rotate(&mut left_wall, 90., glm::vec3(0.0, 1.0, 0.0));
        translate(&mut left_wall, glm::vec3(-1.0, 0.0, 0.0));
        for v in left_wall.iter_mut() {
            v.normals = [
                glm::vec4(0.5, 0.0, 0.0, 1.0),
                glm::vec4(0.5, 0.0, 0.0, 1.0),
                glm::vec4(0.5, 0.0, 0.0, 1.0),
            ]
        }
        object_list.add_mesh(Some(left_wall.len()), left_wall);

        let mut right_wall: Vec<Mesh> = Mesh::quad();
        rotate(&mut right_wall, 90., glm::vec3(0.0, 1.0, 0.0));
        translate(&mut right_wall, glm::vec3(1.0, 0.0, 0.0));
        for v in right_wall.iter_mut() {
            v.normals = [
                glm::vec4(-0.5, 0.0, 0.0, 1.0),
                glm::vec4(-0.5, 0.0, 0.0, 1.0),
                glm::vec4(-0.5, 0.0, 0.0, 1.0),
            ]
        }
        object_list.add_mesh(Some(right_wall.len()), right_wall);

        let mut ceiling = Mesh::quad();
        rotate(&mut ceiling, 90., glm::vec3(1.0, 0.0, 0.0));
        translate(&mut ceiling, glm::vec3(0.0, 1.0, 0.0));
        for v in ceiling.iter_mut() {
            v.normals = [
                glm::vec4(0.0, -0.5, 0.0, 1.0),
                glm::vec4(0.0, -0.5, 0.0, 1.0),
                glm::vec4(0.0, -0.5, 0.0, 1.0),
            ]
        }
        object_list.add_mesh(Some(ceiling.len()), ceiling);

        let mut floor = Mesh::quad();
        rotate(&mut floor, 90., glm::vec3(1.0, 0.0, 0.0));
        translate(&mut floor, glm::vec3(0.0, -1.0, 0.0));
        for v in floor.iter_mut() {
            v.normals = [
                glm::vec4(0.0, 0.5, 0.0, 1.0),
                glm::vec4(0.0, 0.5, 0.0, 1.0),
                glm::vec4(0.0, 0.5, 0.0, 1.0),
            ]
        }
        object_list.add_mesh(Some(floor.len()), floor);

        let mut ceiling_light = Mesh::quad();
        rotate(&mut ceiling_light, 90., glm::vec3(1.0, 0.0, 0.0));
        translate(&mut ceiling_light, glm::vec3(0.0, 0.99, 0.));
        scale(&mut ceiling_light, glm::vec3(0.20, 1.0, 0.2));
        for v in ceiling_light.iter_mut() {
            v.normals = [
                glm::vec4(0.0, -0.5, 0.0, 1.0),
                glm::vec4(0.0, -0.5, 0.0, 1.0),
                glm::vec4(0.0, -0.5, 0.0, 1.0),
            ]
        }
        object_list.add_mesh(Some(ceiling_light.len()), ceiling_light);
        lights.push(Light::new(5, ObjectType::Mesh));

        let mut box1 = Mesh::cube();
        scale(&mut box1, glm::vec3(0.3, 0.3, 0.3));
        rotate(&mut box1, 70., glm::vec3(0.0, 1.0, 0.0));
        translate(&mut box1, glm::vec3(0.3, -0.699, 0.3));
        object_list.add_mesh(Some(box1.len()), box1);

        let mut rectangle_box = Mesh::cube();
        scale(&mut rectangle_box, glm::vec3(0.3, 0.6, 0.3));
        rotate(&mut rectangle_box, 15., glm::vec3(0.0, 1.0, 0.0));
        translate(&mut rectangle_box, glm::vec3(-0.3, -0.399, -0.35));

        object_list.add_mesh(Some(rectangle_box.len()), rectangle_box);

        let path_str = "assets/mesh/suzanne.obj";
        let options = tobj::LoadOptions {
            triangulate: true,
            ..Default::default()
        };
        println!("Current path: {:?}", std::env::current_dir().unwrap());

        let s = tobj::load_obj(path_str, &options).unwrap().0[0].clone();

        let mut sdsd = Mesh::from_tobj(s);
        scale(&mut sdsd, glm::vec3(0.2, 0.2, 0.2));
        rotate(&mut sdsd, -35.0, glm::vec3(1.0, 0.0, 0.0));
        rotate(&mut sdsd, -30.0, glm::vec3(0.0, 1.0, 0.0));
        translate(&mut sdsd, glm::vec3(0.3, -0.30, 0.3));
        object_list.add_mesh(Some(sdsd.len()), sdsd);

        spheres.push(Sphere::new(glm::vec3(-0.5, -0.8, 0.3), 0.2));
        object_list.add_sphere(None);
        let camera = Camera {
            eye_pos: glm::vec3(0.0, 0.0, 5.),
            eye_dir: glm::vec3(0.0, 0.0, -1.0),
            up: glm::vec3(0.0, 1.0, 0.0),
            vfov: 30.0,
            aperture: 0.0,
            focus_distance: 10.0,
        };

        Self {
            camera,
            materials,
            spheres,
            lights,
            render_param,
            frame_data,
            camera_controller: CameraController::new(4.0, 0.4),
            object_list,
        }
    }

    pub fn teapot_scene(render_param: RenderParam, frame_data: FrameData) -> Self {
        let mut materials = Vec::new();
        let mut object_list = ObjectList::new();

        let ground_material = Material::Lambertian {
            albedo: Texture::new_from_color(glm::vec3(0.5, 0.5, 0.5)),
        };

        materials.push(ground_material);

        let path_str = "teapot.obj";
        let options = tobj::LoadOptions {
            triangulate: true,
            ..Default::default()
        };
        println!("Current path: {:?}", std::env::current_dir().unwrap());

        let s = tobj::load_obj(path_str, &options).unwrap().0[0].clone();

        let meshes = Mesh::from_tobj(s);
        object_list.add_mesh(Some(meshes.len()), meshes);

        let camera = Camera {
            eye_pos: glm::vec3(0.0, 0.0, 6.6),
            eye_dir: glm::vec3(0.0, 0.0, -1.0),
            up: glm::vec3(0.0, 1.0, 0.0),
            vfov: 20.0,
            aperture: 0.0,
            focus_distance: 1.0,
        };

        Self {
            camera,
            materials,
            spheres: vec![Sphere::empty()],
            lights: vec![Light::empty()],
            render_param,
            frame_data,
            camera_controller: CameraController::new(4.0, 0.4),
            object_list,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct RenderParam {
    pub samples_max_per_pixel: u32,
    pub samples_per_pixel: u32,
    pub total_samples: u32,
    pub clear_samples: u32,
    pub max_depth: u32,
}

impl RenderParam {
    pub fn update(&mut self) {
        if self.total_samples == 0 {
            self.total_samples += self.samples_per_pixel;
            self.clear_samples = 1;
        } else if self.total_samples <= self.samples_max_per_pixel {
            self.total_samples += self.samples_per_pixel;
            self.clear_samples = 0;
        } else {
            self.samples_per_pixel = 0;
            self.clear_samples = 0;
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FrameData {
    pub width: u32,
    pub height: u32,
    pub index: u32,
}

impl PartialEq for FrameData {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width && self.height == other.height
    }
}
