use bevy::prelude::*;
use bevy_rand::prelude::*;
use bevy_prng::WyRand;
use bevy_ecs_tilemap::prelude::*;
use noise::utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use noise::{Fbm, Perlin, Value};
use rand::Rng;

use crate::{GameAssets, GameConfig, GameState};

pub struct MapGenerationPlugin;

impl Plugin for MapGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, 
            (
                generate_world,
                place_towers,

                draw_map,
            
            ).chain()
        
        );
    }
}

fn draw_map(mut commands: Commands,
    asset_server: Res<AssetServer>,
    assets: Res<GameAssets>,
    mut state: ResMut<GameState>,
    ) 
{

    // Do a basic 3 layers
    // 1 layer for ground
    // 2nd for stuff (towers, people, etc)
    // 3rd layer for fog of war (unexplored regions)
    let tile_texture_handle = assets.tiles.clone();

    let map_size = TilemapSize { x: 1000, y: 1000 };
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

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

    commands.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(tile_texture_handle),
            tile_size,
            transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
            ..Default::default()
        },
    ));
}


fn create_map(seed: u32) -> NoiseMap {
    let fbm = Fbm::<Value>::new(seed);

    let noise_map = PlaneMapBuilder::new(fbm)
        .set_size(1000, 1000)
        .set_x_bounds(-1.0, 1.0)
        .set_y_bounds(-1.0, 1.0)
        .build();

    noise_map
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
    match val.abs() { // previously with .abs()
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

fn get_color(val: f64) -> Color {
    let color_result = match val.abs() {
        // Dark blue water
        v if v < 0.03 => Color::hex("#0000ff"),
        // Light blue water
        v if v < 0.08 => Color::hex("#00aaff"),
        v if v < 0.1 => Color::hex("#0a7e0a"),
        v if v < 0.2 => Color::hex("#0da50d"),
        v if v < 0.3 => Color::hex("#10cb10"),
        v if v < 0.4 => Color::hex("#18ed18"),
        v if v < 0.5 => Color::hex("#3ff03f"),
        v if v < 0.6 => Color::hex("#65f365"),
        v if v < 0.7 => Color::hex("#8cf68c"),
        // Mountains (guessing)
        v if v < 0.8 => Color::hex("#b2f9b2"),
        v if v < 0.9 => Color::hex("#d9fcd9"),
        v if v <= 1.0 => Color::hex("#ffffff"),
        _ => panic!("Unexpected value for color"),
    };
    color_result.unwrap()
}

fn create_treasure_spots(rng: &mut GlobalEntropy<WyRand>) -> Vec<(usize, usize)> {
    let mut locs = Vec::new();

    // Generate between 200 and 500 treasure spots
    let num_treasure_spots = rng.gen_range(200..500);
    for _ in 0..num_treasure_spots {
        let x = rng.gen_range(0..1000);
        let y = rng.gen_range(0..1000);
        locs.push((x, y));
    }

    locs
}

#[derive(Resource, Deref)]
struct Root(Entity);

fn generate_world(
    mut rng: ResMut<GlobalEntropy<WyRand>>,
    mut gamestate: ResMut<GameState>,
) {
    let map = create_map(rng.gen::<u32>());
    let map = simulate_rainfall_river_generation_erosion(map, 10, 0.01);

    let treasure_spots = create_treasure_spots(&mut *rng);
    log::info!("Treasure spots: {:?}", treasure_spots.len());

    gamestate.map = map;
}

fn place_towers(
    mut res: ResMut<GameState>,
    mut rng: ResMut<GlobalEntropy<WyRand>>) 
{
    let mut player_loc: (u64, u64);
    // Find a location that is within 200,200 and 800,800 (so not the edge of the map)
    player_loc = (rng.gen_range(200..800), rng.gen_range(200..800));
    loop {
        // Between 0.1 and 0.7
        let val = res.map.get_value(player_loc.0 as usize, player_loc.1 as usize);
        if val > 0.1 && val < 0.7 {
            break;
        }
        player_loc = (rng.gen_range(200..800), rng.gen_range(200..800));
    }

    res.player_tower_location = player_loc;

    // Place between 10 and 20 enemy towers

    let num_enemy_towers = rng.gen_range(10..20);
    for _ in 0..num_enemy_towers {
        let mut enemy_loc: (u64, u64);
        enemy_loc = (rng.gen_range(100..900), rng.gen_range(100..900));
        loop {
            let val = res.map.get_value(enemy_loc.0 as usize, enemy_loc.1 as usize);
            if val > 0.1 {
                break;
            }
            enemy_loc = (rng.gen_range(200..800), rng.gen_range(200..800));
        }
        res.enemy_tower_locations.push(enemy_loc);
    }
}