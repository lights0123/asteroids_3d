use std::sync::Arc;

use bevy::prelude::*;

use crate::custom_asset::CustomAsset;

pub struct AsteroidsPlugin;

impl Plugin for AsteroidsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<Asteroids>().add_system(spawn.system());
    }
}

pub struct Asteroid {
    pub hits: u8,
    pub children: Arc<Vec<Arc<AsteroidPlan>>>,
}

pub struct AsteroidPlan {
    pub hits: u8,
    pub pbr: PbrBundle,
    pub vhacd: Handle<CustomAsset>,
    pub children: Arc<Vec<Arc<AsteroidPlan>>>,
}

#[derive(Bundle)]
pub struct AsteroidBundle {
    #[bundle]
    pub pbr: PbrBundle,
    pub vhacd: Handle<CustomAsset>,
    pub asteroid: Asteroid,
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
                children: plan.children.clone(),
            },
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
            vhacd: plan.vhacd.clone(),
        }
    }
}

impl From<&Arc<AsteroidPlan>> for Asteroid {
    fn from(_: &Arc<AsteroidPlan>) -> Self {
        todo!()
    }
}

pub struct Asteroids(pub Vec<Arc<AsteroidPlan>>);

impl FromWorld for Asteroids {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        let asteroids = [Arc::new(AsteroidPlan {
            hits: 3,
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

fn spawn(
    mut _commands: Commands,
    _time: Res<Time>,
    _asteroids: Res<Asteroids>,
    mut _timer_opt: Local<Option<Timer>>,
) {
}
