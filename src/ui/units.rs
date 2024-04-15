use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy_ecs_tilemap::prelude::*;
use bevy_egui::egui::Ui;
use bevy_mod_picking::prelude::*;

use crate::*;

#[derive(Event)]
pub struct AddUnitEvent;

pub struct UnitsUiPlugin<S: States> {
    pub state: S,
}

impl<S: States> Plugin for UnitsUiPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_event::<GoToUnit>()
            .add_event::<AddUnitEvent>()
            .add_event::<AddUnitConfirm>()
            .add_systems(OnEnter(self.state.clone()), setup_units_bar)
            .add_systems(Update, interaction.run_if(in_state(self.state.clone())))
            .add_systems(Update, add_unit.run_if(on_event::<AddUnitEvent>()))
            .add_systems(
                PostUpdate,
                add_unit_confirm.run_if(on_event::<AddUnitConfirm>()),
            )
            .add_systems(Update, go_to_unit.run_if(on_event::<GoToUnit>()))
            .add_systems(Update, unit_panel.run_if(in_state(self.state.clone())));
    }
}

#[derive(Component)]
pub struct UnitInfoPanel;

#[derive(Component)]
pub struct DigButton;

fn unit_panel(
    mut commands: Commands,
    selected_unit: Res<SelectedUnit>,
    game_state: Res<GameState>,
    query: Query<(Entity, &UnitInfoPanel)>,
    unit_query: Query<(Entity, &Unit, &Slot, &TilePos, Option<&CanDig>)>,
    assets: Res<GameAssets>,
    mut dig_button_query: Query<&mut Visibility, With<DigButton>>,
) {
    // Despawn if nothing selected
    if selected_unit.unit.is_none() {
        for (e, _) in query.iter() {
            commands.entity(e).despawn_recursive();
        }
        return;
    }

    // Get unit info
    let unit = selected_unit.unit.unwrap();

    if !dig_button_query.is_empty() {
        let mut dig_button_visibility = dig_button_query.single_mut();

        for (e, u, slot, tilepos, candig) in unit_query.iter() {
            if slot.slot == unit {
                if candig.is_some() {
                    *dig_button_visibility = Visibility::Visible;
                } else {
                    *dig_button_visibility = Visibility::Hidden;
                }
            }
        }
    }

    // Create panel?
    if query.is_empty() {
        let text_style = TextStyle {
            font_size: 26.0,
            color: Color::rgba(1., 1., 1., 1.0).into(),
            font: assets.font.clone(),
            ..default()
        };

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::End,
                        ..default()
                    },
                    ..default()
                },
                UnitInfoPanel,
                Pickable::IGNORE,
            ))
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            width: Val::Percent(20.0),
                            height: Val::Percent(30.0),
                            bottom: Val::Percent(50.0),
                            padding: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        background_color: Color::rgba(0.1, 0.1, 0.1, 1.0).into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn((TextBundle {
                            text: Text::from_section("Unit Info", text_style.clone()),
                            ..default()
                        },));
                        parent
                            .spawn((
                                ButtonBundle {
                                    visibility: Visibility::Hidden,
                                    ..default()
                                },
                                DigButton,
                            ))
                            .with_children(|parent| {
                                parent.spawn((TextBundle {
                                    text: Text::from_section("Dig", text_style.clone()),
                                    ..default()
                                },));
                            });
                    });
            });
    }
}

fn go_to_unit(
    mut ev_gotounit: EventReader<GoToUnit>,
    mut ev_centercamera: EventWriter<CenterCamera>,
    q: Query<(&TilePos, &Slot), With<Unit>>,
) {
    for GoToUnit { slot } in ev_gotounit.read() {
        for (tile_pos, unit_slot) in q.iter() {
            if *slot == unit_slot.slot {
                ev_centercamera.send(CenterCamera { loc: *tile_pos });
            }
        }
    }
}

#[derive(Component, Deref, DerefMut, PartialEq, Eq)]
pub struct Slot {
    #[deref]
    pub slot: u8,
}

#[derive(Event)]
pub struct GoToUnit {
    pub slot: u8,
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
    mut ev_addunitcomplete: EventWriter<AddUnitComplete>,
    mut selected_unit: ResMut<SelectedUnit>,
    mut button_query: Query<(Entity, &Slot, &Button, &UiImage)>,
    mut ev_centercamera: EventWriter<CenterCamera>,
) {
    // Despawn the menu
    for (e, AddUnitMenu) in query.iter() {
        commands.remove_resource::<AddingUnit>();
        commands.entity(e).despawn_recursive();
    }

    let (map_transform, map_type, grid_size, tilemap_storage) = tilemap_q.single();

    // Get event data
    for AddUnitConfirm { slot, unit } in ev_addunitconfirm.read() {
        ev_addunitcomplete.send(AddUnitComplete);
        let unit = match unit {
            UnitType::Scout => Unit::scout(),
            UnitType::Excavation => Unit::excavation(),
            UnitType::Attack => Unit::attack(),
        };

        let mut spawn_pos = game_state.player_tower_location;
        // 1 below the player tower
        spawn_pos.1 -= 1;
        let spawn_pos = TilePos {
            x: spawn_pos.0 as u32,
            y: spawn_pos.1 as u32,
        };

        ev_centercamera.send(CenterCamera {
            loc: spawn_pos.clone(),
        });

        let spawn_pos = spawn_pos.center_in_world(grid_size, map_type).extend(3.5);
        let transform = *map_transform * Transform::from_translation(spawn_pos);

        let image = unit.unit_type.icon(&assets);

        let id = commands
            .spawn((
                Name::from(format!("Unit {}", slot)),
                unit,
                SpatialBundle {
                    transform,
                    ..default()
                },
                UnitUninitialized,
                Slot { slot: *slot },
                TilePos {
                    x: spawn_pos.x as u32,
                    y: spawn_pos.y as u32,
                },
            ))
            .id();

        // Update the game state
        game_state.units[*slot as usize] = UnitEntry::Summoned(id);

        selected_unit.unit = Some(*slot);

        // Update the button
        for (e, button_slot, _button, _image) in button_query.iter_mut() {
            if button_slot.slot == *slot {
                commands
                    .entity(e)
                    .insert(UiImage::new(image.clone()))
                    .remove::<AddUnitButton>();
            }
        }
    }
}

