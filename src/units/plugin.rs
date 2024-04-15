use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::*;

use crate::*;

pub struct UnitsPlugin<S: States> {
    pub state: S,
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
            );
    }
}

fn unit_intersections(
    mut commands: Commands,
    query: Query<(Entity, &TilePos, &Unit)>,
    treasures: Res<TreasureLocs>,
    // todo add other enemies / towers / etc
) {
    for (unit_entity, unit_tile_pos, _unit) in query.iter() {
        if treasures.locs.contains(&(unit_tile_pos.x, unit_tile_pos.y)) {
            log::info!("Unit found treasure");
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
    // Clear out a radius of 6 around moving units
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

fn prevent_collision(mut query: Query<(&mut Transform, &UnitVisual)>) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([a1, mut a2]) = combinations.fetch_next() {
        // Make a buffer of at least 6 between each, width wise
        // and 14 height wise
        // Move at a diagonal

        // If a2 is within 6 of a1, move a2 to the right
        if (a1.0.translation.x - a2.0.translation.x).abs() < 6.0 {
            a2.0.translation.x += 0.1;
        }

        // If a2 is within 14 of a1, move a2 down
        if (a1.0.translation.y - a2.0.translation.y).abs() < 14.0 {
            a2.0.translation.y -= 0.1;
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
    query: Query<(&Unit, Entity), (With<UnitUninitialized>, Without<Children>)>,
) {
    for (unit, entity) in query.iter() {
        log::info!("Spawning unit");
        let mut transform = Transform::from_translation(Vec3::ZERO);
        commands.entity(entity).remove::<UnitUninitialized>();
        log::info!("Unit members: {}", unit.members);
        // Add to children
        commands.entity(entity).with_children(|p| {
            for _ in 0..unit.members {
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
