use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy_rapier3d::physics::RigidBodyHandleComponent;
use bevy_rapier3d::rapier::dynamics::{RigidBodyBuilder, RigidBodySet};
use bevy_rapier3d::rapier::geometry::ColliderBuilder;
use bevy_rapier3d::rapier::math::Vector;

use crate::util::DespawnTimer;
#[cfg(target_arch = "wasm32")]
use crate::wasm::{cursor_locked, set_grab_cursor};

mod camera;

pub struct ControlPlugin<T>(pub T);

impl<T: crate::util::StateType> Plugin for ControlPlugin<T> {
    fn build(&self, app: &mut AppBuilder) {
        let set = SystemSet::on_update(self.0.clone())
            .with_system(player_move.system())
            .with_system(player_look.system())
            .with_system(shoot.system())
            .with_system(cursor_lock.system())
            .with_system(transition_to_pause.system());
        #[cfg(not(target_arch = "wasm32"))]
        let set = set.with_system(cursor_unlock.system());

        app.init_resource::<MovementSettings>()
            .init_resource::<BulletAssets>()
            .add_system_set(set)
            .add_plugin(camera::CameraPlugin(self.0.clone()));
    }
}

pub struct Controllable;

struct BulletAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

impl FromWorld for BulletAssets {
    fn from_world(world: &mut World) -> Self {
        BulletAssets {
            mesh: world
                .get_resource_mut::<Assets<Mesh>>()
                .unwrap()
                .add(Mesh::from(shape::Capsule {
                    ..Default::default()
                })),
            material: world
                .get_resource_mut::<Assets<StandardMaterial>>()
                .unwrap()
                .add(StandardMaterial {
                    base_color: Color::rgb(1., 0., 0.),
                    emissive: Color::rgb(0.65, 0., 0.),
                    ..Default::default()
                }),
        }
    }
}

/// Mouse sensitivity and movement speed
pub struct MovementSettings {
    pub sensitivity: f32,
    pub accel: f32,
    pub speed_limit: f32,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.003,
            accel: 50.,
            speed_limit: 2.,
        }
    }
}

