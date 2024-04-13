#![allow(clippy::type_complexity)]
#![feature(trivial_bounds)]

use bevy::prelude::*;
use bevy_prng::Xoroshiro64StarStar;
use bevy_rand::prelude::{EntropyComponent, EntropyPlugin, ForkableRng, GlobalEntropy};
use rand::prelude::{IteratorRandom, Rng};

pub mod map;

pub use map::*;

#[derive(Resource)]
pub struct GameConfig {
    pub seed: u32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self { seed: 42 }
    }
}
