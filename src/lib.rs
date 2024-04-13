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

#[derive(Resource, Default)]
pub struct GameState {
    pub player_tower_location: (u32, u32),
    pub enemy_tower_locations: Vec<(u32, u32)>,
    pub map: NoiseMap,
    pub score: u64,
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
}

impl Default for GameConfig {
    fn default() -> Self {
        Self { seed: 42 }
    }
}
