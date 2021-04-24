use bevy::prelude::*;
use bevy::render::camera::Camera;

use super::Controllable;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(startup.system())
            .add_system(update_camera.system());
    }
}

fn startup(mut commands: Commands) {
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0., 9., -6.).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..Default::default()
    });
}

fn update_camera(
    mut query: QuerySet<(
        Query<&Transform, With<Controllable>>,
        Query<&mut Transform, With<Camera>>,
    )>,
) {
    if let Ok(controllable) = query.q0().single() {
        let controllable: Transform = *controllable;
        if let Ok(camera) = query.q1_mut().single_mut() {
            // IntelliJ types
            let mut camera: Mut<Transform> = camera;
            camera.translation =
                controllable.translation + controllable.rotation * Vec3::new(0., 9., -30.);
            camera.look_at(
                controllable.translation + controllable.rotation * Vec3::new(0., 0., 20.),
                controllable.local_z(),
            );
        }
    }
}
