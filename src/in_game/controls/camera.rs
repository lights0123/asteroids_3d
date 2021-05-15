use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::render::render_graph::base::camera::CAMERA_3D;

use crate::in_game::TiedToGame;

use super::Controllable;

pub struct CameraPlugin<T>(pub T);

impl<T: crate::util::StateType> Plugin for CameraPlugin<T> {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_update(self.0.clone()).with_system(update_camera.system()),
        )
        .add_system_set(SystemSet::on_enter(self.0.clone()).with_system(startup.system()));
    }
}

fn startup(mut commands: Commands) {
    commands
        .spawn_bundle(PerspectiveCameraBundle::default())
        .insert(TiedToGame);
}

fn update_camera(
    mut query: QuerySet<(
        Query<&Transform, With<Controllable>>,
        Query<(&mut Transform, &Camera)>,
    )>,
) {
    if let Ok(&controllable) = query.q0().single() {
        for (mut transform, camera) in query.q1_mut().iter_mut() {
            if camera.name.as_ref().map(|s| s.as_str()) != Some(CAMERA_3D) {
                continue;
            }
            transform.translation =
                controllable.translation + controllable.rotation * Vec3::new(0., 9., -30.);
            transform.look_at(
                controllable.translation + controllable.rotation * Vec3::new(0., 0., 20.),
                controllable.local_z(),
            );
        }
    }
}
