use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::*;

use crate::*;

pub struct UnitsPlugin<S: States> {
    pub state: S,
}

#[derive(Component)]
pub struct Digging {
    pub progress: f32,
}

impl Digging {
    pub fn new() -> Self {
        Self { progress: 0.0 }
    }
}

impl<S: States> Plugin for UnitsPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_event::<AddUnitComplete>()
            // .add_systems(OnEnter(self.state.clone()), setup_units_bar)
            .add_systems(
                PreUpdate,
                spawn_sprites.run_if(on_event::<AddUnitComplete>()),
            )
            .add_systems(
                PreUpdate,
                (prevent_collision).run_if(in_state(self.state.clone())),
            )
            .add_systems(Update, set_direction.run_if(in_state(self.state.clone())))
            .add_systems(
                Update,
                (unit_intersections, move_units).run_if(in_state(self.state.clone())),
            )
            .add_systems(Update, update_unit_pos.run_if(in_state(self.state.clone())))
            .add_systems(
                Update,
                units_fog_of_war.run_if(in_state(self.state.clone())),
            )
            .add_systems(Update, dig.run_if(in_state(self.state.clone())));
    }
}

pub fn dig(
    mut commands: Commands,
    mut query: Query<(Entity, &TilePos, &mut Digging, &mut Unit, &Slot)>,
    mut gamestate: ResMut<GameState>,
    mut treasure_locs: ResMut<TreasureLocs>,
    time: Res<Time>,
    stuff_q: Query<(&MapStuff, &TileStorage), (Without<MapGround>, Without<MapFogOfWar>)>,
    mut tile_query: Query<&mut TileVisible>,
) {
    let (_, stuff_tile_storage) = stuff_q.single();

    for (e, tile_pos, mut digging, mut unit, slot) in query.iter_mut() {
        log::info!("Digging... {:?}", digging.progress);

        digging.progress += unit.excavation_speed as f32 * time.delta_seconds();
        if digging.progress >= 100.0 {
            commands.entity(e).remove::<Digging>();

            // Add the treasure to the game state
            treasure_locs
                .locs
                .iter()
                .zip(treasure_locs.treasures.iter())
                .find(|(pos, _)| **pos == (tile_pos.x, tile_pos.y))
                .map(|(_, treasure)| {
                    gamestate.treasures_found.push(treasure.clone());
                });

            gamestate.score += gamestate.treasures_found.last().unwrap().score as u64;

            if let Some(tile_entity) = stuff_tile_storage.get(&tile_pos) {
                let mut visibility = tile_query.get_mut(tile_entity).unwrap();
                visibility.0 = false;
            }

            // Remove the treasure from the list
            treasure_locs
                .locs
                .retain(|&x| x != (tile_pos.x, tile_pos.y));

            let mut treasure_found = gamestate.treasures_found.last().unwrap().clone();
            log::info!("Treasure found: {:?}", treasure_found);

            // If less than 3 treasures found, then it's slot 0
            if gamestate.treasures_found.len() < 3 {
                treasure_found.slot = 0;
            }

            if !treasure_found.boons.is_empty() {
                let mut total_boons = &mut gamestate.unit_boons[treasure_found.slot as usize];
                for boon in &treasure_found.boons {
                    match boon.category {
                        BoonType::Health => {
                            total_boons.health.push((boon.operation, boon.value));
                        }
                        BoonType::Visibility => {
                            total_boons.visibility.push((boon.operation, boon.value));
                        }
                        BoonType::OverworldSpeed => {
                            total_boons
                                .overworld_speed
                                .push((boon.operation, boon.value));
                        }
                        BoonType::ExcavationSpeed => {
                            total_boons
                                .excavation_speed
                                .push((boon.operation, boon.value));
                        }
                        BoonType::BattleSpeed => {
                            total_boons.battle_speed.push((boon.operation, boon.value));
                        }
                        BoonType::Damage => {
                            total_boons.damage.push((boon.operation, boon.value));
                        }
                        BoonType::Members => {
                            total_boons.members.push((boon.operation, boon.value));
                        }
                    }
                }
            }

            // Do we have a spawned unit in this slot, then apply the boons
                if slot.slot == treasure_found.slot {
                    let total_boons = &gamestate.unit_boons[slot.slot as usize];

                    if !treasure_found.boons.is_empty() {
                        for boon in &treasure_found.boons {
                            match boon.category {
                                BoonType::Health => {
                                    let previous_total_health = unit.total_health;
                                    if boon.operation == BoonOperation::Add {
                                        unit.health_per_member += boon.value;
                                        unit.total_health =
                                            unit.members as f32 * unit.health_per_member as f32;
                                    } else {
                                        unit.health_per_member = (unit.health_per_member as f32
                                            * boon.value as f32)
                                            as u8;
                                        unit.total_health =
                                            unit.members as f32 * unit.health_per_member as f32;
                                        unit.current_health = unit.total_health;
                                    }
                                    let difference = unit.total_health - previous_total_health;
                                    unit.current_health += difference;
                                }
                                BoonType::Visibility => {
                                    if boon.operation == BoonOperation::Add {
                                        unit.visibility += boon.value as u8;
                                    } else {
                                        unit.visibility =
                                            (unit.visibility as i8 * boon.value as i8) as u8;
                                    }
                                }
                                BoonType::OverworldSpeed => {
                                    if boon.operation == BoonOperation::Add {
                                        unit.overworld_speed += boon.value as u8;
                                    } else {
                                        unit.overworld_speed =
                                            (unit.overworld_speed as i8 * boon.value as i8) as u8;
                                    }
                                }
                                BoonType::ExcavationSpeed => {
                                    if boon.operation == BoonOperation::Add {
                                        unit.excavation_speed += boon.value as u8;
                                    } else {
                                        unit.excavation_speed =
                                            (unit.excavation_speed as i8 * boon.value as i8) as u8;
                                    }
                                }
                                BoonType::BattleSpeed => {
                                    if boon.operation == BoonOperation::Add {
                                        unit.battle_speed += boon.value as u8;
                                    } else {
                                        unit.battle_speed =
                                            (unit.battle_speed as i8 * boon.value as i8) as u8;
                                    }
                                }
                                BoonType::Damage => {
                                    if boon.operation == BoonOperation::Add {
                                        unit.damage += boon.value as u8;
                                    } else {
                                        unit.damage = (unit.damage as i8 * boon.value as i8) as u8;
                                    }
                                }
                                BoonType::Members => {
                                    if boon.operation == BoonOperation::Add {
                                        unit.members += boon.value as u8;
                                    } else {
                                        unit.members =
                                            (unit.members as i8 * boon.value as i8) as u8;
                                    }
                                    // Trigger spawning more little guys
                                    commands.entity(e).insert(UnitUninitialized);
                                }
                            }
                        }
                    }
                }
        }
    }
}

