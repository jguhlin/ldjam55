#![allow(clippy::type_complexity)]
#![feature(trivial_bounds)]

use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::{EntropyComponent, EntropyPlugin, ForkableRng, GlobalEntropy};
use noise::utils::NoiseMap;
use rand::prelude::{IteratorRandom, Rng};

pub mod map;

pub use map::*;

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

#[derive(Resource, Default)]
pub struct GameState {
    pub player_tower_location: (u64, u64),
    pub enemy_tower_locations: Vec<(u64, u64)>,
    pub map: NoiseMap,
}

#[derive(Resource, Default)]
pub struct GameAssets {
    pub tiles: Handle<Image>,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self { seed: 42 }
    }
}