/// Handles keyboard input and movement
fn player_move(
    gamepad: Res<Option<Gamepad>>,
    axes: Res<Axis<GamepadAxis>>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    settings: Res<MovementSettings>,
    mut rigid_bodies: ResMut<RigidBodySet>,
    query: Query<(&Transform, &RigidBodyHandleComponent), With<Controllable>>,
) {
    for (transform, rigid_body_component) in query.iter() {
        let transform: &Transform = transform;
        let rigid_body_component: &RigidBodyHandleComponent = rigid_body_component;
        let forward = Vec3::Z;
        let right = -Vec3::X;
        let up = Vec3::Y;
        let mut force = {
            let mut force = Vec3::default();
            for key in keys.get_pressed() {
                match key {
                    KeyCode::W => force += forward,
                    KeyCode::S => force -= forward,
                    KeyCode::A => force -= right,
                    KeyCode::D => force += right,
                    KeyCode::Space => force += up,
                    KeyCode::LShift => force -= up,
                    _ => (),
                }
            }
            force.normalize_or_zero()
        };

        force += if let Some((x, y)) = gamepad.and_then(|gamepad| {
            Some((
                axes.get(GamepadAxis(gamepad, GamepadAxisType::LeftStickX))?,
                axes.get(GamepadAxis(gamepad, GamepadAxisType::LeftStickY))?,
            ))
        }) {
            forward * y + right * x
        } else {
            Vec3::default()
        };

        if force.length() > 2. {
            force = force.normalize() * 2.;
        }

        if let Some(rb) = rigid_bodies.get_mut(rigid_body_component.handle()) {
            let linvel: Vec3 = transform.rotation.inverse() * Vec3::from(rb.linvel().clone_owned());
            {
                for (&current, force) in linvel.as_ref().iter().zip(force.as_mut().iter_mut()) {
                    if current > settings.speed_limit {
                        *force = force.min(0.);
                    }
                    if current < -settings.speed_limit {
                        *force = force.max(0.);
                    }
                }
                rb.apply_force(
                    Vector::from(
                        transform.rotation * force * time.delta_seconds() * settings.accel,
                    ),
                    true,
                );
            }
            for (i, force) in force.as_ref().iter().enumerate() {
                if force.abs() < 0.0005 {
                    let mut v = Vec3::default();

                    v[i] = -linvel[i].clamp(-0.5, 0.5);
                    rb.apply_force(Vector::from(transform.rotation * v), true);
                }
            }
            {
                let mut linvel = rb.linvel().clone_owned();
                for vel in linvel.iter_mut() {
                    if (*vel).abs() < 0.001 {
                        *vel = 0.;
                    }
                }
                rb.set_linvel(linvel, true);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn set_grab_cursor(window: &mut Window, locked: bool) {
    if locked {
        window.set_cursor_position(window.position().unwrap().as_f32());
    }
    window.set_cursor_lock_mode(locked);
    window.set_cursor_visibility(!locked);
}

#[cfg(not(target_arch = "wasm32"))]
fn cursor_locked(window: &Window) -> bool {
    window.cursor_locked()
}

fn player_look(
    gamepad: Res<Option<Gamepad>>,
    axes: Res<Axis<GamepadAxis>>,
    settings: Res<MovementSettings>,
    time: Res<Time>,
    mut windows: ResMut<Windows>,
    #[cfg(target_arch = "wasm32")] winit_windows: Res<bevy::winit::WinitWindows>,
    mut state: EventReader<MouseMotion>,
    mut rigid_bodies: ResMut<RigidBodySet>,
    query: Query<(&Transform, &RigidBodyHandleComponent), With<Controllable>>,
) {
    let window = windows.get_primary_mut().unwrap();
    for (transform, rigid_body_component) in query.iter() {
        let mut torque: Vector<f32> = state
            .iter()
            .map(|ev| {
                let mut torque = Vector::<f32>::zeros();
                if cursor_locked(
                    window,
                    #[cfg(target_arch = "wasm32")]
                    &winit_windows,
                ) {
                    torque += Vector::from(
                        transform.local_x()
                            * (settings.sensitivity * ev.delta.y * window.height()).to_radians(),
                    );
                    torque -= Vector::from(
                        transform.local_y()
                            * (settings.sensitivity * ev.delta.x * window.width()).to_radians(),
                    );
                }
                torque * time.delta_seconds() * settings.accel
            })
            .sum();
        torque += if let Some((x, y)) = gamepad.and_then(|gamepad| {
            Some((
                axes.get(GamepadAxis(gamepad, GamepadAxisType::RightStickX))?,
                axes.get(GamepadAxis(gamepad, GamepadAxisType::RightStickY))?,
            ))
        }) {
            Vector::from(
                (transform.local_x() * (settings.sensitivity * y * window.height()).to_radians())
                    - (transform.local_y()
                        * (settings.sensitivity * x * window.width()).to_radians()),
            )
        } else {
            Vector::zeros()
        };
        if torque.magnitude() > 1. {
            torque = torque.normalize();
        }
        if let Some(rb) = rigid_bodies.get_mut(rigid_body_component.handle()) {
            {
                let x = rb.angvel();
                let mut torque = torque.clone_owned();
                for i in 0..3 {
                    if x[i] > 0.5 {
                        torque[i] = torque[i].min(0.);
                    }
                    if x[i] < -0.5 {
                        torque[i] = torque[i].max(0.);
                    }
                }
                rb.apply_torque(torque, true);
            }

            for i in 0..3 {
                if torque[i].abs() < 0.0005 {
                    let mut v = Vector::default();
                    v[i] = -rb.angvel()[i].clamp(-0.1, 0.1);
                    rb.apply_torque(v, true);
                }
            }
        }
    }
}

fn cursor_lock(
    mut windows: ResMut<Windows>,
    #[cfg(target_arch = "wasm32")] winit_windows: Res<bevy::winit::WinitWindows>,
    mouse_button_input: Res<Input<MouseButton>>,
) {
    let window = windows.get_primary_mut().unwrap();
    if !cursor_locked(
        window,
        #[cfg(target_arch = "wasm32")]
        &winit_windows,
    ) && mouse_button_input.just_pressed(MouseButton::Left)
    {
        set_grab_cursor(
            window,
            true,
            #[cfg(target_arch = "wasm32")]
            &winit_windows,
        );
    }
}

fn transition_to_pause(
    mut state: ResMut<State<crate::AppState>>,
    mut windows: ResMut<Windows>,
    #[cfg(target_arch = "wasm32")] winit_windows: Res<bevy::winit::WinitWindows>,
) {
    let window = windows.get_primary_mut().unwrap();
    if !cursor_locked(
        window,
        #[cfg(target_arch = "wasm32")]
        &winit_windows,
    ) {
        state.push(crate::AppState::Paused);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn cursor_unlock(
    mut windows: ResMut<Windows>,
    mut focus_events: EventReader<bevy::window::WindowFocused>,
) {
    let window = windows.get_primary_mut().unwrap();
    if let Some(e) = focus_events.iter().last() {
        if !e.focused && cursor_locked(window) {
            set_grab_cursor(window, false);
        }
    }
}

fn shoot(
    mut commands: Commands,
    windows: Res<Windows>,
    #[cfg(target_arch = "wasm32")] winit_windows: Res<bevy::winit::WinitWindows>,
    gamepad: Res<Option<Gamepad>>,
    mouse_button_input: Res<Input<MouseButton>>,
    button_inputs: Res<Input<GamepadButton>>,
    resources: Res<BulletAssets>,
    query: Query<&Transform, With<Controllable>>,
) {
    let window = windows.get_primary().unwrap();
    let gamepad_button_pressed =
        |gamepad| button_inputs.just_pressed(GamepadButton(gamepad, GamepadButtonType::South));
    if cursor_locked(
        window,
        #[cfg(target_arch = "wasm32")]
        &winit_windows,
    ) && mouse_button_input.just_pressed(MouseButton::Left)
        || gamepad.map_or(false, gamepad_button_pressed)
    {
        if let Ok(ship) = query.single() {
            let translation = ship.translation + ship.rotation * Vec3::new(0., 0., 5.);
            let mut transform = Transform::from_translation(translation);
            transform.rotation = ship.rotation * Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
            let local_z = ship.local_z() * 45.;
            commands
                .spawn_bundle(PbrBundle {
                    mesh: resources.mesh.clone(),
                    material: resources.material.clone(),
                    transform,
                    ..Default::default()
                })
                .insert(super::Bullet)
                .insert(
                    RigidBodyBuilder::new_dynamic()
                        .position(crate::util::nalgebra_pos(
                            transform.translation,
                            transform.rotation,
                        ))
                        .linvel(local_z.x, local_z.y, local_z.z),
                )
                .insert(ColliderBuilder::capsule_y(0.5, 0.1))
                .insert(DespawnTimer(Timer::from_seconds(5., false)));
        }
    }
}
