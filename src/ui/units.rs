use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::*;

#[derive(Event)]
pub struct AddUnitEvent;

pub struct UnitsUiPlugin<S: States> {
    pub state: S,
}

impl<S: States> Plugin for UnitsUiPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_event::<AddUnitEvent>()
            .add_event::<AddUnitConfirm>();
        app.add_systems(OnEnter(self.state.clone()), setup_units_bar)
            .add_systems(Update, interaction.run_if(in_state(self.state.clone())))
            .add_systems(Update, add_unit.run_if(on_event::<AddUnitEvent>()))
            .add_systems(
                Update,
                add_unit_confirm.run_if(on_event::<AddUnitConfirm>()),
            );
    }
}

#[derive(Event)]
pub struct AddUnitConfirm {
    slot: u8,
    unit: UnitType,
}

#[derive(Resource)]
pub struct AddingUnit {
    slot: u8,
}

const TEXT_COLOR: Color = Color::rgb(0.2, 0.2, 0.2);

#[derive(Component)]
pub struct AddUnitMenu;

#[derive(Component)]
pub struct AddScoutUnitButton;

#[derive(Component)]
pub struct AddExcavationUnitButton;

#[derive(Component)]
pub struct AddAttackUnitButton;

fn add_unit_confirm(
    mut commands: Commands,
    mut ev_addunitconfirm: EventReader<AddUnitConfirm>,
    mut game_state: ResMut<GameState>,
    query: Query<(Entity, &AddUnitMenu)>,
    assets: Res<GameAssets>,
    tilemap_q: Query<(&Transform, &TilemapType, &TilemapGridSize, &TileStorage), With<MapStuff>>,
) {
    // Despawn the menu
    for (e, AddUnitMenu) in query.iter() {
        commands.remove_resource::<AddingUnit>();
        commands.entity(e).despawn_recursive();
    }

    let (map_transform, map_type, grid_size, tilemap_storage) = tilemap_q.single();

    // Get event data
    for AddUnitConfirm { slot, unit } in ev_addunitconfirm.read() {
        let unit = match unit {
            UnitType::Scout => Unit::scout(),
            UnitType::Excavation => Unit::excavation(),
            UnitType::Attack => Unit::attack(),
        };

        let mut spawn_pos = game_state.player_tower_location;
        // 1 below the player tower
        spawn_pos.1 -= 1;
        let spawn_pos = TilePos { x: spawn_pos.0 as u32, y: spawn_pos.1 as u32 };
        let spawn_pos = spawn_pos.center_in_world(grid_size, map_type).extend(3.5);
        let transform = *map_transform * Transform::from_translation(spawn_pos);

        log::info!("Spawning unit at {:?} because of pos {:?}", transform, game_state.player_tower_location);

        let id = commands
            .spawn((
                Name::from(format!("Unit {}", slot)),
                unit,
                SpriteSheetBundle {
                    texture: assets.tiles.clone(),
                    atlas: TextureAtlas {
                        layout: assets.tiles_layout.clone(),
                        index: 8,
                    },
                    // Place right below the player tower
                    transform,
                    ..default()
                },
            ))
            .id();

        log::info!("Spawned at {:?} with id {:?}", transform, id);

        // Update the game state
        game_state.units[*slot as usize] = UnitEntry::Summoned(id);
    }
}

fn add_unit(
    mut commands: Commands,
    ev_addunit: Res<Events<AddUnitEvent>>,
    mut game_state: ResMut<GameState>,
    query: Query<(Entity, &AddUnitButton, &Button)>,
    assets: Res<GameAssets>,
) {
    for (e, AddUnitButton(unit), button) in query.iter() {
        commands.insert_resource(AddingUnit { slot: *unit });

        // Display a set of buttons to ask what type of unit to summon
        // Then, when the user clicks on one, we'll send a SummonUnitEvent
        // Common style for all buttons on the screen
        let button_style = Style {
            width: Val::Px(250.0),
            height: Val::Px(65.0),
            margin: UiRect::all(Val::Px(20.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };

        let button_text_style = TextStyle {
            font_size: 26.0,
            color: TEXT_COLOR,
            font: assets.font.clone(),
            ..default()
        };

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
                AddUnitMenu,
            ))
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::rgba(0.1, 0.1, 0.1, 0.0).into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                AddScoutUnitButton,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    "Scout Unit",
                                    button_text_style.clone(),
                                ));
                            });

                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                AddExcavationUnitButton,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    "Excavation Unit",
                                    button_text_style.clone(),
                                ));
                            });

                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                AddAttackUnitButton,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    "Attack Unit",
                                    button_text_style.clone(),
                                ));
                            });
                    });
            });
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
            Option<&AddScoutUnitButton>,
            Option<&AddExcavationUnitButton>,
            Option<&AddAttackUnitButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut ev_gototower: EventWriter<GoToTowerEvent>,
    mut ev_addunit: EventWriter<AddUnitEvent>,
    mut ev_addunitconfirm: EventWriter<AddUnitConfirm>,
    adding_unit: Option<Res<AddingUnit>>,
) {
    for (
        e,
        interaction,
        mut bg,
        go_to_tower,
        defense_army,
        add_unit,
        add_scout,
        add_excavation,
        add_attack,
    ) in interaction_query.iter_mut()
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

        if add_unit.is_some() && *interaction == Interaction::Pressed {
            ev_addunit.send(AddUnitEvent);
        }

        if add_scout.as_ref().is_some() && *interaction == Interaction::Pressed {
            if adding_unit.as_ref().is_some() {
                let AddingUnit { slot } = **adding_unit.as_ref().unwrap();
                ev_addunitconfirm.send(AddUnitConfirm {
                    slot: slot,
                    unit: UnitType::Scout,
                });
            }
        }

        if add_excavation.as_ref().is_some() && *interaction == Interaction::Pressed {
            if adding_unit.as_ref().is_some() {
                let AddingUnit { slot } = **adding_unit.as_ref().unwrap();
                ev_addunitconfirm.send(AddUnitConfirm {
                    slot: slot,
                    unit: UnitType::Excavation,
                });
            }
        }

        if add_attack.as_ref().is_some() && *interaction == Interaction::Pressed {
            if adding_unit.as_ref().is_some() {
                let AddingUnit { slot } = **adding_unit.as_ref().unwrap();
                ev_addunitconfirm.send(AddUnitConfirm {
                    slot: slot,
                    unit: UnitType::Attack,
                });
            }
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
