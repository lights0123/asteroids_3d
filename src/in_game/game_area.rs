use std::collections::HashMap;

use bevy::prelude::*;
use bevy_rapier3d::physics::InteractionPairFilters;
use bevy_rapier3d::rapier::dynamics::RigidBodyBuilder;
use bevy_rapier3d::rapier::geometry::ColliderBuilder;

use crate::physics::OneWayPlatformHook;

pub struct GameAreaPlugin<T>(pub T);

impl<T: crate::util::StateType> Plugin for GameAreaPlugin<T> {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(SystemSet::on_enter(self.0.clone()).with_system(spawn.system()));
    }
}

#[derive(Copy, Clone)]
enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    fn set_translation(self, mut trans: Vec3, value: f32) -> Vec3 {
        match self {
            Axis::X => trans.x = value,
            Axis::Y => trans.y = value,
            Axis::Z => trans.z = value,
        }
        trans
    }
    fn get_sizes(major: Self, minor: Self, x: f32, y: f32, z: f32) -> Option<(f32, f32, f32)> {
        use self::Axis::{X, Y, Z};
        match (major, minor) {
            (X, Y) => Some((x, y, z)),
            (X, Z) => Some((x, z, y)),
            (Y, X) => Some((y, x, z)),
            (Y, Z) => Some((y, z, x)),
            (Z, X) => Some((z, x, y)),
            (Z, Y) => Some((z, y, x)),
            _ => None,
        }
    }
    fn get_tertiary(major: Self, minor: Self) -> Option<Axis> {
        use self::Axis::{X, Y, Z};
        match (major, minor) {
            (X, Y) | (Y, X) => Some(Z),
            (X, Z) | (Z, X) => Some(Y),
            (Z, Y) | (Y, Z) => Some(X),
            _ => None,
        }
    }
    fn get_sign(major: Self, minor: Self) -> Option<f32> {
        use self::Axis::{X, Y, Z};
        match (major, minor) {
            (X, Y) => Some(1.),
            (X, Z) => Some(1.),
            (Y, X) => Some(-1.),
            (Y, Z) => Some(1.),
            (Z, X) => Some(-1.),
            (Z, Y) => Some(-1.),
            _ => None,
        }
    }
}

pub const WIDTH: f32 = 100.;
pub const HEIGHT: f32 = 100.;
pub const LENGTH: f32 = 100.;

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut filters: ResMut<InteractionPairFilters>,
) {
    let lines_every = 10.;
    let minor_color = materials.add(StandardMaterial {
        base_color: Color::rgb(0., 0.6, 0.6),
        unlit: true,
        ..Default::default()
    });
    let major_color = materials.add(StandardMaterial {
        base_color: Color::rgb(0., 1., 1.),
        emissive: Color::rgb(0., 1., 1.),
        unlit: true,
        ..Default::default()
    });
    let mut platforms = HashMap::new();
    let mut handle_plane = |major_axis: Axis, minor_axis: Axis| {
        let (major, minor, tertiary) =
            Axis::get_sizes(major_axis, minor_axis, WIDTH, LENGTH, HEIGHT).unwrap();
        let tertiary_axis = Axis::get_tertiary(major_axis, minor_axis).unwrap();
        let lines = (major / lines_every) as i32;
        let mesh = {
            let size = minor_axis
                .set_translation(major_axis.set_translation(Default::default(), 0.1), minor);
            meshes.add(Mesh::from(shape::Box::new(size.x, size.y, size.z)))
        };

        for i in 1..lines {
            for side in [-1., 1.].iter().copied() {
                commands.spawn_bundle(PbrBundle {
                    mesh: mesh.clone(),
                    material: minor_color.clone(),
                    transform: Transform::from_translation(tertiary_axis.set_translation(
                        major_axis.set_translation(
                            Default::default(),
                            i as f32 * (major / lines_every) - (major / 2.),
                        ),
                        minor / 2. * side,
                    )),
                    ..Default::default()
                });
            }
        }
        for lr_side in [-1., 1.].iter().copied() {
            for side in [-1., 1.].iter().copied() {
                commands.spawn_bundle(PbrBundle {
                    mesh: mesh.clone(),
                    material: major_color.clone(),
                    transform: Transform::from_translation(tertiary_axis.set_translation(
                        major_axis.set_translation(Default::default(), WIDTH / 2. * lr_side),
                        LENGTH / 2. * side,
                    )),
                    ..Default::default()
                });
            }
        }
        let plane = minor_axis
            .set_translation(major_axis.set_translation(Default::default(), major), minor);
        let plane_offset = tertiary_axis.set_translation(Default::default(), tertiary / 2.)
            * Axis::get_sign(major_axis, minor_axis).unwrap();
        let plane_id = commands
            .spawn()
            .insert(RigidBodyBuilder::new_static())
            .insert(
                ColliderBuilder::cuboid(plane.x, plane.y, plane.z)
                    .translation(plane_offset.x, plane_offset.y, plane_offset.z)
                    .modify_solver_contacts(true),
            )
            .id();
        platforms.insert(plane_id.to_bits(), (-plane_offset).normalize().into());
    };
    handle_plane(Axis::X, Axis::Y);
    handle_plane(Axis::X, Axis::Z);
    handle_plane(Axis::Y, Axis::X);
    handle_plane(Axis::Y, Axis::Z);
    handle_plane(Axis::Z, Axis::X);
    handle_plane(Axis::Z, Axis::Y);
    filters.hook = Some(Box::new(OneWayPlatformHook { platforms }));
}
