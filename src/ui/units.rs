use bevy::prelude::*;

use crate::*;

pub struct UnitsUiPlugin;

impl Plugin for UnitsUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Game::Playing), (load_units_bar));
    }
}

fn load_units_bar(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,

                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Bottom tool bar
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(80.0),
                        height: Val::Px(80.0),
                        border: UiRect::all(Val::Px(5.)),
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(5.0),

                        ..default()
                    },
                    background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                    ..default()
                })
                .with_children(|parent| {
                    // Bottom tool bar
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.),
                                ..default()
                            },
                            background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            // text
                            parent.spawn((
                                TextBundle::from_section(
                                    "Text Example",
                                    TextStyle {
                                        font: asset_server.load("fonts/MonaspaceRadon-Regular.otf"),
                                        font_size: 30.0,
                                        ..default()
                                    },
                                )
                                .with_style(Style {
                                    margin: UiRect::all(Val::Px(5.)),
                                    ..default()
                                }),
                                // Because this is a distinct label widget and
                                // not button/list item text, this is necessary
                                // for accessibility to treat the text accordingly.
                                Label,
                            ));
                        });
                });
        });
}
