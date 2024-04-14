use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::*;

#[derive(Event)]
pub struct AddUnitEvent;

pub struct MapInteractionPlugin<S: States> {
    pub state: S,
}

#[derive(Event)]
pub struct MapClick(pub Vec2);

impl<S: States> Plugin for MapInteractionPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_event::<MapClick>()
            .add_systems(
                PreUpdate,
                map_click.run_if(in_state(self.state.clone()))
            );

    }
}

fn map_click(
    mut ev_map_click: EventWriter<MapClick>,
    cursor_pos: Res<CursorPos>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &Transform,
    )>,
) {
    for (map_size, grid_size, map_type, tile_storage, map_transform) in tilemap_q.iter() {
        // Grab the cursor position from the `Res<CursorPos>`
        let cursor_pos: Vec2 = cursor_pos.0;
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
            log::info!("Tile Pos: {:?}", tile_pos);
        }
    }

}
