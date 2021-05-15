use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;

use crate::in_game::Points;

pub struct EndPlugin;

impl Plugin for EndPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let update_set = SystemSet::on_update(crate::AppState::End)
            .with_system(button_system.system())
            .with_system(quit_to_title.system());
        app.init_resource::<ButtonMaterials>()
            .add_system_set(
                SystemSet::on_enter(crate::AppState::End).with_system(setup_menu.system()),
            )
            .add_system_set(update_set)
            .add_system_set(
                SystemSet::on_exit(crate::AppState::End).with_system(remove_menu.system()),
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
            justify_content: JustifyContent::Center,
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

struct QuitToTitleButton;

fn setup_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
    points: Res<Points>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: materials.add(Color::rgba(0., 0., 0.05, 0.85).into()),
            ..Default::default()
        })
        .insert(PartOfUi)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                style: Style {
                    margin: Rect {
                        bottom: Val::Px(30.),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                text: Text::with_section(
                    format!("Score: {}", points.0),
                    TextStyle {
                        font: asset_server.load(asset!("RobotoCondensed-Regular.ttf")),
                        font_size: 40.0,
                        color: Color::rgb(0.5, 0.5, 1.0),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::ColumnReverse,
                        size: Size::new(Val::Px(200.), Val::Auto),
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
                        "BACK",
                    )
                    .insert(QuitToTitleButton);
                });
        });
}

fn remove_menu(mut commands: Commands, query: Query<Entity, With<PartOfUi>>) {
    query.for_each(|e| commands.entity(e).despawn_recursive());
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

fn quit_to_title(
    mut state: ResMut<State<crate::AppState>>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<QuitToTitleButton>)>,
) {
    if matches!(interaction_query.single(), Ok(&Interaction::Clicked)) {
        log_error!(state.replace(crate::AppState::Home));
    }
}
