use bevy::prelude::*;
use bevy_rapier3d::physics::EventQueue;
use bevy_rapier3d::rapier::geometry::{ColliderHandle, ColliderSet, ContactEvent};

use super::asteroids::{Asteroid, AsteroidBundle};
use super::bounds::ColliderProps;
use super::controls::Controllable;
use crate::in_game::points::AddPoints;
use crate::util::set_grab_cursor;

pub struct EventsPlugin<T>(pub T);

impl<T: crate::util::StateType> Plugin for EventsPlugin<T> {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<Contact>().add_system_set(
            SystemSet::on_update(self.0.clone())
                .with_system(events_adapter.system().label(EventAdapter))
                .with_system(ship_asteroid_contact.system().after(EventAdapter))
                .with_system(bullet_asteroid_contact.system().after(EventAdapter)),
        );
    }
}

#[derive(SystemLabel, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct EventAdapter;

enum Contact {
    Started(Entity, Entity),
    Stopped(Entity, Entity),
}

fn events_adapter(
    mut contact_events: EventWriter<Contact>,
    events: Res<EventQueue>,
    c: Res<ColliderSet>,
) {
    let get_entity = |collider: ColliderHandle| {
        c.get(collider)
            .map(|z| Entity::from_bits(z.user_data as u64))
    };

    while let Ok(contact_event) = events.contact_events.pop() {
        match contact_event {
            ContactEvent::Started(a, b) => {
                if let (Some(a), Some(b)) = (get_entity(a), get_entity(b)) {
                    contact_events.send(Contact::Started(a, b));
                }
            }
            ContactEvent::Stopped(a, b) => {
                if let (Some(a), Some(b)) = (get_entity(a), get_entity(b)) {
                    contact_events.send(Contact::Stopped(a, b));
                }
            }
        }
    }
}

fn ship_asteroid_contact(
    mut events: EventReader<Contact>,
    mut state: ResMut<State<crate::AppState>>,
    ship_query: Query<(), With<Controllable>>,
    asteroid_query: Query<(), With<Asteroid>>,
    mut windows: ResMut<Windows>,
    #[cfg(target_arch = "wasm32")] winit_windows: Res<bevy::winit::WinitWindows>,
) {
    for event in events.iter() {
        match *event {
            Contact::Started(a, b) => {
                if [(a, b), (b, a)]
                    .iter()
                    .copied()
                    .any(|(a, b)| (ship_query.get(a).is_ok() && asteroid_query.get(b).is_ok()))
                {
                    let window = windows.get_primary_mut().unwrap();
                    set_grab_cursor(
                        window,
                        false,
                        #[cfg(target_arch = "wasm32")]
                            &winit_windows,
                    );
                    log_error!(state.replace(crate::AppState::End));
                }
            }
            _ => {}
        }
    }
}

fn bullet_asteroid_contact(
    mut commands: Commands,
    mut events: EventReader<Contact>,
    mut points: EventWriter<AddPoints>,
    bullet_query: Query<(), With<super::Bullet>>,
    mut asteroid_query: Query<(&Transform, &mut Asteroid), With<Asteroid>>,
) {
    for event in events.iter() {
        match *event {
            Contact::Started(a, b) => {
                for (bullet, asteroid_e) in [(a, b), (b, a)].iter().copied() {
                    if let (Ok(_), Ok((asteroid_transform, asteroid))) =
                        (bullet_query.get(bullet), asteroid_query.get_mut(asteroid_e))
                    {
                        commands.entity(bullet).despawn_recursive();

                        let mut asteroid: Mut<Asteroid> = asteroid;
                        let asteroid_transform: &Transform = asteroid_transform;
                        // Wrapping so if two bullets hit at the same time, it doesn't panic
                        asteroid.hits = asteroid.hits.wrapping_sub(1);
                        if asteroid.hits == 0 {
                            points.send(AddPoints(asteroid.points));
                            commands.entity(asteroid_e).despawn_recursive();
                            for child in asteroid.children.iter() {
                                commands.spawn_bundle(AsteroidBundle {
                                    collider_props: ColliderProps {
                                        linvel: Default::default(),
                                        angvel: Default::default(),
                                    },
                                    ..AsteroidBundle::new(&child, *asteroid_transform)
                                });
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
