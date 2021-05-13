use std::sync::Arc;

use bevy::prelude::*;
use rand::distributions::Open01;
use rand::prelude::*;

use crate::custom_asset::CustomAsset;
use crate::in_game::TiedToGame;

use super::bounds::ColliderProps;
use super::game_area::{HEIGHT, LENGTH, WIDTH};

pub struct AsteroidsPlugin<T>(pub T);

impl<T: crate::util::StateType> Plugin for AsteroidsPlugin<T> {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<Asteroids>()
            .add_system_set(SystemSet::on_update(self.0.clone()).with_system(spawn.system()));
    }
}

pub struct Asteroid {
    pub hits: u8,
    pub points: u64,
    pub children: Arc<Vec<Arc<AsteroidPlan>>>,
}

pub struct AsteroidPlan {
    pub hits: u8,
    pub points: u64,
    pub pbr: PbrBundle,
    pub vhacd: Handle<CustomAsset>,
    pub children: Arc<Vec<Arc<AsteroidPlan>>>,
}

#[derive(Bundle)]
pub(super) struct AsteroidBundle {
    #[bundle]
    pub pbr: PbrBundle,
    pub vhacd: Handle<CustomAsset>,
    pub calc_bounds: super::bounds::CalcBounds,
    pub collider_props: ColliderProps,
    pub asteroid: Asteroid,
    pub tied_to_game: TiedToGame,
}

impl AsteroidBundle {
    pub fn new(plan: &AsteroidPlan, origin: Transform) -> Self {
        let mut transform = plan.pbr.transform;
        transform.translation =
            origin.rotation * (transform.translation - origin.translation) + origin.translation;
        transform.rotation = origin.rotation;
        transform.translation += origin.rotation * origin.translation;
        AsteroidBundle {
            asteroid: Asteroid {
                hits: plan.hits,
                points: plan.points,
                children: plan.children.clone(),
            },
            calc_bounds: Default::default(),
            collider_props: Default::default(),
            pbr: PbrBundle {
                mesh: plan.pbr.mesh.clone(),
                material: plan.pbr.material.clone(),
                main_pass: plan.pbr.main_pass.clone(),
                draw: plan.pbr.draw.clone(),
                visible: plan.pbr.visible.clone(),
                render_pipelines: plan.pbr.render_pipelines.clone(),
                transform,
                global_transform: plan.pbr.global_transform,
            },
            tied_to_game: Default::default(),
            vhacd: plan.vhacd.clone(),
        }
    }
}

pub struct Asteroids(pub Vec<Arc<AsteroidPlan>>);

impl FromWorld for Asteroids {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        let asteroids = [Arc::new(AsteroidPlan {
            hits: 3,
            points: 3,
            pbr: PbrBundle {
                mesh: asset_server.load(asset!("Itokawa_1_1.glb", "Mesh0/Primitive0")),
                material: asset_server.load(asset!("Itokawa_1_1.glb", "Material0")),
                transform: Transform::from_scale(Vec3::splat(0.1)),
                ..Default::default()
            },
            vhacd: asset_server.load(asset!("vhacd/Itokawa_1_1.custom", "Mesh0/Primitive0")),
            children: Arc::new(vec![
                Arc::new(AsteroidPlan {
                    hits: 2,
                    points: 2,
                    pbr: PbrBundle {
                        mesh: asset_server.load(asset!("Itokawa_broken_1.glb", "Mesh0/Primitive0")),
                        material: asset_server.load(asset!("Itokawa_broken_1.glb", "Material0")),
                        transform: Transform::from_scale(Vec3::splat(0.1))
                            * Transform::from_xyz(-76.0531, -14.2851, 4.33338),
                        ..Default::default()
                    },
                    vhacd: asset_server
                        .load(asset!("vhacd/Itokawa_broken_1.custom", "Mesh0/Primitive0")),
                    children: Arc::new(vec![]),
                }),
                Arc::new(AsteroidPlan {
                    hits: 1,
                    points: 1,
                    pbr: PbrBundle {
                        mesh: asset_server.load(asset!("Itokawa_broken_1.glb", "Mesh1/Primitive0")),
                        material: asset_server.load(asset!("Itokawa_broken_1.glb", "Material0")),
                        transform: Transform::from_scale(Vec3::splat(0.1))
                            * Transform::from_xyz(241.754, 8.16478, 8.81916),
                        ..Default::default()
                    },
                    vhacd: asset_server
                        .load(asset!("vhacd/Itokawa_broken_1.custom", "Mesh1/Primitive0")),
                    children: Default::default(),
                }),
            ]),
        })];

        let mut asteroids_vec = vec![];
        fn recurse(asteroid: Arc<AsteroidPlan>, asteroids: &mut Vec<Arc<AsteroidPlan>>) {
            asteroids.push(asteroid.clone());
            for child in asteroid.children.iter() {
                recurse(child.clone(), asteroids);
            }
        }
        for asteroid in std::array::IntoIter::new(asteroids) {
            recurse(asteroid, &mut asteroids_vec);
        }
        Asteroids(asteroids_vec)
    }
}

struct OpenNeg11;

impl Distribution<f32> for OpenNeg11 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f32 {
        rng.sample::<f32, _>(Open01) * if rng.gen::<bool>() { 1. } else { -1. }
    }
}

impl Distribution<[f32; 3]> for OpenNeg11 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [f32; 3] {
        [
            rng.sample(OpenNeg11),
            rng.sample(OpenNeg11),
            rng.sample(OpenNeg11),
        ]
    }
}

fn spawn(
    mut commands: Commands,
    time: Res<Time>,
    asteroids: Res<Asteroids>,
    mut timer_opt: Local<Option<Timer>>,
) {
    let mut rng = rand::thread_rng();
    let radius = (LENGTH * LENGTH + WIDTH * WIDTH + HEIGHT * HEIGHT).sqrt();
    if let Some(ref mut timer) = *timer_opt {
        if timer.tick(time.delta()).finished() {
            *timer = Timer::from_seconds(rng.gen_range(1.0..3.0), false);
            let vec = Vec3::from(rng.sample::<[f32; 3], _>(OpenNeg11)).normalize();
            let position = vec * radius;
            let mut origin = Transform::from_translation(position);
            origin.scale = Vec3::new(
                rng.gen_range(0.5..1.5),
                rng.gen_range(0.5..1.5),
                rng.gen_range(0.5..1.5),
            );
            let child = (asteroids.0).choose(&mut rng).unwrap();
            commands.spawn_bundle(AsteroidBundle {
                collider_props: ColliderProps {
                    linvel: -vec * rng.gen_range(5.0..8.0),
                    ..Default::default()
                },
                ..AsteroidBundle::new(&child, origin)
            });
        }
    } else {
        *timer_opt = Some(Timer::from_seconds(2., false));
    }
}