fn add_unit(
    mut commands: Commands,
    query: Query<(Entity, &AddUnitButton, &Button)>,
    assets: Res<GameAssets>,
) {
    commands.insert_resource(MenuOpen);

    for (_e, AddUnitButton(unit), button) in query.iter() {
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
                Pickable::IGNORE,
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
                                    focus_policy: FocusPolicy::Block,
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
                                    focus_policy: FocusPolicy::Block,
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
                                    focus_policy: FocusPolicy::Block,
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
    mut commands: Commands,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &mut BackgroundColor,
            &mut Style,
            &mut BorderColor,
            Option<&GoToTowerButton>,
            Option<&DefenseArmyButton>,
            Option<&AddUnitButton>,
            Option<&AddScoutUnitButton>,
            Option<&AddExcavationUnitButton>,
            Option<&AddAttackUnitButton>,
            Option<&Slot>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut ev_gototower: EventWriter<GoToTowerEvent>,
    mut ev_addunit: EventWriter<AddUnitEvent>,
    mut ev_addunitconfirm: EventWriter<AddUnitConfirm>,
    adding_unit: Option<Res<AddingUnit>>,
    mut selected_unit: ResMut<SelectedUnit>,
    mut ev_gotounit: EventWriter<GoToUnit>,
) {
    for (
        e,
        interaction,
        mut bg,
        mut style,
        mut border_color,
        go_to_tower,
        defense_army,
        add_unit,
        add_scout,
        add_excavation,
        add_attack,
        slot,
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
            selected_unit.unit = None;
            style.border = UiRect::all(Val::ZERO);
        }

        if add_unit.is_some() && *interaction == Interaction::Pressed {
            ev_addunit.send(AddUnitEvent);
            selected_unit.unit = None;
            style.border = UiRect::all(Val::ZERO);
        }

        if add_scout.as_ref().is_some() && *interaction == Interaction::Pressed {
            if adding_unit.as_ref().is_some() {
                let AddingUnit { slot } = **adding_unit.as_ref().unwrap();
                ev_addunitconfirm.send(AddUnitConfirm {
                    slot: slot,
                    unit: UnitType::Scout,
                });
                selected_unit.unit = None;
            }
            style.border = UiRect::all(Val::ZERO);
        }

        if add_excavation.as_ref().is_some() && *interaction == Interaction::Pressed {
            if adding_unit.as_ref().is_some() {
                let AddingUnit { slot } = **adding_unit.as_ref().unwrap();
                ev_addunitconfirm.send(AddUnitConfirm {
                    slot: slot,
                    unit: UnitType::Excavation,
                });
                selected_unit.unit = None;
            }
            style.border = UiRect::all(Val::ZERO);
        }

        if add_attack.as_ref().is_some() && *interaction == Interaction::Pressed {
            if adding_unit.as_ref().is_some() {
                let AddingUnit { slot } = **adding_unit.as_ref().unwrap();
                ev_addunitconfirm.send(AddUnitConfirm {
                    slot: slot,
                    unit: UnitType::Attack,
                });
                selected_unit.unit = None;
            }
            style.border = UiRect::all(Val::ZERO);
        }

        if slot.as_ref().is_some() && *interaction == Interaction::Pressed {
            let slot = slot.unwrap().slot;
            selected_unit.unit = Some(slot);
            style.border = UiRect::all(Val::Px(4.0));
            ev_gotounit.send(GoToUnit { slot });
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
            Pickable::IGNORE,
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
                            focus_policy: FocusPolicy::Block,
                            ..default()
                        },
                        GoToTowerButton,
                    ));

                    parent.spawn((
                        ButtonBundle {
                            style: style.clone(),
                            image: UiImage::new(assets.icons.shield.clone()),
                            focus_policy: FocusPolicy::Block,
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
                        let mut ec = parent.spawn((
                            ButtonBundle {
                                style: style.clone(),
                                image: UiImage::new(image),
                                focus_policy: FocusPolicy::Block,
                                ..default()
                            },
                            Slot { slot: n as u8 },
                        ));

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
