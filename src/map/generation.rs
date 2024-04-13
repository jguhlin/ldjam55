use bevy::prelude::*;
use noise::{Fbm, Perlin};
use noise::utils::{NoiseMapBuilder, PlaneMapBuilder, NoiseMap};

pub struct MapGenerationPlugin;

impl Plugin for MapGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, generate_world);
    }
}

fn create_map(seed: u32) -> NoiseMap {
    let fbm = Fbm::<Perlin>::new(seed);

    let noise_map = PlaneMapBuilder::new(fbm)
            .set_size(1000, 1000)
            .set_x_bounds(-1.0, 1.0)
            .set_y_bounds(-1.0, 1.0)
            .build();

    noise_map
}

fn simulate_rainfall_river_generation_erosion(mut map: NoiseMap, iterations: usize, rainfall_amount: f64) -> NoiseMap {
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
    


// Set to maximum height of 5.0
pub fn circular_border_mountain_range(mut map: NoiseMap) -> NoiseMap {
    let map_size = map.size();
    let width = map_size.0;
    let height = map_size.1;

    let center_x = width as f64 / 2.0;
    let center_y = height as f64 / 2.0;

    for x in 0..width {
        for y in 0..height {
            let distance = ((x as f64 - center_x).powi(2) + (y as f64 - center_y).powi(2)).sqrt();
            let val = map.get_value(x, y);
            let new_val = val + (1.0 - distance / 100.0);
            // Clamp to -5., 5.
            let new_val = new_val.min(1.0);
            let new_val = new_val.max(-1.0);
            // println!("{}:{} {} -> {}", x, y, val, new_val);
            map.set_value(x, y, new_val);
        }
    }

    map
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
        v if v < 0.8 => Color::hex("#b2f9b2"),
        v if v < 0.9 => Color::hex("#d9fcd9"),
        v if v <= 1.0 => Color::hex("#ffffff"),
        _ => panic!("Unexpected value for color")
    };
    color_result.expect("Getting color from HEX error")
}

#[derive(Resource, Deref)]
struct Root(Entity);

fn generate_world(
    mut commands: Commands,
) {
    let map = create_map(89432234);
    // let map = circular_border_mountain_range(map);
    let map = simulate_rainfall_river_generation_erosion(map, 5, 0.01);

    let (grid_width, grid_height) = map.size();
    debug!("Map size: {}x{}", grid_width, grid_height);

    let tile_size = 16_f32;

    let start_x = -(grid_width as f32) * tile_size / 2.0;
    let start_y = -(grid_height as f32) * tile_size / 2.0;

    let root = commands.spawn(
        SpatialBundle::default()
    ).with_children(|parent| {
        for col_x in 0..grid_width {
            for col_y in 0..grid_height {
                let val = map.get_value(col_x, col_y);
                let x = start_x + col_x as f32 * tile_size;
                let y = start_y + col_y as f32 * tile_size;

                parent.spawn(
                    SpriteBundle {
                        sprite: Sprite {
                            color: get_color(val),
                            custom_size: Some(Vec2::new(tile_size, tile_size)),
                            ..default()
                        },
                        transform: Transform::from_translation(Vec3::new(x, y, 0.)),
                        ..default()
                    }
                );
            }
        }
    }).id();

    commands.insert_resource(Root(root));
}
