use crate::object::{Mesh, ObjectType, Sphere};
use glm::Vec3;
use nalgebra_glm as glm;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BvhNode {
    pub min: [f32; 3],
    pub data: u32, // left child index (if internal) or primitive index (if leaf)
    pub max: [f32; 3],
    pub count: u32, // primitive count (if leaf) or 0 (if internal)
}

// Helper struct for building
struct BvhBuildNode {
    min: Vec3,
    max: Vec3,
    left: Option<Box<BvhBuildNode>>,
    right: Option<Box<BvhBuildNode>>,
    primitive_indices: Vec<(ObjectType, usize)>, // Type and Index
}

#[derive(Clone, Copy)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Aabb { min, max }
    }

    pub fn empty() -> Self {
        Aabb {
            min: Vec3::repeat(f32::INFINITY),
            max: Vec3::repeat(f32::NEG_INFINITY),
        }
    }

    pub fn grow(&mut self, point: Vec3) {
        self.min = glm::min2(&self.min, &point);
        self.max = glm::max2(&self.max, &point);
    }

    pub fn grow_aabb(&mut self, other: &Aabb) {
        self.min = glm::min2(&self.min, &other.min);
        self.max = glm::max2(&self.max, &other.max);
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }
}

pub trait Bounded {
    fn aabb(&self) -> Aabb;
}

pub fn build_bvh_flat(spheres: &[Sphere], meshes: &[Mesh]) -> Vec<BvhNode> {
    // 1. Collect all primitives with their AABBs
    let mut primitives: Vec<((ObjectType, usize), Aabb)> = Vec::new();

    primitives.extend(
        spheres
            .iter()
            .enumerate()
            .map(|(i, s)| ((ObjectType::Sphere, i), s.aabb())),
    );

    primitives.extend(
        meshes
            .iter()
            .enumerate()
            .map(|(i, m)| ((ObjectType::Mesh, i), m.aabb())),
    );

    // 2. Build Tree
    let root = build_recursive(&mut primitives);

    // 3. Flatten
    let mut nodes = Vec::new();
    flatten_tree(&root, &mut nodes);

    nodes
}

fn build_recursive(primitives: &mut [((ObjectType, usize), Aabb)]) -> BvhBuildNode {
    // Compute Bounds for this node
    let mut bounds = Aabb::empty();
    for (_, aabb) in primitives.iter() {
        bounds.grow_aabb(aabb);
    }

    if primitives.len() <= 1 {
        return BvhBuildNode {
            min: bounds.min,
            max: bounds.max,
            left: None,
            right: None,
            primitive_indices: primitives.iter().map(|(id, _)| *id).collect(),
        };
    }

    // Split
    let extent = bounds.max - bounds.min;
    let axis = if extent.x > extent.y && extent.x > extent.z {
        0
    } else if extent.y > extent.z {
        1
    } else {
        2
    };

    // Sort primitives based on centroid position along the chosen axis
    primitives.sort_by(|(_, a), (_, b)| {
        let ac = a.center();
        let bc = b.center();
        ac[axis]
            .partial_cmp(&bc[axis])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let split_idx = primitives.len() / 2;

    let (left_prims, right_prims) = primitives.split_at_mut(split_idx);

    let left = build_recursive(left_prims);
    let right = build_recursive(right_prims);

    BvhBuildNode {
        min: bounds.min,
        max: bounds.max,
        left: Some(Box::new(left)),
        right: Some(Box::new(right)),
        primitive_indices: Vec::new(),
    }
}

fn flatten_tree(node: &BvhBuildNode, nodes: &mut Vec<BvhNode>) -> u32 {
    let index = nodes.len() as u32;
    // Push dummy to reserve spot
    nodes.push(BvhNode {
        min: [0.0; 3],
        data: 0,
        max: [0.0; 3],
        count: 0,
    });

    let min: [f32; 3] = node.min.into();
    let max: [f32; 3] = node.max.into();

    if node.left.is_none() && node.right.is_none() {
        // Leaf
        let prim = node.primitive_indices.first();
        let (obj_type, obj_idx) = if let Some(p) = prim {
            *p
        } else {
            (ObjectType::Sphere, 0)
        }; // dummy if empty

        let type_bit = match obj_type {
            ObjectType::Sphere => 0,
            ObjectType::Mesh => 1,
        };

        let data = (type_bit << 31) | (obj_idx as u32);
        let count = node.primitive_indices.len() as u32;

        nodes[index as usize] = BvhNode {
            min,
            max,
            data,
            count,
        };
    } else {
        // Internal
        if let Some(left) = &node.left {
            flatten_tree(left, nodes);
        }
        let right_idx = if let Some(right) = &node.right {
            flatten_tree(right, nodes)
        } else {
            0
        };

        nodes[index as usize] = BvhNode {
            min,
            max,
            data: right_idx,
            count: 0,
        };
    }

    index
}
