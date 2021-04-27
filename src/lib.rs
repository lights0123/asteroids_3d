use bevy::pbr::AmbientLight;
use bevy::prelude::*;

#[macro_use]
mod util;
mod custom_asset;
mod in_game;
mod pause;
mod physics;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Home,
    InGame,
    Paused,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
    let mut app = App::build();
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(wasm::WasmPlugin);
    app.insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_state(AppState::InGame);
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);
    // #[cfg(not(target_arch = "wasm32"))]
    // app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new());
    app.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0 / 5.0f32,
    })
    .init_resource::<Option<Gamepad>>()
    .add_system_to_stage(CoreStage::PreUpdate, connect_gamepad.system())
    .add_plugin(custom_asset::CustomAssetPlugin)
    .add_plugin(util::UtilPlugin)
    .add_plugin(in_game::InGamePlugin)
    .add_plugin(pause::PausePlugin)
    .run();
}

fn connect_gamepad(
    mut lobby: ResMut<Option<Gamepad>>,
    mut gamepad_event: EventReader<GamepadEvent>,
) {
    for event in gamepad_event.iter() {
        match &event {
            GamepadEvent(gamepad, GamepadEventType::Connected) => {
                if lobby.is_none() {
                    *lobby = Some(*gamepad);
                }
                println!("{:?} Connected", gamepad);
            }
            GamepadEvent(gamepad, GamepadEventType::Disconnected) => {
                if *lobby == Some(*gamepad) {
                    *lobby = None;
                }
                println!("{:?} Disconnected", gamepad);
            }
            _ => (),
        }
    }
}
