use std::fmt::Debug;
use std::hash::Hash;

use bevy::ecs::component::Component;
use bevy::prelude::*;
use bevy_rapier3d::na::Translation3;
use bevy_rapier3d::physics::ColliderHandleComponent;
use bevy_rapier3d::rapier::geometry::ColliderSet;
use bevy_rapier3d::rapier::math::{Isometry, Vector};

#[cfg(target_arch = "wasm32")]
pub use crate::wasm::{cursor_locked, set_grab_cursor};

pub struct UtilPlugin;

impl Plugin for UtilPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(attach_entity_id.system())
            .add_system(despawn_timer.system());
    }
}

struct UserIdSet;

fn attach_entity_id(
    mut commands: Commands,
    mut colliders: ResMut<ColliderSet>,
    query: Query<(Entity, &ColliderHandleComponent), Without<UserIdSet>>,
) {
    query.for_each(|(entity, collider): (Entity, &ColliderHandleComponent)| {
        if let Some(collider) = colliders.get_mut(collider.handle()) {
            collider.user_data = entity.to_bits() as u128;
            commands.entity(entity).insert(UserIdSet);
        }
    });
}

pub struct DespawnTimer(pub Timer);

fn despawn_timer(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(Entity, &mut DespawnTimer)>,
) {
    query.for_each_mut(|(entity, mut timer): (Entity, Mut<DespawnTimer>)| {
        if timer.0.tick(time.delta()).finished() {
            commands.entity(entity).despawn_recursive();
        }
    });
}

pub fn nalgebra_pos(pos: Vec3, rot: Quat) -> Isometry<f32> {
    Isometry::from_parts(Translation3::from(Vector::from(pos)), rot.into())
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! asset {
    ($path:literal, $part:literal) => {
        concat!($path, "#", $part)
    };
    ($path:literal) => {
        $path
    };
}

#[cfg(target_arch = "wasm32")]
macro_rules! asset {
    ($path:literal, $part:literal) => {
        concat!(asset!($path), "#", $part)
    };
    ($path:literal) => {
        include_str!(concat!(env!("OUT_DIR"), "/assets/", $path))
    };
}

macro_rules! log_error {
    ($ex:expr) => {
        if let Err(e) = $ex {
            ::bevy::prelude::error!(error = (&e as &(dyn ::std::error::Error + 'static)))
        }
    };
}

pub trait StateType: Component + Debug + Clone + Eq + Hash {}

impl<T: Component + Debug + Clone + Eq + Hash> StateType for T {}

#[cfg(not(target_arch = "wasm32"))]
pub fn cursor_locked(window: &Window) -> bool {
    window.cursor_locked()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_grab_cursor(window: &mut Window, locked: bool) {
    if locked {
        window.set_cursor_position(window.position().unwrap().as_f32());
    }
    window.set_cursor_lock_mode(locked);
    window.set_cursor_visibility(!locked);
}
