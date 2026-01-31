use glm::Vec3;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
// TODO: For the moment, vec4 for padding, include manually
pub struct Mesh {
    pub vertices: [glm::Vec4; 3],
    pub normals: [glm::Vec4; 3],
}

impl Mesh {
    #[allow(dead_code)]
    pub fn square() -> Vec<Mesh> {
        vec![
            Mesh {
                vertices: [
                    glm::vec4(-0.5, -0.5, 0.0, 1.0),
                    glm::vec4(0.5, -0.5, 0.0, 1.0),
                    glm::vec4(-0.5, 0.0, 0.0, 1.0),
                ],
                normals: [
                    glm::vec4(0.0, 0.0, 0.0, 1.0),
                    glm::vec4(0.0, 0.0, 0.0, 1.0),
                    glm::vec4(0.0, 0.0, 0.0, 1.0),
                ],
            },
            Mesh {
                vertices: [
                    glm::vec4(0.5, 0.0, 0.0, 1.0),
                    glm::vec4(0.5, -0.5, 0.0, 1.0),
                    glm::vec4(-0.5, 0.0, 0.0, 1.0),
                ],
                normals: [
                    glm::vec4(0.0, 0.0, 0.0, 1.0),
                    glm::vec4(0.0, 0.0, 0.0, 1.0),
                    glm::vec4(0.0, 0.0, 0.0, 1.0),
                ],
            },
        ]
    }

    pub fn quad() -> Vec<Mesh> {
        vec![
            Mesh {
                vertices: [
                    glm::vec4(-1.0, -1.0, 0.0, 1.0),
                    glm::vec4(1.0, -1.0, 0.0, 1.0),
                    glm::vec4(-1.0, 1.0, 0.0, 1.0),
                ],
                normals: [
                    glm::vec4(0.0, 0.0, 0.5, 1.0),
                    glm::vec4(0.0, 0.0, 0.5, 1.0),
                    glm::vec4(0.0, 0.0, 0.5, 1.0),
                ],
            },
            Mesh {
                vertices: [
                    glm::vec4(1.0, 1.0, 0.0, 1.0),
                    glm::vec4(1.0, -1.0, 0.0, 1.0),
                    glm::vec4(-1.0, 1.0, 0.0, 1.0),
                ],
                normals: [
                    glm::vec4(0.0, 0.0, 0.5, 1.0),
                    glm::vec4(0.0, 0.0, 0.5, 1.0),
                    glm::vec4(0.0, 0.0, 0.5, 1.0),
                ],
            },
        ]
    }

    pub fn empty() -> Self {
        Self {
            vertices: [glm::vec4(0.0, 0.0, 0.0, 0.0); 3],
            normals: [glm::vec4(0.0, 0.0, 0.0, 0.0); 3],
        }
    }

    pub fn cube() -> Vec<Mesh> {
        let mut meshes = vec![];
        // Front
        let mut front = Mesh::quad();
        translate(&mut front, glm::vec3(0.0, 0.0, 1.0));
        for v in front.iter_mut() {
            v.normals = [
                glm::vec4(0.0, 0.0, 1.0, 1.0),
                glm::vec4(0.0, 0.0, 1.0, 1.0),
                glm::vec4(0.0, 0.0, 1.0, 1.0),
            ]
        }
        meshes.append(&mut front);

        // Back
        let mut back = Mesh::quad();
        rotate(&mut back, 180.0, glm::vec3(0.0, 1.0, 0.0));
        translate(&mut back, glm::vec3(0.0, 0.0, -1.0));
        for v in back.iter_mut() {
            v.normals = [
                glm::vec4(0.0, 0.0, -1.0, 1.0),
                glm::vec4(0.0, 0.0, -1.0, 1.0),
                glm::vec4(0.0, 0.0, -1.0, 1.0),
            ]
        }
        meshes.append(&mut back);

        // Top
        let mut top = Mesh::quad();
        rotate(&mut top, 90.0, glm::vec3(1.0, 0.0, 0.0));
        translate(&mut top, glm::vec3(0.0, 1.0, 0.0));
        for v in top.iter_mut() {
            v.normals = [
                glm::vec4(0.0, 1.0, 0.0, 1.0),
                glm::vec4(0.0, 1.0, 0.0, 1.0),
                glm::vec4(0.0, 1.0, 0.0, 1.0),
            ]
        }
        meshes.append(&mut top);

        // Bottom
        let mut bottom = Mesh::quad();
        rotate(&mut bottom, -90.0, glm::vec3(1.0, 0.0, 0.0));
        translate(&mut bottom, glm::vec3(0.0, -1.0, 0.0));
        for v in bottom.iter_mut() {
            v.normals = [
                glm::vec4(0.0, -1.0, 0.0, 1.0),
                glm::vec4(0.0, -1.0, 0.0, 1.0),
                glm::vec4(0.0, -1.0, 0.0, 1.0),
            ]
        }
        meshes.append(&mut bottom);

        // Right
        let mut right = Mesh::quad();
        rotate(&mut right, 90.0, glm::vec3(0.0, 1.0, 0.0));
        translate(&mut right, glm::vec3(1.0, 0.0, 0.0));
        for v in right.iter_mut() {
            v.normals = [
                glm::vec4(1.0, 0.0, 0.0, 1.0),
                glm::vec4(1.0, 0.0, 0.0, 1.0),
                glm::vec4(1.0, 0.0, 0.0, 1.0),
            ]
        }
        meshes.append(&mut right);

        // Left
        let mut left = Mesh::quad();
        rotate(&mut left, -90.0, glm::vec3(0.0, 1.0, 0.0));
        translate(&mut left, glm::vec3(-1.0, 0.0, 0.0));
        for v in left.iter_mut() {
            v.normals = [
                glm::vec4(-1.0, 0.0, 0.0, 1.0),
                glm::vec4(-1.0, 0.0, 0.0, 1.0),
                glm::vec4(-1.0, 0.0, 0.0, 1.0),
            ]
        }
        meshes.append(&mut left);

        meshes
    }

