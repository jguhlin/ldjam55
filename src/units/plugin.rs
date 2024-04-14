use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::*;

use crate::*;

pub struct UnitsPlugin<S: States> {
    pub state: S,
}

impl<S: States> Plugin for UnitsPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_event::<AddUnitComplete>()
            // .add_systems(OnEnter(self.state.clone()), setup_units_bar)
            .add_systems(Update, spawn_sprites.run_if(on_event::<AddUnitComplete>()))
            .add_systems(
                PreUpdate,
                (prevent_collision, jitter_units).run_if(in_state(self.state.clone())),
            );
    }
}

fn prevent_collision(mut query: Query<(&mut Transform, &UnitVisual)>) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([a1, mut a2]) = combinations.fetch_next() {
        // Make a buffer of at least 6 between each, width wise
        // and 14 height wise
        // Move at a diagonal

        // If a2 is within 6 of a1, move a2 to the right
        if (a1.0.translation.x - a2.0.translation.x).abs() < 6.0 {
            a2.0.translation.x += 0.1;
        }

        // If a2 is within 14 of a1, move a2 down
        if (a1.0.translation.y - a2.0.translation.y).abs() < 14.0 {
            a2.0.translation.y -= 0.1;
        }
    }
}

fn jitter_units(
    mut query: Query<(&mut Transform, &UnitVisual)>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
) {
    for (mut transform, _) in query.iter_mut() {
        // Don't get more than 20 away from 0,0
        let mut x_movement = rng.gen_range(-0.15..0.15);
        let mut y_movement = rng.gen_range(-0.15..0.15);

        if transform.translation.x.abs() > 20.0 {
            // Change sign so movement is towards 0
            if transform.translation.x > 0.0 {
                x_movement = -0.15;
            } else {
                x_movement = 0.15;
            }
        }
        transform.translation.x += x_movement;

        if transform.translation.y.abs() > 20.0 {
            // Change sign so movement is towards 0
            if transform.translation.y > 0.0 {
                y_movement = -0.15;
            } else {
                y_movement = 0.15;
            }
        }
        transform.translation.y += y_movement;
    }
}

fn spawn_sprites(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut query: Query<(&mut Unit, Entity), With<UnitUninitialized>>,
) {
    for (mut unit, entity) in query.iter_mut() {
        log::info!("Spawning unit");
        let mut transform = Transform::from_translation(Vec3::ZERO);
        commands.entity(entity).remove::<UnitUninitialized>();
        // Add to children
        commands.entity(entity).with_children(|p| {
            for _ in 0..unit.members {
                // Stagger the kids a little
                transform.translation.x += 0.5;
                transform.translation.y += 0.8;
                p.spawn((
                    SpriteSheetBundle {
                        texture: assets.tiles.clone(),
                        atlas: TextureAtlas {
                            layout: assets.tiles_layout.clone(),
                            index: unit.unit_type.index(),
                        },
                        // Place right below the player tower
                        transform,
                        ..default()
                    },
                    UnitVisual,
                ));
            }
        });
    }
}
