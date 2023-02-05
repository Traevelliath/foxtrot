use bevy::prelude::*;
use bevy::render::mesh::{MeshVertexAttributeId, PrimitiveTopology, VertexAttributeValues};

pub trait Vec3Ext {
    #[allow(clippy::wrong_self_convention)] // Because [`Vec3`] is [`Copy`]
    fn is_approx_zero(self) -> bool;
    fn x0z(self) -> Vec3;
    fn collapse_approx_zero(self) -> Vec3;
}
impl Vec3Ext for Vec3 {
    fn is_approx_zero(self) -> bool {
        self.to_array().iter().all(|&x| x.abs() < 1e-5)
    }
    fn x0z(self) -> Vec3 {
        Vec3::new(self.x, 0., self.z)
    }
    fn collapse_approx_zero(self) -> Vec3 {
        let collapse = |x: f32| if x.abs() < 1e-5 { 0. } else { x };
        Vec3::new(collapse(self.x), collapse(self.y), collapse(self.z))
    }
}

pub trait Vec2Ext {
    #[allow(clippy::wrong_self_convention)] // Because [`Vec3`] is [`Copy`]
    fn is_approx_zero(self) -> bool;
    fn x0y(self) -> Vec3;
}
impl Vec2Ext for Vec2 {
    fn is_approx_zero(self) -> bool {
        self.to_array().iter().all(|&x| x.abs() < 1e-5)
    }
    fn x0y(self) -> Vec3 {
        Vec3::new(self.x, 0., self.y)
    }
}

pub trait MeshExt {
    fn transform(&mut self, transform: Transform);
    fn transformed(&self, transform: Transform) -> Mesh;
    fn read_coords_mut(&mut self, id: impl Into<MeshVertexAttributeId>) -> &mut Vec<[f32; 3]>;
    fn search_in_children<'a>(
        parent: Entity,
        children: &'a Query<&Children>,
        meshes: &'a Assets<Mesh>,
        mesh_handles: &'a Query<&Handle<Mesh>>,
    ) -> Vec<(Entity, &'a Mesh)>;
}

impl MeshExt for Mesh {
    fn transform(&mut self, transform: Transform) {
        for coords in self.read_coords_mut(Mesh::ATTRIBUTE_POSITION.clone()) {
            let vec3 = (*coords).into();
            let transformed = transform.transform_point(vec3);
            *coords = transformed.into();
        }
        for normal in self.read_coords_mut(Mesh::ATTRIBUTE_NORMAL.clone()) {
            let vec3 = (*normal).into();
            let transformed = transform.rotation.mul_vec3(vec3);
            *normal = transformed.into();
        }
    }

    fn transformed(&self, transform: Transform) -> Mesh {
        let mut mesh = self.clone();
        mesh.transform(transform);
        mesh
    }

    fn read_coords_mut(&mut self, id: impl Into<MeshVertexAttributeId>) -> &mut Vec<[f32; 3]> {
        match self.attribute_mut(id).unwrap() {
            VertexAttributeValues::Float32x3(values) => values,
            // Guaranteed by Bevy
            _ => unreachable!(),
        }
    }

    fn search_in_children<'a>(
        parent: Entity,
        children_query: &'a Query<&Children>,
        meshes: &'a Assets<Mesh>,
        mesh_handles: &'a Query<&Handle<Mesh>>,
    ) -> Vec<(Entity, &'a Mesh)> {
        if let Ok(children) = children_query.get(parent) {
            let mut result: Vec<_> = children
                .iter()
                .filter_map(|entity| mesh_handles.get(*entity).ok().map(|mesh| (*entity, mesh)))
                .map(|(entity, mesh_handle)| (entity, meshes.get(mesh_handle).unwrap()))
                .map(|(entity, mesh)| {
                    assert_eq!(mesh.primitive_topology(), PrimitiveTopology::TriangleList);
                    (entity, mesh)
                })
                .collect();
            let mut inner_result = children
                .iter()
                .flat_map(|entity| {
                    Self::search_in_children(*entity, children_query, meshes, mesh_handles)
                })
                .collect();
            result.append(&mut inner_result);
            result
        } else {
            Vec::new()
        }
    }
}
