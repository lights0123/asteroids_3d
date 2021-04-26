use bevy::pbr::AmbientLight;
use bevy::prelude::*;
use bevy_rapier3d::physics::{RapierConfiguration, RapierPhysicsPlugin};
use bevy_rapier3d::rapier::dynamics::MassProperties;
use bevy_rapier3d::rapier::math::Vector;

use crate::asteroids::{Asteroid, AsteroidsPlugin};
use crate::controls::{ControlPlugin, Controllable};
use crate::events::EventsPlugin;
use crate::game_area::GameAreaPlugin;
use crate::util::UtilPlugin;

#[macro_use]
mod util;
mod asteroids;
mod controls;
mod custom_asset;
mod events;
mod game_area;
mod physics;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut app = App::build();
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(wasm::WasmPlugin);
    app.insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins);
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);
    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new());
    app.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0 / 5.0f32,
    })
    .add_plugin(custom_asset::CustomAssetPlugin)
    .add_plugin(AsteroidsPlugin)
    .add_plugin(ControlPlugin)
    .add_plugin(UtilPlugin)
    .add_plugin(RapierPhysicsPlugin)
    .add_plugin(EventsPlugin)
    .add_plugin(GameAreaPlugin)
    .insert_resource(RapierConfiguration {
        gravity: Vector::default(),
        ..Default::default()
    })
    .add_startup_system(setup.system())
    .run();
}

struct Bullet;

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
}
