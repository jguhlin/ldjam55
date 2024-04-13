use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiSettings};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::prelude::*;
use bevy_prng::ChaCha8Rng;
use bevy_rand::prelude::*;
use bevy::asset::AssetMetaCheck;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;

#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use ldjam55::*;

fn main() {
    let mut app = App::new();
    // .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
    // Normally MSAA 4 but from the template (for web? I suspect) we turn it off
    app.insert_resource(Msaa::Off)
        .insert_resource(AssetMetaCheck::Never)
        .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy game".to_string(), // ToDo
                // Bind to canvas included in `index.html`
                canvas: Some("#bevy".to_owned()), // From the web template, hopefully this fixes it!
                // Tells wasm not to override default event handling, like F5 and Ctrl+R
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        // Default plugin
        .add_systems(Startup, setup)
        .add_plugins(MapGenerationPlugin)
        .init_state::<GameState>()
        .add_systems(PreUpdate, camera_control);

    #[cfg(debug_assertions)]
    app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));

    // Enable only for development
    #[cfg(debug_assertions)]
    {
        app.add_plugins(WorldInspectorPlugin::new());
    }

    #[cfg(not(debug_assertions))]
    {
        app.add_plugins(EguiPlugin);
    }

    app.run();
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    MapGeneration,
    Loading, // Here, but dunno if we need it
    Playing,
    Menu,
    Paused,
}

fn setup(mut commands: Commands) {
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