fn unit_intersections(
    mut commands: Commands,
    query: Query<(Entity, &TilePos, &Unit, Option<&Digging>)>,
    treasures: Res<TreasureLocs>,
    // todo add other enemies / towers / etc
) {
    for (unit_entity, unit_tile_pos, _unit, digging) in query.iter() {
        if treasures.locs.contains(&(unit_tile_pos.x, unit_tile_pos.y)) && digging.is_none() {
            commands.entity(unit_entity).insert(CanDig);
        } else {
            commands.entity(unit_entity).remove::<CanDig>();
        }
    }
}

fn update_unit_pos(
    mut q: Query<(&mut TilePos, &Transform, &Unit), Without<TileVisible>>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &Transform,
        &MapGround,
    )>,
) {
    let (map_size, grid_size, map_type, tile_storage, map_transform, _) = tilemap_q.single();
    for (mut tile_pos, transform, _) in q.iter_mut() {
        let cursor_in_map_pos: Vec2 = {
            // Extend the cursor_pos vec3 by 0.0 and 1.0
            let cursor_pos = Vec4::from((transform.translation.xy(), 0.0, 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xy()
        };
        // Once we have a world position we can transform it into a possible tile position.
        if let Some(new_tile_pos) =
            TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type)
        {
            *tile_pos = new_tile_pos;
        }
    }
}

fn units_fog_of_war(
    q: Query<(Entity, &TilePos, &Unit, &UnitDirection)>,
    fog_q: Query<(&MapFogOfWar, &TileStorage), Without<MapStuff>>,
    mut tile_query: Query<&mut TileVisible>,
) {
    // todo make circular
    let (_map_fog_of_war, fog_tile_storage) = fog_q.single();

    for (_e, unit_tile_pos, unit, _) in q.iter() {
        let radius = unit.visibility as i32;

        for x in (unit_tile_pos.x as i32 - radius)..=(unit_tile_pos.x as i32 + radius) {
            for y in (unit_tile_pos.y as i32 - radius)..=(unit_tile_pos.y as i32 + radius) {
                let tile_pos = TilePos {
                    x: x as u32,
                    y: y as u32,
                };
                if let Some(tile_entity) = fog_tile_storage.get(&tile_pos) {
                    let mut visibility = tile_query.get_mut(tile_entity).unwrap();
                    visibility.0 = false;
                }
            }
        }
    }
}

// Need to give the unit a direction, then act on it later...
fn set_direction(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &Unit, &Slot)>,
    cursor_pos: Res<CursorPos>,
    mut ev_mapclick: EventReader<MapClick>,
    selected_unit: Res<SelectedUnit>,
) {
    // todo Not sure if this is needed
    if ev_mapclick.is_empty() || selected_unit.unit.is_none() {
        return;
    }

    let mapclick = ev_mapclick.read();

    for (e, transform, _, slot) in query.iter_mut() {
        if slot.slot != *selected_unit.unit.as_ref().unwrap() {
            continue;
        }
        log::info!("Setting direction for unit");
        let direction = cursor_pos.tile_position_real - transform.translation.xy();
        let direction = direction.normalize();
        commands.entity(e).insert(UnitDirection {
            direction: direction,
            destination: cursor_pos.tile_position_real,
            destination_in_tile_pos: cursor_pos.tile_position,
        });
    }
}

