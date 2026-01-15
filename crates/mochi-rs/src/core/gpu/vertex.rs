use glam::{Vec2, Vec3, Vec4};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub color: Vec4,
    pub tex_coord: Vec2,
}

impl Vertex {
    pub fn new(position: Vec3, color: Vec4, tex_coord: Vec2) -> Self {
        Self {
            position,
            color,
            tex_coord,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}

pub fn vertices_to_floats(vertices: &[Vertex]) -> Vec<f32> {
    let mut floats = Vec::with_capacity(vertices.len() * 9);
    for v in vertices {
        floats.push(v.position.x);
        floats.push(v.position.y);
        floats.push(v.position.z);
        floats.push(v.color.x);
        floats.push(v.color.y);
        floats.push(v.color.z);
        floats.push(v.color.w);
        floats.push(v.tex_coord.x);
        floats.push(v.tex_coord.y);
    }
    floats
}
