use bevy::prelude::{Component, Entity, Handle, Image};

use crate::GameAssets;

#[derive(Component)]
pub struct UnitUninitialized;

#[derive(Component)]
pub enum UnitEntry {
    Unavailable,
    Available,
    Summoned(Entity),
}

#[derive(Component)]
pub struct Unit {
    pub unit_type: UnitType,
    pub health_per_unit: u32,
    pub overworld_speed: u32,
    pub excavation_speed: u32,
    pub battle_speed: u32,
    pub members: u32,

    pub total_health: f32,
    // todo: damage, health regen, attack types, etc...
}

impl Unit {
    pub fn scout() -> Self {
        Self {
            unit_type: UnitType::Scout,
            health_per_unit: 50,
            overworld_speed: 20,
            excavation_speed: 10,
            battle_speed: 10,
            members: 1,
            total_health: 50.0,
        }
    }
    pub fn excavation() -> Self {
        Self {
            unit_type: UnitType::Excavation,
            health_per_unit: 75,
            overworld_speed: 10,
            excavation_speed: 20,
            battle_speed: 4,
            members: 2,
            total_health: 150.0,
        }
    }

    // todo: archers, mages, infantry, etc...
    pub fn attack() -> Self {
        Self {
            unit_type: UnitType::Attack,
            health_per_unit: 70,
            overworld_speed: 10,
            excavation_speed: 1,
            battle_speed: 10,
            members: 2,
            total_health: 140.0,
        }
    }
}

pub enum UnitType {
    Scout,
    Excavation,
    Attack,
}

impl UnitType {
    pub fn index(&self) -> usize {
        match self {
            UnitType::Scout => 8,
            UnitType::Excavation => 9,
            UnitType::Attack => 10,
        }
    }

    pub fn icon(&self, assets: &GameAssets) -> Handle<Image> {
        match self {
            UnitType::Scout => assets.icons.scout.clone(),
            UnitType::Excavation => assets.icons.excavator.clone(),
            UnitType::Attack => assets.icons.attack.clone(),
        }
    }
}

#[derive(Component)]
pub struct UnitVisual;
