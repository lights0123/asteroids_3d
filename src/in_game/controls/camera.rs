use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::render::render_graph::base::camera::CAMERA_3D;
use bevy_rapier3d::rapier::geometry::{ColliderSet, Ray};
use bevy_rapier3d::rapier::pipeline::QueryPipeline;

use crate::in_game::asteroids::Asteroid;
use crate::in_game::TiedToGame;

use super::Controllable;

pub struct CameraPlugin<T>(pub T);

impl<T: crate::util::StateType> Plugin for CameraPlugin<T> {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_update(self.0.clone())
                .with_system(calc_camera_pos.system().chain(update_camera.system())),
        )
        .add_system_set(SystemSet::on_enter(self.0.clone()).with_system(startup.system()));
    }
}

fn startup(mut commands: Commands) {
    commands
        .spawn_bundle(PerspectiveCameraBundle::default())
        .insert(TiedToGame);
}

/// Find the ideal camera position. If an asteroid is in the way, the camera will be placed
/// closer.
fn calc_camera_pos(
    query_pipeline: Res<QueryPipeline>,
    colliders: Res<ColliderSet>,
    ship: Query<&Transform, With<Controllable>>,
    asteroids: Query<(), With<Asteroid>>,
) -> Vec3 {
    if let Ok(ship) = ship.single() {
        let offset = Vec3::new(0., 9., -30.);
        let vec = ship.rotation * offset.normalize();
        let hit = query_pipeline.cast_ray_and_get_normal(
            &colliders,
            &Ray::new(ship.translation.into(), vec.into()),
            offset.length(),
            true,
            Default::default(),
            Some(&|_, c| asteroids.get(Entity::from_bits(c.user_data as u64)).is_ok()),
        );

        if let Some(hit) = hit {
            ship.translation + vec * hit.1.toi
        } else {
            ship.translation + ship.rotation * offset
        }
    } else {
        Default::default()
    }
}

fn update_camera(
    In(camera_pos): In<Vec3>,
    mut query: QuerySet<(
        Query<&Transform, With<Controllable>>,
        Query<(&mut Transform, &Camera)>,
    )>,
) {
    if let Ok(&controllable) = query.q0().single() {
        for (mut transform, camera) in query.q1_mut().iter_mut() {
            if camera.name.as_deref() != Some(CAMERA_3D) {
                continue;
            }
            transform.translation = camera_pos;
            transform.look_at(
                controllable.translation + controllable.rotation * Vec3::new(0., 0., 20.),
                controllable.local_z(),
            );
        }
    }
}
