use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_ecs_tilemap::prelude::*;

use crate::*;

#[derive(Event)]
pub struct AddUnitEvent;

pub struct MapInteractionPlugin<S: States> {
    pub state: S,
}

#[derive(Event)]
pub struct MapClick(pub Vec2);

#[derive(Event, Deref, DerefMut)]
pub struct CenterCamera {
    pub loc: TilePos,
}

impl<S: States> Plugin for MapInteractionPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_event::<CenterCamera>()
            .add_event::<MapClick>()
            .add_systems(PreUpdate, map_click.run_if(in_state(self.state.clone())))
            .add_systems(PreUpdate, center_camera.run_if(on_event::<CenterCamera>()));
    }
}

fn center_camera(
    mut query: Query<&mut Transform, With<Camera>>,
    mut ev_centercamera: EventReader<CenterCamera>,
) {
    let mut camera_transform = query.single_mut();
    let event = ev_centercamera.read().next().unwrap();

    // Map is centered at 0, 0 and size is 1000, 1000
    // tile sizes are 32x32

    let player_tower_x = event.x;
    let player_tower_y = event.y;

    let x = player_tower_x as f32 * 32.0;
    let y = player_tower_y as f32 * 32.0;
    // Map is centered, so subtract
    camera_transform.translation = Vec3::new(x - 500.0 * 32.0, y - 500.0 * 32.0, 10.0);
}

fn map_click(
    mut ev_map_click: EventWriter<MapClick>,
    mut cursor: ResMut<CursorPos>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &Transform,
    )>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = q_windows.single();
    if let Some(position) = q_windows.single().cursor_position() {
        // Get window size
        let window_size = Vec2::new(window.width(), window.height());
        // Get window position
        let window_position = Vec2::new(position.x, position.y);

        // Bottom 80px is UI, so if we are there, return
        if window_position.y > window_size.y - 80.0 {
            return;
        }
    } else {
        return; // Not in this window
    }

    for (map_size, grid_size, map_type, tile_storage, map_transform) in tilemap_q.iter() {
        // Grab the cursor position from the `Res<CursorPos>`
        let cursor_pos: Vec2 = cursor.mouse_position;

        // Get window coordinates

        // We need to make sure that the cursor's world position is correct relative to the map
        // due to any map transformation.
        let cursor_in_map_pos: Vec2 = {
            // Extend the cursor_pos vec3 by 0.0 and 1.0
            let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xy()
        };
        // Once we have a world position we can transform it into a possible tile position.
        if let Some(tile_pos) =
            TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type)
        {
            if mouse_button_input.just_pressed(MouseButton::Left) {
                log::info!("Map click at: {:?}", tile_pos);
                ev_map_click.send(MapClick(cursor_in_map_pos));
            }
            cursor.tile_position = tile_pos;
            cursor.tile_position_real = cursor.mouse_position;
            return;
        }
    }
}
