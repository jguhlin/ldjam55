use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::*;
use noise::utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use noise::{Fbm, Perlin, Value};
use rand::{Rng, SeedableRng};
use xxhash_rust::xxh3::xxh3_64;

use crate::*;

#[derive(Event)]
pub struct GoToTowerEvent;

pub struct MapGenerationPlugin;

impl Plugin for MapGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GoToTowerEvent>();
        app.add_systems(
            OnEnter(Game::MapGeneration),
            (
                generate_world,
                place_towers,
                draw_map,
                spawn_player_tower,
                center_camera_on_player_tower,
            )
                .chain(),
        )
        .add_systems(
            Update,
            center_camera_on_player_tower.run_if(on_event::<GoToTowerEvent>()),
        );
    }
}

fn center_camera_on_player_tower(
    state: Res<GameState>,
    mut query: Query<&mut Transform, With<Camera>>,
    mut q2: Query<&TilePos, (With<PlayerTower>, Without<Camera>)>,
    mut game: ResMut<NextState<Game>>,
) {
    game.set(Game::Playing);

    let player_tower_tilepos = q2.single();
    let mut camera_transform = query.single_mut();

    // Map is centered at 0, 0 and size is 1000, 1000
    // tile sizes are 32x32

    let player_tower_x = player_tower_tilepos.x;
    let player_tower_y = player_tower_tilepos.y;

    let x = player_tower_x as f32 * 32.0;
    let y = player_tower_y as f32 * 32.0;
    // Map is centered, so subtract
    camera_transform.translation = Vec3::new(x - 500.0 * 32.0, y - 500.0 * 32.0, 10.0);
}

fn spawn_player_tower(
    mut commands: Commands,
    state: Res<GameState>,
    mut q: Query<(Entity, &MapStuff, &mut TileStorage), Without<MapFogOfWar>>,
    mut fog_q: Query<(&MapFogOfWar, &TileStorage), Without<MapStuff>>,
) {
    let player_tower_location = state.player_tower_location;

    let (e, map_stuff, mut tile_storage) = q.single_mut();
    // No tile exists, need a new tile bundle
    let tile_pos = TilePos {
        x: player_tower_location.0 as u32,
        y: player_tower_location.1 as u32,
    };
    let tile_entity = commands
        .spawn((
            TileBundle {
                position: tile_pos,
                tilemap_id: TilemapId(e),
                texture_index: TileTextureIndex(6),
                ..Default::default()
            },
            PlayerTower,
        ))
        .id();
    tile_storage.set(&tile_pos, tile_entity);

    // Clear out a radius of 6 around the player tower
    // todo make circular
    let (_map_fog_of_war, fog_tile_storage) = fog_q.single_mut();
    for x in (player_tower_location.0 as i32 - 6)..=(player_tower_location.0 as i32 + 6) {
        for y in (player_tower_location.1 as i32 - 6)..=(player_tower_location.1 as i32 + 6) {
            let tile_pos = TilePos {
                x: x as u32,
                y: y as u32,
            };
            let tile_entity = fog_tile_storage.get(&tile_pos);
            if let Some(tile_entity) = tile_entity {
                commands.entity(tile_entity).despawn();
            }
        }
    }
}

