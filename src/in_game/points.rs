use bevy::prelude::*;

pub struct PointsPlugin<T>(pub T);

impl<T: crate::util::StateType> Plugin for PointsPlugin<T> {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<Points>()
            .add_event::<AddPoints>()
            .add_system_set(SystemSet::on_enter(self.0.clone()).with_system(setup.system()))
            .add_system_set(SystemSet::on_exit(self.0.clone()).with_system(leave.system()))
            .add_system_set(
                SystemSet::on_update(self.0.clone())
                    .with_system(update_points.system().label(PointsSystem)),
            );
    }
}

struct PartOfUi;
struct ScoreLabel;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct Points(u64);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct AddPoints(pub u64);

#[derive(SystemLabel, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct PointsSystem;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut points: ResMut<Points>,
) {
    points.0 = 0;
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                padding: Rect::all(Val::Px(5.0)),
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            material: materials.add(Color::BLACK.into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: Text::with_section(
                        "Score:",
                        TextStyle {
                            font: asset_server.load(asset!("RobotoCondensed-Regular.ttf")),
                            font_size: 40.0,
                            color: Color::rgb(0.5, 0.5, 1.0),
                        },
                        Default::default(),
                    ),
                    ..Default::default()
                })
                .insert(ScoreLabel);
        })
        .insert(PartOfUi);
}

fn leave(mut commands: Commands, query: Query<Entity, With<PartOfUi>>) {
    query.for_each(|e| commands.entity(e).despawn_recursive());
}

fn update_points(
    mut points: ResMut<Points>,
    mut e: EventReader<AddPoints>,
    mut query: Query<&mut Text, With<ScoreLabel>>,
) {
    points.0 += e.iter().map(|p| p.0).sum::<u64>();
    if let Ok(mut text) = query.single_mut() {
        text.sections[0].value = format!("Score: {}", points.0);
    }
}
