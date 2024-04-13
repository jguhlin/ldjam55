use bevy::prelude::*;

use crate::*;

#[derive(Event)]
pub struct AddUnitEvent;

pub struct UnitsUiPlugin<S: States> {
    pub state: S,
}

impl<S: States> Plugin for UnitsUiPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_event::<AddUnitEvent>();
        app.add_systems(OnEnter(self.state.clone()), setup_units_bar)
            .add_systems(Update, interaction.run_if(in_state(self.state.clone())));
    }
}

#[derive(Component)]
pub struct GoToTowerButton;

#[derive(Component)]
pub struct DefenseArmyButton;

#[derive(Component)]
pub struct AddUnitButton(u8);

impl AddUnitButton {
    pub fn new(unit: u8) -> Self {
        Self(unit)
    }
}

#[derive(Component)]
pub struct UnitButton;

const NORMAL_BUTTON: Color = Color::rgba(0.9, 0.9, 0.9, 0.9);
const HOVERED_BUTTON: Color = Color::rgba(1.0, 1.0, 1.0, 1.0);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn interaction(
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &mut BackgroundColor,
            Option<&GoToTowerButton>,
            Option<&DefenseArmyButton>,
            Option<&AddUnitButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut ev_gototower: EventWriter<GoToTowerEvent>,
    mut ev_addunit: EventWriter<AddUnitEvent>,
) {
    for (e, interaction, mut bg, go_to_tower, defense_army, add_unit) in
        interaction_query.iter_mut()
    {
        match *interaction {
            Interaction::Pressed => {
                bg.0 = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                bg.0 = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                bg.0 = NORMAL_BUTTON.into();
            }
        }

        if go_to_tower.is_some() && *interaction == Interaction::Pressed {
            ev_gototower.send(GoToTowerEvent);
        }
    }
}

fn setup_units_bar(mut commands: Commands, assets: Res<GameAssets>, game_state: Res<GameState>) {
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

                    parent.spawn((
                        ButtonBundle {
                            style: style.clone(),
                            image: UiImage::new(assets.icons.tower.clone()),
                            ..default()
                        },
                        GoToTowerButton,
                    ));

                    parent.spawn((
                        ButtonBundle {
                            style: style.clone(),
                            image: UiImage::new(assets.icons.shield.clone()),
                            ..default()
                        },
                        DefenseArmyButton,
                    ));

                    for (n, i) in game_state.units.iter().enumerate() {
                        let image = match i {
                            UnitEntry::Available => assets.icons.plus.clone(),
                            UnitEntry::Unavailable => assets.icons.x.clone(),
                            UnitEntry::Summoned(_) => assets.icons.shield.clone(), // todo
                        };
                        let mut ec = parent.spawn(ButtonBundle {
                            style: style.clone(),
                            image: UiImage::new(image),
                            ..default()
                        });

                        match i {
                            UnitEntry::Available => {
                                ec.insert(AddUnitButton::new(n as u8));
                            }
                            UnitEntry::Unavailable => (),
                            UnitEntry::Summoned(_) => (), // todo
                        };
                    }
                });
        });
}
