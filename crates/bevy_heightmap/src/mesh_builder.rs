use bevy::{
    asset::RenderAssetUsages,
    math::{UVec2, Vec2},
    mesh::{Indices, Mesh, PrimitiveTopology},
};

use crate::HeightMap;

/// Utility struct for building a mesh.
#[derive(Default)]
pub struct MeshBuilder {
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

impl MeshBuilder {
    pub fn position_to_uv(p: &[f32; 3]) -> [f32; 2] {
        [0.5 + p[0], 0.5 - p[1]]
    }

    pub fn index(x: u32, y: u32, w: u32) -> u32 {
        x + y * w
    }

    pub fn quad_indices(x: u32, y: u32, w: u32) -> [u32; 6] {
        [
            // Bottom triangle
            Self::index(x, y, w),
            Self::index(x + 1, y, w),
            Self::index(x + 1, y + 1, w),
            // Top triangle
            Self::index(x, y, w),
            Self::index(x + 1, y + 1, w),
            Self::index(x, y + 1, w),
        ]
    }
    /// Compute a grid mesh of quads according to size.
    pub fn grid(size: UVec2) -> Self {
        let bounds = size - UVec2::ONE;
        let num_points = size.x as usize * size.y as usize;
        let num_quads = bounds.x as usize * bounds.y as usize;
        let mut builder = Self {
            positions: Vec::with_capacity(num_points),
            uvs: Vec::with_capacity(num_points),
            normals: Vec::with_capacity(num_points),
            indices: Vec::with_capacity(num_quads * 6),
        };
        for y in 0..size.y {
            for x in 0..size.x {
                let xy = UVec2::new(x, y).as_vec2() / bounds.as_vec2() - Vec2::splat(0.5);
                builder.positions.push(xy.extend(0.0).to_array());
            }
        }
        for y in 0..bounds.y {
            for x in 0..bounds.x {
                builder.indices.extend(Self::quad_indices(x, y, size.x));
            }
        }
        for p in builder.positions.iter_mut() {
            builder.uvs.push(Self::position_to_uv(p));
        }
        builder
    }

    /// Updates z positions to use the heightmap.
    pub fn update_z_positions<H: HeightMap>(&mut self, heightmap: &H) {
        for p in self.positions.iter_mut() {
            p[2] = heightmap.h(Vec2::new(p[0], p[1]));
        }
    }

    /// Produce a mesh from the accumulated attributes.
    pub fn build(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
        .with_inserted_indices(Indices::U32(self.indices))
        .with_computed_area_weighted_normals()
    }
}
