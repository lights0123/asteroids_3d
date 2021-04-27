use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<ButtonMaterials>().add_system_set(
            SystemSet::on_enter(crate::AppState::Paused).with_system(setup_menu.system()),
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
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
        }
    }
}

fn create_button<'a, 'c>(
    parent: &'c mut ChildBuilder<'a, '_>,
    button_materials: &ButtonMaterials,
    font: Handle<Font>,
    text: &str,
) -> EntityCommands<'a, 'c> {
    let mut c = parent.spawn_bundle(ButtonBundle {
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
    c.with_children(|parent| {
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
    c
}

fn setup_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // ui camera
    commands.spawn_bundle(UiCameraBundle::default());
    let button_entity = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                ..Default::default()
            },
            material: materials.add(Color::rgba(0., 0., 0.05, 0.85).into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(200.), Val::Auto),
                        margin: Rect {
                            left: Val::Px(10.),
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
                                asset_server.load("RobotoCondensed-Regular.ttf"),
                                "RESUME",
                            );
                            create_button(
                                parent,
                                &button_materials,
                                asset_server.load("RobotoCondensed-Regular.ttf"),
                                "QUIT",
                            );
                        });
                });
        });
}
