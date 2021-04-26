use std::convert::TryInto;

use bevy::prelude::*;
use bevy::render::mesh::{Indices, VertexAttributeValues};
use bevy::render::pipeline::PrimitiveTopology;
use bevy_rapier3d::physics::ColliderHandleComponent;
use bevy_rapier3d::rapier::dynamics::{MassProperties, RigidBodyBuilder};
use bevy_rapier3d::rapier::geometry::{ColliderBuilder, SharedShape};
use bevy_rapier3d::rapier::math::{Isometry, Point, Vector};

use crate::custom_asset::CustomAsset;

pub struct CalcBoundsPlugin<T>(pub T);

impl<T: crate::util::StateType> Plugin for CalcBoundsPlugin<T> {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_update(self.0.clone()).with_system(calculate_bounds.system()),
        );
    }
}

#[derive(Default)]
pub struct CalcBounds;

#[derive(Default)]
pub struct ColliderProps {
    pub linvel: Vec3,
    pub angvel: Vec3,
}

fn calculate_bounds(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    custom_assets: Res<Assets<CustomAsset>>,
    query: Query<
        (
            &Handle<Mesh>,
            &Transform,
            Entity,
            Option<&MassProperties>,
            Option<&Handle<CustomAsset>>,
            Option<&ColliderProps>,
        ),
        (With<CalcBounds>, Without<ColliderHandleComponent>),
    >,
) {
    for (mesh, transform, entity, mass, custom_asset, collider_props) in query.iter() {
        let mesh = match custom_asset {
            Some(a) => {
                let custom_asset = match custom_assets.get(a) {
                    None => continue,
                    Some(mesh) => mesh,
                };

                let mut parts = vec![];
                for (vertices, indices) in &custom_asset.0 {
                    let vertices: Vec<Point<f32>> = vertices
                        .iter()
                        .copied()
                        .map(|point| Point::from(transform.scale * Vec3::from(point)))
                        .collect();
                    if let Some(convex) = SharedShape::convex_mesh(vertices.clone(), &indices) {
                        parts.push((Isometry::identity(), convex));
                    }
                }
                let mass = mass
                    .cloned()
                    .unwrap_or_else(|| MassProperties::from_compound(0.1, &parts));
                ColliderBuilder::new(SharedShape::compound(parts)).mass_properties(mass)
            }
            None => {
                let mesh = match meshes.get(mesh) {
                    None => continue,
                    Some(mesh) => mesh,
                };

                if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
                    panic!("Mesh primitive isn't triangle");
                }
                let vertices: Vec<Point<f32>> = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                    None => panic!("Mesh does not contain vertex positions"),
                    Some(vertex_values) => match &vertex_values {
                        VertexAttributeValues::Float3(positions) => positions
                            .iter()
                            .map(|coordinates| {
                                Point::from(transform.scale * Vec3::from(*coordinates))
                            })
                            .collect(),
                        _ => panic!("Unexpected vertex types in ATTRIBUTE_POSITION"),
                    },
                };
                let indices: Vec<[u32; 3]> = match mesh.indices().unwrap() {
                    Indices::U32(i) => Some(i),
                    _ => None,
                }
                .unwrap()
                .chunks_exact(3)
                .map(|tri| tri.try_into().unwrap())
                .collect();

                let mass = mass.cloned().unwrap_or_else(|| {
                    MassProperties::from_convex_polyhedron(0.1, &vertices, &indices)
                });
                ColliderBuilder::convex_decomposition(&vertices, &indices).mass_properties(mass);
                ColliderBuilder::trimesh(vertices, indices).mass_properties(mass)
            }
        };
        let mut builder = RigidBodyBuilder::new_dynamic().position(crate::util::nalgebra_pos(
            transform.translation,
            transform.rotation,
        ));
        if let Some(ColliderProps { linvel, angvel }) = collider_props {
            builder = builder
                .linvel(linvel.x, linvel.y, linvel.z)
                .angvel(Vector::from(*angvel))
        }
        commands.entity(entity).insert(mesh).insert(builder);
    }
}
