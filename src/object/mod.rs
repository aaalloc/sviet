mod sphere;
pub use sphere::Sphere;

mod mesh;
pub use mesh::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct Object {
    pub id: u32,
    pub obj_type: u32,
    pub count: u32,
    pub offset: u32,
}

#[derive(Clone, Debug)]

pub struct ObjectList {
    pub objects: Vec<Object>,
    pub meshes: Vec<Mesh>,
    // hashmap where key is the object id and value is a tuple of start and end index in the mesh vector
    pub object_hashmap: std::collections::HashMap<u32, (u32, u32)>,
    pub counter: u32,
    pub offset_counter: u32,
    pub offset_counter_spheres: u32,
}

impl ObjectList {
    pub fn new() -> Self {
        ObjectList {
            objects: Vec::new(),
            counter: 0,
            offset_counter: 0,
            offset_counter_spheres: 0,
            meshes: Vec::new(),
            object_hashmap: std::collections::HashMap::new(),
        }
    }

    pub fn new_empty_mesh() -> Self {
        ObjectList {
            objects: Vec::new(),
            counter: 0,
            offset_counter: 0,
            offset_counter_spheres: 0,
            meshes: vec![Mesh::empty()],
            object_hashmap: std::collections::HashMap::new(),
        }
    }

    pub fn add(&mut self, obj: Object, meshes: Option<Vec<Mesh>>) {
        match obj.obj_type.into() {
            ObjectType::Sphere => self.offset_counter_spheres += obj.count,
            ObjectType::Mesh => self.offset_counter += obj.count,
        }
        self.objects.push(obj);
        self.counter += 1;
        if let Some(mut mesh) = meshes {
            mesh.iter_mut().for_each(|m| {
                m.material_idx = obj.id;
            });
            mesh.iter().for_each(|m| self.meshes.push(*m));

            self.object_hashmap.insert(
                obj.id,
                (self.offset_counter - obj.count, self.offset_counter),
            );
        }
    }

    pub fn add_sphere(&mut self, count: Option<usize>) {
        self.add(
            Object::new(
                self.counter,
                ObjectType::Sphere,
                count,
                Some(self.offset_counter_spheres),
            ),
            None,
        );
    }

    pub fn add_mesh(&mut self, count: Option<usize>, meshes: Vec<Mesh>) {
        self.add(
            Object::new(
                self.counter,
                ObjectType::Mesh,
                count,
                Some(self.offset_counter),
            ),
            Some(meshes),
        );
    }
}

impl Object {
    pub fn new(id: u32, obj_type: ObjectType, count: Option<usize>, offset: Option<u32>) -> Self {
        Object {
            id,
            obj_type: obj_type as u32,
            count: count.unwrap_or(1) as u32,
            offset: offset.unwrap_or(0),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ObjectType {
    Sphere = 0,
    Mesh = 1,
}

impl From<u32> for ObjectType {
    fn from(item: u32) -> Self {
        match item {
            0 => ObjectType::Sphere,
            1 => ObjectType::Mesh,
            _ => ObjectType::Sphere,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct Light {
    pub id: u32,
    pub light_type: u32,
}

impl Light {
    pub fn new(id: u32, light_type: ObjectType) -> Self {
        Light {
            id,
            light_type: light_type as u32,
        }
    }
}
