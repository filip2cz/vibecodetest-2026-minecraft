use bevy::prelude::*;

mod camera;
mod voxel;
mod ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Vibecraft".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(camera::CameraPlugin)
        .add_plugins(voxel::VoxelPlugin)
        .add_plugins(ui::UiPlugin)
        .add_systems(Startup, setup_light)
        .run();
}

fn setup_light(mut commands: Commands) {
    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
