#[cfg(not(target_arch = "wasm32"))]
use bevy::app::AppExit;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;

use crate::util::{cursor_locked, set_grab_cursor};

pub struct HomePlugin;

impl Plugin for HomePlugin {
    fn build(&self, app: &mut AppBuilder) {
        let update_set = SystemSet::on_update(crate::AppState::Home)
            .with_system(button_system.system())
            .with_system(resume_button_system.system())
            .with_system(resume.system());
        #[cfg(not(target_arch = "wasm32"))]
        let update_set = update_set.with_system(quit_button_system.system());
        app.init_resource::<ButtonMaterials>()
            .add_system_set(
                SystemSet::on_enter(crate::AppState::Home).with_system(setup_menu.system()),
            )
            .add_system_set(update_set)
            .add_system_set(
                SystemSet::on_exit(crate::AppState::Home).with_system(remove_menu.system()),
            );
    }
}

struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    pressed: Handle<ColorMaterial>,
}

impl FromWorld for ButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgba(0.15, 0.15, 0.15, 0.).into()),
            hovered: materials.add(Color::rgba(0.25, 0.25, 0.25, 0.9).into()),
            pressed: materials.add(Color::rgb(0.2, 0.2, 0.2).into()),
        }
    }
}

fn create_button<'a, 'c>(
    parent: &'c mut ChildBuilder<'a, '_>,
    button_materials: &ButtonMaterials,
    font: Handle<Font>,
    text: &str,
) -> EntityCommands<'a, 'c> {
    let mut button = parent.spawn_bundle(ButtonBundle {
        style: Style {
            size: Size::new(Val::Percent(100.), Val::Auto),
            padding: Rect::all(Val::Px(5.)),
            margin: Rect::all(Val::Auto),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        material: button_materials.normal.clone(),
        ..Default::default()
    });
    button.with_children(|parent| {
        parent.spawn_bundle(TextBundle {
            text: Text::with_section(
                text,
                TextStyle {
                    font,
                    font_size: 40.0,
                    color: Color::rgb(0.65, 0.65, 0.65),
                },
                Default::default(),
            ),
            ..Default::default()
        });
    });
    button
}

struct PartOfUi;

struct StartButton;

#[cfg(not(target_arch = "wasm32"))]
struct QuitButton;

fn setup_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            material: materials.add(Color::rgba(0., 0., 0.05, 0.85).into()),
            ..Default::default()
        })
        .insert(PartOfUi)
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(200.), Val::Auto),
                        margin: Rect {
                            left: Val::Px(30.),
                            ..Default::default()
                        },
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    material: materials.add(Color::rgba(0., 0., 0., 0.).into()),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::ColumnReverse,
                                ..Default::default()
                            },
                            material: materials.add(Color::rgba(0., 0., 0., 0.).into()),
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            create_button(
                                parent,
                                &button_materials,
                                asset_server.load(asset!("RobotoCondensed-Regular.ttf")),
                                "START",
                            )
                            .insert(StartButton);
                            #[cfg(not(target_arch = "wasm32"))]
                            create_button(
                                parent,
                                &button_materials,
                                asset_server.load(asset!("RobotoCondensed-Regular.ttf")),
                                "QUIT",
                            )
                            .insert(QuitButton);
                        });
                });
        });
}

fn remove_menu(mut commands: Commands, interaction_query: Query<Entity, With<PartOfUi>>) {
    interaction_query.for_each(|e| commands.entity(e).despawn_recursive());
}

fn button_system(
    button_materials: Res<ButtonMaterials>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut material, children) in interaction_query.iter_mut() {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                *material = button_materials.pressed.clone();
            }
            Interaction::Hovered => {
                *material = button_materials.hovered.clone();
                text.sections[0].style.color = Color::rgb(1., 1., 1.);
            }
            Interaction::None => {
                *material = button_materials.normal.clone();
                text.sections[0].style.color = Color::rgb(0.65, 0.65, 0.65);
            }
        }
    }
}

fn resume_button_system(
    mut windows: ResMut<Windows>,
    #[cfg(target_arch = "wasm32")] winit_windows: Res<bevy::winit::WinitWindows>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<StartButton>)>,
    mut change_next: Local<bool>,
) {
    if let Ok(interaction) = interaction_query.single() {
        if *interaction == Interaction::Clicked {
            *change_next = true;
        } else if *change_next {
            let window = windows.get_primary_mut().unwrap();
            set_grab_cursor(
                window,
                true,
                #[cfg(target_arch = "wasm32")]
                &winit_windows,
            );
            *change_next = false;
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn quit_button_system(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<QuitButton>)>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    if matches!(interaction_query.single(), Ok(&Interaction::Clicked)) {
        app_exit_events.send(AppExit);
    }
}

fn resume(
    mut state: ResMut<State<crate::AppState>>,
    windows: ResMut<Windows>,
    #[cfg(target_arch = "wasm32")] winit_windows: Res<bevy::winit::WinitWindows>,
) {
    let window = windows.get_primary().unwrap();
    if cursor_locked(
        window,
        #[cfg(target_arch = "wasm32")]
        &winit_windows,
    ) {
        log_error!(state.replace(crate::AppState::InGame));
    }
}
