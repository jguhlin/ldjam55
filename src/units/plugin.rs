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