    #[allow(dead_code)]
    pub fn from_tobj(tobj: tobj::Model) -> Vec<Mesh> {
        let mesh = &tobj.mesh;
        println!("Positions: {:?}", mesh.positions.len());
        let vertices = mesh
            .positions
            .chunks(3)
            .map(|c| glm::vec4(c[0], c[1], c[2], 0.0))
            .collect::<Vec<_>>();

        let normals = mesh
            .normals
            .chunks(3)
            .map(|c| glm::vec4(c[0], c[1], c[2], 0.0))
            .collect::<Vec<_>>();

        let indices = mesh.indices.chunks(3).map(|c| Mesh {
            vertices: [
                vertices[c[0] as usize],
                vertices[c[1] as usize],
                vertices[c[2] as usize],
            ],
            normals: [
                normals[c[0] as usize],
                normals[c[1] as usize],
                normals[c[2] as usize],
            ],
        });
        indices.collect()
    }
}

pub fn center_surface(meshes: &Vec<Mesh>) -> Vec3 {
    let mut min = glm::vec3(f32::MAX, f32::MAX, f32::MAX);
    let mut max = glm::vec3(f32::MIN, f32::MIN, f32::MIN);
    for mesh in meshes.iter() {
        for vertex in mesh.vertices.iter() {
            min.x = min.x.min(vertex.x);
            min.y = min.y.min(vertex.y);
            min.z = min.z.min(vertex.z);
            max.x = max.x.max(vertex.x);
            max.y = max.y.max(vertex.y);
            max.z = max.z.max(vertex.z);
        }
    }
    let center = (min + max) / 2.0;
    center
}

pub fn area(meshes: &Vec<Mesh>) -> f32 {
    let mut area = 0.0;
    for mesh in meshes.iter() {
        let a = glm::vec3(mesh.vertices[0].x, mesh.vertices[0].y, mesh.vertices[0].z);
        let b = glm::vec3(mesh.vertices[1].x, mesh.vertices[1].y, mesh.vertices[1].z);
        let c = glm::vec3(mesh.vertices[2].x, mesh.vertices[2].y, mesh.vertices[2].z);
        let ab = b - a;
        let ac = c - a;
        let cross = glm::cross(&ab, &ac);
        area += glm::length(&cross) / 2.0;
    }
    area
}

pub fn rotate(meshes: &mut Vec<Mesh>, angle: f32, axis: glm::Vec3) {
    // degree to radian
    let angle = angle.to_radians();
    let rotation = glm::quat_angle_axis(angle, &axis);
    for mesh in meshes.iter_mut() {
        for vertex in mesh.vertices.iter_mut() {
            let position = glm::vec3(vertex.x, vertex.y, vertex.z);
            let rotated = glm::quat_rotate_vec3(&rotation, &position);
            vertex.x = rotated.x;
            vertex.y = rotated.y;
            vertex.z = rotated.z;
        }
        for normal in mesh.normals.iter_mut() {
            let position = glm::vec3(normal.x, normal.y, normal.z);
            let rotated = glm::quat_rotate_vec3(&rotation, &position);
            normal.x = rotated.x;
            normal.y = rotated.y;
            normal.z = rotated.z;
        }
    }
}

pub fn translate(meshes: &mut Vec<Mesh>, translation: glm::Vec3) {
    for mesh in meshes.iter_mut() {
        for vertex in mesh.vertices.iter_mut() {
            vertex.x += translation.x;
            vertex.y += translation.y;
            vertex.z += translation.z;
        }
    }
}

pub fn position(meshes: &[Mesh]) -> Vec3 {
    let mut position = glm::vec3(0.0, 0.0, 0.0);
    for mesh in meshes.iter() {
        for vertex in mesh.vertices.iter() {
            position.x += vertex.x;
            position.y += vertex.y;
            position.z += vertex.z;
        }
    }
    let length = meshes.len() as f32 * 3.0;
    position.x /= length;
    position.y /= length;
    position.z /= length;
    position
}

pub fn scale(meshes: &mut Vec<Mesh>, scale: glm::Vec3) {
    for mesh in meshes.iter_mut() {
        for vertex in mesh.vertices.iter_mut() {
            vertex.x *= scale.x;
            vertex.y *= scale.y;
            vertex.z *= scale.z;
        }
        for normal in mesh.normals.iter_mut() {
            normal.x *= scale.x;
            normal.y *= scale.y;
            normal.z *= scale.z;
        }
    }
}
