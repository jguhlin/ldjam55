use bevy::prelude::*;

use crate::*;

pub struct UnitsUiPlugin<S: States> {
    pub state: S
}

impl<S: States> Plugin for UnitsUiPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Game::Playing), (load_units_bar)

        .run_if(in_state(self.state.clone())));
    }
}

fn load_units_bar(mut commands: Commands, assets: Res<GameAssets>, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    flex_direction: FlexDirection::Column,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,

                    ..default()
                },
                ..default()
            },
            Name::from("WholeScreenContainer"),
        ))
        .with_children(|parent| {
            // Bottom tool bar
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(80.0),
                            height: Val::Px(48.0),
                            border: UiRect::all(Val::Px(5.)),
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(5.0),
                            // align_items: AlignItems::Center,
                            // align_content: AlignContent::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            flex_direction: FlexDirection::Row,
                            grid_auto_flow: GridAutoFlow::Column,
                            display: Display::Grid,
                            ..default()
                        },
                        background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                        ..default()
                    },
                    Name::from("BottomBar"),
                ))
                .with_children(|parent| {
                    // First is a button to go back to the tower
                    // Second is a button for the defense army of the tower
                    // Then 7 buttons for the units
                    // So we have a button for each unit (group of mobs)

                    // Go-to Tower button
                    let style = Style {
                        width: Val::Px(32.0),
                        height: Val::Px(32.0),
                        ..default()
                    };

                    parent.spawn(ButtonBundle {
                        style: style.clone(),
                        image: UiImage::new(assets.icons.tower.clone()),
                        ..default()
                    });

                    parent.spawn(ButtonBundle {
                        style: style.clone(),
                        image: UiImage::new(assets.icons.shield.clone()),
                        ..default()
                    });

                    for i in 0..7 {
                        parent.spawn(ButtonBundle {
                            style: style.clone(),
                            image: UiImage::new(assets.icons.x.clone()),
                            ..default()
                        });
                    }
                });
        });
}
