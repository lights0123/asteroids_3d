use bevy::prelude::*;
use bevy_rapier3d::physics::{RapierConfiguration, RapierPhysicsPlugin};
use bevy_rapier3d::rapier::dynamics::MassProperties;
use bevy_rapier3d::rapier::math::Vector;

use crate::custom_asset;
use crate::in_game::controls::Controllable;

mod asteroids;
mod bounds;
mod controls;
mod events;
mod game_area;
mod points;

pub struct InGamePlugin;

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        let state = super::AppState::InGame;
        app.add_plugin(RapierPhysicsPlugin)
            .insert_resource(RapierConfiguration {
                gravity: Vector::default(),
                ..Default::default()
            })
            .add_plugin(asteroids::AsteroidsPlugin(state))
            .add_plugin(controls::ControlPlugin(state))
            .add_plugin(events::EventsPlugin(state))
            .add_plugin(game_area::GameAreaPlugin(state))
            .add_plugin(bounds::CalcBoundsPlugin(state))
            .add_plugin(points::PointsPlugin(state))
            .add_system_set(SystemSet::on_enter(state).with_system(setup.system()))
            .add_system_set(SystemSet::on_resume(state).with_system(start.system()))
            .add_system_set(SystemSet::on_pause(state).with_system(stop.system()))
            .add_system_set(SystemSet::on_exit(state).with_system(teardown.system()));
    }
}

struct Bullet;

#[derive(Default)]
struct TiedToGame;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut transform = Transform::from_xyz(0., 0., 20.);
    transform.scale = Vec3::splat(0.1);
    transform.rotation = Quat::from_rotation_y(std::f32::consts::PI);
    commands
        .spawn_bundle(PbrBundle {
            mesh: asset_server.load(asset!("ship.glb", "Mesh0/Primitive0")),
            material: asset_server.load(asset!("ship.glb", "Material0")),
            ..Default::default()
        })
        .insert(Controllable)
        .insert(bounds::CalcBounds)
        .insert(TiedToGame)
        .insert(
            asset_server.load::<custom_asset::CustomAsset, _>(asset!(
                "vhacd/ship.custom",
                "Mesh0/Primitive0"
            )),
        )
        .insert(MassProperties::from_cuboid(
            1.,
            Vector::from_row_slice(&[0.5, 0.5, 0.5]),
        ));

    commands
        .spawn_bundle(LightBundle {
            light: Light {
                color: Color::rgb(1.0, 1.0, 1.0),
                depth: 0.1..20.0,
                fov: f32::to_radians(60.0),
                intensity: 200000.0,
                range: 2000.0,
                ..Default::default()
            },
            transform: Transform::from_xyz(200., 200., 72.),
            ..Default::default()
        })
        .insert(TiedToGame);
}

fn start(mut config: ResMut<RapierConfiguration>) {
    config.physics_pipeline_active = true;
    config.query_pipeline_active = true;
}

fn stop(mut config: ResMut<RapierConfiguration>) {
    config.physics_pipeline_active = false;
    config.query_pipeline_active = false;
}

fn teardown(mut commands: Commands, despawn: Query<Entity, With<TiedToGame>>) {
    despawn.for_each(|e| commands.entity(e).despawn_recursive());
}