pub const MOVEMENT_SPEED_SCALE: f32 = 20.0;

fn move_units(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &Unit, &TilePos, &mut UnitDirection)>,
    time: Res<Time>,
) {
    for (e, mut transform, unit, tilepos, mut unit_direction) in query.iter_mut() {
        let direction = unit_direction.destination - transform.translation.xy();
        unit_direction.direction = direction.normalize();

        if *tilepos == unit_direction.destination_in_tile_pos {
            log::info!("Unit has reached destination");
            commands.entity(e).remove::<UnitDirection>();
        } else {
            transform.translation.x += unit_direction.direction.x
                * unit.overworld_speed as f32
                * time.delta_seconds()
                * MOVEMENT_SPEED_SCALE;
            transform.translation.y += unit_direction.direction.y
                * unit.overworld_speed as f32
                * time.delta_seconds()
                * MOVEMENT_SPEED_SCALE;
        }
    }
}

fn prevent_collision(mut query: Query<(&mut Transform, &UnitVisual, Entity)>) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([a1, mut a2]) = combinations.fetch_next() {
        // Make a buffer of at least 6 between each, width wise
        // and 14 height wise
        // Move at a diagonal

        let bits: [u8; 8] = unsafe { std::mem::transmute(a1.2.to_bits()) };

        let hash_a = xxh3_64(&bits[..]);

        let bits: [u8; 8] = unsafe { std::mem::transmute(a2.2.to_bits()) };

        let hash_b = xxh3_64(&bits[..]);

        // If a2 is within 6 of a1, move a2
        if (a1.0.translation.x - a2.0.translation.x).abs() < 7.0 {
            let mut move_distance = 0.1;

            if hash_a > hash_b {
                move_distance = -0.1;
            }

            a2.0.translation.x += move_distance;
        }

        if (a1.0.translation.y - a2.0.translation.y).abs() < 17.0 {
            let mut move_distance = 0.1;
            if hash_a > hash_b {
                move_distance = -0.1;
            }

            a2.0.translation.y += move_distance;
        }
    }
}

fn jitter_units(
    mut query: Query<(&mut Transform, &UnitVisual)>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
) {
    for (mut transform, _) in query.iter_mut() {
        // Don't get more than 20 away from 0,0
        let mut x_movement = rng.gen_range(-0.15..0.15);
        let mut y_movement = rng.gen_range(-0.15..0.15);

        if transform.translation.x.abs() > 20.0 {
            // Change sign so movement is towards 0
            if transform.translation.x > 0.0 {
                x_movement = -0.15;
            } else {
                x_movement = 0.15;
            }
        }
        transform.translation.x += x_movement;

        if transform.translation.y.abs() > 20.0 {
            // Change sign so movement is towards 0
            if transform.translation.y > 0.0 {
                y_movement = -0.15;
            } else {
                y_movement = 0.15;
            }
        }
        transform.translation.y += y_movement;
    }
}

fn spawn_sprites(
    mut commands: Commands,
    assets: Res<GameAssets>,
    query: Query<(&Unit, Entity, Option<&Children>), With<UnitUninitialized>>,
) {
    for (unit, entity, children) in query.iter() {
        let mut transform = Transform::from_translation(Vec3::ZERO);
        commands.entity(entity).remove::<UnitUninitialized>();

        let cur_children_count = match children {
            Some(children) => children.len(),
            None => 0,
        };

        // Add to children
        commands.entity(entity).with_children(|p| {
            for _ in 0..unit.members as usize - cur_children_count {
                log::info!("Spawning unit member");
                // Stagger the kids a little
                transform.translation.x += 0.5;
                transform.translation.y += 0.1;
                p.spawn((
                    SpriteSheetBundle {
                        texture: assets.tiles.clone(),
                        atlas: TextureAtlas {
                            layout: assets.tiles_layout.clone(),
                            index: unit.unit_type.index(),
                        },
                        // Place right below the player tower
                        transform,
                        ..default()
                    },
                    UnitVisual,
                ));
            }
        });
    }
}
