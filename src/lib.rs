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

pub use map::*;
pub use ui::*;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Game {
    #[default]
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

pub struct Unit {
    pub health: u32,
    pub overworld_speed: u32,
    pub excavation_speed: u32,
    pub battle_speed: u32,
    // todo: damage, health regen, attack types, etc...
}

#[derive(Resource, Default)]
pub struct GameAssets {
    pub tiles: Handle<Image>,
    pub icons: Icons,
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