fn draw_map(mut commands: Commands, assets: Res<GameAssets>, state: Res<GameState>) {
    // Do a basic 3 layers
    // 1 layer for ground
    // 2nd for stuff (towers, people, etc)
    // 3rd layer for fog of war (unexplored regions)
    let tile_texture_handle = assets.tiles.clone();

    let map_size = TilemapSize { x: 1000, y: 1000 };
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn(MapGround).id();

    for x in 0..1000_u32 {
        for y in 0..1000_u32 {
            let val = state.map.get_value(x as usize, y as usize);
            let tilemap_idx = get_index(val);
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(tilemap_idx),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 32.0, y: 32.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert((TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(tile_texture_handle),
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    },));

    let tile_texture_handle = assets.tiles.clone();
    // Spawn the second layer, but it's empty
    let tilemap_entity = commands.spawn(MapStuff).id();
    let mut tile_storage = TileStorage::empty(map_size);
    commands.entity(tilemap_entity).insert((TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(tile_texture_handle),
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 1.0),
        ..Default::default()
    },));

    // 3rd layer (Fog of war) all black (tilemap index 7)
    let tilemap_entity = commands.spawn(MapFogOfWar).id();
    let mut tile_storage = TileStorage::empty(map_size);
    for x in 0..1000_u32 {
        for y in 0..1000_u32 {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(7),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_texture_handle = assets.tiles.clone();
    commands.entity(tilemap_entity).insert((TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(tile_texture_handle),
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 2.0),
        ..Default::default()
    },));
}

fn create_map(seed: u32) -> NoiseMap {
    let fbm = Fbm::<Value>::new(seed);

    let mut noise_map = PlaneMapBuilder::new(fbm)
        .set_size(1000, 1000)
        .set_x_bounds(-1.0, 1.0)
        .set_y_bounds(-1.0, 1.0)
        .build();

    // Get average value
    let mut sum = 0.0;
    for x in 0..1000 {
        for y in 0..1000 {
            sum += noise_map.get_value(x, y);
        }
    }
    let avg = sum / 1_000_000.0;
    log::info!("Average value: {}", avg);

    // Average value should be between 0.3 and 0.7
    // Scale the map so that the average value is 0.5
    if avg < 0.3 || avg > 0.7 {
        let diff = 0.5 - avg;

        for x in 0..1000 {
            for y in 0..1000 {
                let val = noise_map.get_value(x, y);
                noise_map.set_value(x, y, val + diff);
            }
        }
    }

    // Min and max should be between 0 and 1

    // f64 so have to use fold
    let max = noise_map.iter().fold(f64::MIN, |acc, x| acc.max(*x));
    let min = noise_map.iter().fold(f64::MAX, |acc, x| acc.min(*x));
    log::info!("Min: {}, Max: {}", min, max);

    if min < 0.0 || max > 1.0 {
        for x in 0..1000 {
            for y in 0..1000 {
                let val = noise_map.get_value(x, y);
                if val < 0.0 || val > 1.0 {
                    noise_map.set_value(x, y, sigmoid(val));
                }
            }
        }
    }

    noise_map
}

// Hacky, from ml world, but whatev
fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

fn simulate_rainfall_river_generation_erosion(
    mut map: NoiseMap,
    iterations: usize,
    rainfall_amount: f64,
) -> NoiseMap {
    let map_size = map.size();
    let width = map_size.0;
    let height = map_size.1;

    // Rainfall should generate rivers and lakes, but also a little bit of erosion to smooth out places
    // Rivers will be downhill, but also valleys and lakes will be created

    for _ in 0..iterations {
        for x in 0..width {
            for y in 0..height {
                let val = map.get_value(x, y);
                let mut new_val = val;

                // Rainfall
                new_val += rainfall_amount;

                // Erosion
                let mut lowest_neighbour = val;
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                            let neighbour_val = map.get_value(nx as usize, ny as usize);
                            if neighbour_val < lowest_neighbour {
                                lowest_neighbour = neighbour_val;
                            }
                        }
                    }
                }

                new_val = (new_val + lowest_neighbour) / 2.0;

                map.set_value(x, y, new_val);
            }
        }
    }

    // Detect rivers and store as a tuple
    let mut rivers: Vec<(usize, usize)> = Vec::new();
    for x in 0..width {
        for y in 0..height {
            let val = map.get_value(x, y);
            let mut is_lowest = true;
            for dx in -1..=1 {
                for dy in -1..=1 {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let neighbour_val = map.get_value(nx as usize, ny as usize);
                        if neighbour_val < val {
                            is_lowest = false;
                            break;
                        }
                    }
                }
                if !is_lowest {
                    break;
                }
            }
            if is_lowest {
                rivers.push((x, y));
            }
        }
    }

    // Go through all rivers and fill the lowest neighbour
    for (x, y) in rivers {
        let val = map.get_value(x, y);
        let mut lowest_neighbour = val;
        let mut lowest_neighbour_pos = (x, y);
        for dx in -1..=1 {
            for dy in -1..=1 {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                    let neighbour_val = map.get_value(nx as usize, ny as usize);
                    if neighbour_val < lowest_neighbour {
                        lowest_neighbour = neighbour_val;
                        lowest_neighbour_pos = (nx as usize, ny as usize);
                    }
                }
            }
        }
        map.set_value(lowest_neighbour_pos.0, lowest_neighbour_pos.1, val);
    }

    map
}

