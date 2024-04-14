#![allow(clippy::type_complexity)]
#![feature(trivial_bounds)]

pub use bevy::prelude::*;
pub use bevy_prng::WyRand;
pub use bevy_rand::prelude::{EntropyComponent, EntropyPlugin, ForkableRng, GlobalEntropy};
pub use noise::utils::NoiseMap;
pub use rand::prelude::{IteratorRandom, Rng};
pub use xxhash_rust::xxh3::xxh3_64;

pub mod map;
pub mod ui;
pub mod units;

pub use map::*;
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

pub enum UnitEntry {
    Unavailable,
    Available,
    Summoned(Entity),
}

#[derive(Component)]
pub struct Unit {
    pub health: u32,
    pub overworld_speed: u32,
    pub excavation_speed: u32,
    pub battle_speed: u32,
    pub members: u32,
    // todo: damage, health regen, attack types, etc...
}

impl Unit {
    pub fn scout() -> Self {
        Self {
            health: 100,
            overworld_speed: 20,
            excavation_speed: 10,
            battle_speed: 10,
            members: 1,
        }
    }
    pub fn excavation() -> Self {
        Self {
            health: 150,
            overworld_speed: 10,
            excavation_speed: 20,
            battle_speed: 4,
            members: 2,
        }
    }

    // todo: archers, mages, infantry, etc...
    pub fn attack() -> Self {
        Self {
            health: 100,
            overworld_speed: 10,
            excavation_speed: 1,
            battle_speed: 10,
            members: 2,
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
}

impl Default for GameConfig {
    fn default() -> Self {
        Self { seed: 42 }
    }
}

pub enum UnitType {
    Scout,
    Excavation,
    Attack,
}

#[derive(Resource)]
pub struct CursorPos(pub Vec2);
impl Default for CursorPos {
    fn default() -> Self {
        // Initialize the cursor pos at some far away place. It will get updated
        // correctly when the cursor moves.
        Self(Vec2::new(-1000.0, -1000.0))
    }
}
