use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;
use bevy::{asset::AssetMetaCheck, text};
use bevy_ecs_tilemap::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiSettings};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::*;
use wyrand;

#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use ldjam55::*;
use rand::SeedableRng;

fn main() {
    let mut app = App::new();
    // .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
    // Normally MSAA 4 but from the template (for web? I suspect) we turn it off
    app
        // Engine-level Resources
        .insert_resource(Msaa::Off)
        .insert_resource(GameConfig::default())
        .insert_resource(AssetMetaCheck::Never)
        .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))
        // Plugins
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "ldjam55".to_string(), // ToDo
                        // Bind to canvas included in `index.html`
                        canvas: Some("#bevy".to_owned()), // From the web template, hopefully this fixes it!
                        // Tells wasm not to override default event handling, like F5 and Ctrl+R
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(DefaultPickingPlugins)
        .insert_resource(DebugPickingMode::Normal)
        .add_plugins(EntropyPlugin::<WyRand>::default())
        .add_plugins(TilemapPlugin)
        // States
        .init_state::<Game>()
        // Game-level resources
        .insert_resource(GameState::default())
        .insert_resource(GameAssets::default())
        .insert_resource(CursorPos::default())
        .insert_resource(SelectedUnit::default())
        // Systems
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, camera_control)
        .add_systems(PreUpdate, update_cursor_pos)
        // Game plugins
        .add_plugins(MapGenerationPlugin)
        .add_plugins(UnitsUiPlugin {
            state: Game::Playing,
        })
        .add_plugins(UnitsPlugin {
            state: Game::Playing,
        })
        .add_plugins(MapInteractionPlugin {
            state: Game::Playing,
        })
        .add_plugins(TreasureGenerationPlugin);

    #[cfg(debug_assertions)]
    {
        // app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        // app.add_plugins(WorldInspectorPlugin::new());
    }

    #[cfg(not(debug_assertions))]
    {
        // app.add_plugins(EguiPlugin);
    }

    app.run();
}

// We need to keep the cursor position updated based on any `CursorMoved` events.
pub fn update_cursor_pos(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    for cursor_moved in cursor_moved_events.read() {
        // To get the mouse's world position, we have to transform its window position by
        // any transforms on the camera. This is done by projecting the cursor position into
        // camera space (world space).
        for (cam_t, cam) in camera_q.iter() {
            if let Some(pos) = cam.viewport_to_world_2d(cam_t, cursor_moved.position) {
                cursor_pos.mouse_position = pos;
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut assets: ResMut<GameAssets>,
    asset_server: Res<AssetServer>,
    mut game: ResMut<NextState<Game>>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture_handle: Handle<Image> = asset_server.load("tiles.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(32.0, 32.0), 12, 1, None, None);
    let texture_atlas_layout = texture_atlases.add(layout);
    assets.tiles = texture_handle;
    assets.tiles_layout = texture_atlas_layout;
    assets.font = asset_server.load("fonts/MonaspaceRadon-Regular.otf");

    assets.icons = Icons {
        tower: asset_server.load("icons/tower.png"),
        x: asset_server.load("icons/x.png"),
        shield: asset_server.load("icons/shield.png"),
        plus: asset_server.load("icons/plus.png"),
        scout: asset_server.load("icons/scout.png"),
        excavator: asset_server.load("icons/excavator.png"),
        attack: asset_server.load("icons/attack.png"),
    };

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    commands.spawn(Camera2dBundle::default());
    // Take u32 twice and convert to u64 from config.seed
    // Using mem::transmute to convert u32 to u64
    let seed: [u32; 2] = [config.seed; 2];
    let seed: u64 = unsafe { std::mem::transmute(seed) };

    commands.insert_resource(GlobalEntropy::new(WyRand::new(wyrand::WyRand::new(seed))));
    log::info!("Setup complete");

    game.set(Game::MapGeneration);
}

fn camera_control(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    for mut transform in query.iter_mut() {
        let mut translation = Vec3::ZERO;
        let speed = 100.0;

        if keyboard_input.pressed(KeyCode::KeyW) {
            translation.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            translation.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            translation.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            translation.x += 1.0;
        }

        // Zoom out
        if keyboard_input.pressed(KeyCode::KeyQ) {
            transform.scale *= 1.0 + time.delta_seconds();
        }

        // Zoom in
        if keyboard_input.pressed(KeyCode::KeyE) {
            transform.scale *= 1.0 - time.delta_seconds();
        }

        if translation.length() > 0.0 {
            translation = translation.normalize();
        }

        transform.translation += translation * speed * time.delta_seconds();
    }
}