fn get_index(val: f64) -> u32 {
    match val.abs() {
        // previously with .abs()
        // Dark blue water
        v if v < 0.03 => 0,
        // Light blue water
        v if v < 0.08 => 1,
        v if v < 0.1 => 2,
        v if v < 0.2 => 3,
        v if v < 0.3 => 4,
        v if v < 0.4 => 4,
        v if v < 0.5 => 4,
        v if v < 0.6 => 4,
        // Mountains (guessing)
        v if v < 0.7 => 5,
        v if v < 0.8 => 5,
        v if v < 0.9 => 5,
        v if v <= 1.0 => 5,
        _ => panic!("Unexpected value for color"),
    }
}

fn create_treasure_spots(seed: u32) -> Vec<(u32, u32)> {
    let mut locs = Vec::new();

    let mut rng = WyRand::seed_from_u64(seed as u64);

    // Generate between 200 and 500 treasure spots
    let num_treasure_spots = rng.gen_range(200..500);
    for _ in 0..num_treasure_spots {
        let x: u32 = rng.gen_range(0..1000);
        let y: u32 = rng.gen_range(0..1000);
        locs.push((x, y));
    }

    locs
}

fn generate_world(
    mut commands: Commands,
    mut gamestate: ResMut<GameState>,
    config: Res<GameConfig>,
) {
    let map = create_map(config.seed);
    let map = simulate_rainfall_river_generation_erosion(map, 2, 0.01);

    let treasure_spots = create_treasure_spots(config.seed);
    commands.insert_resource(TreasureLocs {
        locs: treasure_spots,
    });

    gamestate.map = map;
    log::info!("World generated");
}

fn place_towers(mut res: ResMut<GameState>, config: Res<GameConfig>) {
    let mut rng = WyRand::seed_from_u64(xxh3_64(&config.seed.to_le_bytes()[..]));

    let mut player_loc: (u32, u32);
    // Find a location that is within 200,200 and 800,800 (so not the edge of the map)
    player_loc = (rng.gen_range(200..800), rng.gen_range(200..800));
    loop {
        // Between 0.1 and 0.7
        let val = res
            .map
            .get_value(player_loc.0 as usize, player_loc.1 as usize);
        if val > 0.1 && val < 0.7 {
            break;
        }
        player_loc = (rng.gen_range(200..800), rng.gen_range(200..800));
    }

    log::info!("Player Tower Location: {:?}", player_loc);

    res.player_tower_location = player_loc;

    // Place between 10 and 20 enemy towers

    let num_enemy_towers = rng.gen_range(10..20);
    for _ in 0..num_enemy_towers {
        let mut enemy_loc: (u32, u32);
        enemy_loc = (rng.gen_range(100..900), rng.gen_range(100..900));
        loop {
            let val = res
                .map
                .get_value(enemy_loc.0 as usize, enemy_loc.1 as usize);
            if val > 0.1 {
                break;
            }
            enemy_loc = (rng.gen_range(200..800), rng.gen_range(200..800));
        }
        res.enemy_tower_locations.push(enemy_loc);
    }

    log::info!("Towers Placed");
}
