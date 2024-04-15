use bevy::prelude::*;
use bevy_prng::*;
use bevy_rand::*;
use rand_distr::{Distribution, Poisson};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::*;

#[derive(Debug, Clone)]
pub struct Treasure {
    pub score: u16,
    pub boons: Vec<Boon>, // 1 to 6, weighted towards 1
    pub summon: Option<SummonType>,
    pub slot: u8,
}

#[derive(Debug, EnumIter, Clone)]
pub enum SummonType {
    FireElemental,
    WaterElemental,
    EarthElemental,
    AirElemental,
    GravityElemental,
}

#[derive(Debug, Clone)]
pub struct Boon {
    pub category: BoonType,
    pub operation: BoonOperation,
    pub value: u8,
}

#[derive(EnumIter, Eq, PartialEq, Debug, Clone, Copy)]
pub enum BoonOperation {
    Add,
    Multiply,
}

impl std::fmt::Display for BoonOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoonOperation::Add => write!(f, "+"),
            BoonOperation::Multiply => write!(f, "x"),
        }
    }
}

#[derive(EnumIter, Debug, Clone)]
pub enum BoonType {
    Health,
    Visibility,
    OverworldSpeed,
    ExcavationSpeed,
    BattleSpeed,
    Damage,
    Members,
}

impl BoonType {
    pub fn range_add(&self) -> (u8, u8) {
        match self {
            BoonType::Health => (20, 100),
            BoonType::Visibility => (1, 10),
            BoonType::OverworldSpeed => (1, 10),
            BoonType::ExcavationSpeed => (1, 5),
            BoonType::BattleSpeed => (1, 10),
            BoonType::Damage => (1, 20),
            BoonType::Members => (1, 4),
        }
    }

    pub fn range_multiply(&self) -> (u8, u8) {
        match self {
            BoonType::Health => (2, 4),
            BoonType::Visibility => (2, 3),
            BoonType::OverworldSpeed => (2, 3),
            BoonType::ExcavationSpeed => (2, 3),
            BoonType::BattleSpeed => (2, 4),
            BoonType::Damage => (2, 4),
            BoonType::Members => (2, 4),
        }
    }
}

impl std::fmt::Display for BoonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoonType::Health => write!(f, "Health"),
            BoonType::Visibility => write!(f, "Visibility"),
            BoonType::OverworldSpeed => write!(f, "Overworld Speed"),
            BoonType::ExcavationSpeed => write!(f, "Excavation Speed"),
            BoonType::BattleSpeed => write!(f, "Battle Speed"),
            BoonType::Damage => write!(f, "Damage"),
            BoonType::Members => write!(f, "Members"),
        }
    }
}

pub struct TreasureGenerationPlugin;

impl Plugin for TreasureGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MapClick>()
            .add_systems(OnExit(Game::MapGeneration), fill_treasures);
    }
}

fn fill_treasures(mut treasures: ResMut<TreasureLocs>, mut rng: ResMut<GlobalEntropy<WyRand>>) {
    let mut treasures_vec = vec![];

    let dist = Poisson::new(1.4).unwrap();

    for _ in 0..treasures.locs.len() {
        let mut score = 100; // 100 for finding a treasure

        let boons_count: f32 = dist.sample(&mut *rng);
        // Clamp to 1 to 6
        let boons_count = boons_count.clamp(1.0, 6.0);
        let boons_count = boons_count as u8;

        // Choose a boon
        let mut boons = vec![];
        for _ in 0..boons_count {
            let category = BoonType::iter().choose(&mut *rng).unwrap();

            // Weighted choice, 90% add, 10% multiply
            let operation = if rng.gen_bool(0.9) {
                score += 20;
                BoonOperation::Add
            } else {
                score += 60;
                BoonOperation::Multiply
            };

            let (min, max) = if operation == BoonOperation::Add {
                category.range_add()
            } else {
                category.range_multiply()
            };

            let value = rng.gen_range(min..max);

            boons.push(Boon {
                category,
                operation,
                value,
            });
        }

        // 5% of the time, summon an elemental
        let summon = if rng.gen_bool(0.05) {
            // Remove all boons
            boons.clear();
            score += 200;
            Some(SummonType::iter().choose(&mut *rng).unwrap())
        } else {
            None
        };

        let slot = rng.gen_range(0..8); // Slot 0 is special, it is the defense army

        treasures_vec.push(Treasure {
            score,
            boons,
            summon,
            slot,
        });
    }
    treasures.treasures = treasures_vec;
}
