#![allow(clippy::type_complexity)]
#![feature(trivial_bounds)]

pub use bevy::prelude::*;
pub use bevy_ecs_tilemap::prelude::*;
pub use bevy_prng::WyRand;
pub use bevy_rand::prelude::{EntropyComponent, EntropyPlugin, ForkableRng, GlobalEntropy};
pub use noise::utils::NoiseMap;
pub use rand::prelude::{IteratorRandom, Rng};
pub use xxhash_rust::xxh3::xxh3_64;

pub mod map;
pub mod treasures;
pub mod ui;
pub mod units;

pub use map::*;
pub use treasures::*;
pub use ui::*;
pub use units::*;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Game {
    #[default]
    Startup,
    MapGeneration,
    Loading, // Here, but dunno if we need it
    Playing,
    Menu,
    Paused,
}

#[derive(Resource)]
pub struct GameConfig {
    pub seed: u32,
}

#[derive(Component)]
pub struct MapGround;

#[derive(Component)]
pub struct MapStuff;

#[derive(Component)]
pub struct MapFogOfWar;

#[derive(Component)]
pub struct PlayerTower;

#[derive(Resource)]
pub struct TreasureLocs {
    pub locs: Vec<(u32, u32)>,
    pub treasures: Vec<Treasure>,
}

#[derive(Resource)]
pub struct GameState {
    pub player_tower_location: (u32, u32),
    pub enemy_tower_locations: Vec<(u32, u32)>,
    pub map: NoiseMap,
    pub score: u64,
    pub units: [UnitEntry; 7],
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            player_tower_location: (0, 0),
            enemy_tower_locations: vec![],
            map: NoiseMap::default(),
            score: 0,
            units: [
                UnitEntry::Available,
                UnitEntry::Unavailable,
                UnitEntry::Unavailable,
                UnitEntry::Unavailable,
                UnitEntry::Unavailable,
                UnitEntry::Unavailable,
                UnitEntry::Unavailable,
            ],
        }
    }
}

#[derive(Resource, Default)]
pub struct GameAssets {
    pub tiles: Handle<Image>,
    pub tiles_layout: Handle<TextureAtlasLayout>,
    pub icons: Icons,
    pub font: Handle<Font>,
}

#[derive(Default)]
pub struct Icons {
    pub tower: Handle<Image>,
    pub x: Handle<Image>,
    pub shield: Handle<Image>,
    pub plus: Handle<Image>,

    // Unit icons for toolbar
    pub scout: Handle<Image>,
    pub excavator: Handle<Image>,
    pub attack: Handle<Image>,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self { seed: 442 }
    }
}

#[derive(Resource)]
pub struct CursorPos {
    pub mouse_position: Vec2,
    pub tile_position: TilePos,
    pub tile_position_real: Vec2,
}

impl Default for CursorPos {
    fn default() -> Self {
        // Initialize the cursor pos at some far away place. It will get updated
        // correctly when the cursor moves.
        Self {
            mouse_position: Vec2::new(-1000.0, -1000.0),
            tile_position: TilePos::default(),
            tile_position_real: Vec2::new(-1000.0, -1000.0),
        }
    }
}

#[derive(Resource, Default)]
pub struct SelectedUnit {
    pub unit: Option<u8>, // slot
}

#[derive(Resource)]
pub struct MenuOpen;
