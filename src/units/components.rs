use bevy::prelude::{Component, Deref, DerefMut, Entity, Handle, Image, Vec2};
use bevy_ecs_tilemap::prelude::TilePos;

use crate::GameAssets;

#[derive(Component)]
pub struct CanDig;

#[derive(Component)]
pub struct UnitUninitialized;

#[derive(Component)]
pub enum UnitEntry {
    Unavailable,
    Available,
    Summoned(Entity),
}

#[derive(Component, Clone)]
pub struct Unit {
    pub unit_type: UnitType,
    pub health_per_member: u8,
    pub overworld_speed: u8,
    pub excavation_speed: u8,
    pub battle_speed: u8,
    pub members: u8,
    pub visibility: u8, // How much fog of war can be seen

    pub total_health: f32,
    pub current_health: f32,

    pub damage: u8,
    // todo: damage, health regen, attack types, etc...
}

impl Unit {
    pub fn scout() -> Self {
        Self {
            members: 5,
            unit_type: UnitType::Scout,
            health_per_member: 50,
            total_health: 50.0,
            current_health: 50.0,

            overworld_speed: 20,
            excavation_speed: 6,
            battle_speed: 10,

            visibility: 8,
            damage: 4,
        }
    }
    pub fn excavation() -> Self {
        Self {
            members: 7,
            health_per_member: 75,
            total_health: 7. * 75.0,
            current_health: 7. * 75.0,

            unit_type: UnitType::Excavation,

            overworld_speed: 8,
            excavation_speed: 20,
            battle_speed: 4,

            visibility: 3,
            damage: 8,
        }
    }

    // todo: archers, mages, infantry, etc...
    pub fn attack() -> Self {
        Self {
            unit_type: UnitType::Attack,

            members: 20,
            health_per_member: 60,
            total_health: 20. * 60.,
            current_health: 20. * 60.,

            overworld_speed: 10,

            excavation_speed: 2,
            battle_speed: 10,

            visibility: 5,
            damage: 15,
        }
    }
}

#[derive(Copy, Clone)]
pub enum UnitType {
    Scout,
    Excavation,
    Attack,
}

impl std::fmt::Display for UnitType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitType::Scout => write!(f, "Scout"),
            UnitType::Excavation => write!(f, "Excavation"),
            UnitType::Attack => write!(f, "Attack"),
        }
    }
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

#[derive(Component)]
pub struct UnitDirection {
    pub direction: Vec2,
    pub destination: Vec2,
    pub destination_in_tile_pos: TilePos,
}
