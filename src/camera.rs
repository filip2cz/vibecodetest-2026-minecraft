use bevy::prelude::*;
use bevy::input::mouse::{AccumulatedMouseMotion, MouseWheel};
use bevy::window::{PrimaryWindow, CursorGrabMode, CursorOptions};
use crate::voxel::VoxelWorld;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, camera_movement);
    }
}

#[derive(Component)]
pub struct FlyCamera {
    pub speed: f32,
    pub sensitivity: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub velocity: Vec3,
    pub on_ground: bool,
    pub selected_block: crate::voxel::BlockType,
    pub interaction_timer: Timer,
}

fn setup_camera(mut commands: Commands) {
    let mut timer = Timer::from_seconds(0.2, TimerMode::Once);
    timer.set_elapsed(std::time::Duration::from_secs_f32(0.2)); // Ready immediately

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(16.0, 40.0, 16.0).looking_at(Vec3::new(16.0, 39.0, 15.0), Vec3::Y),
        FlyCamera {
            speed: 5.0,
            sensitivity: 0.002,
            pitch: 0.0,
            yaw: 0.0,
            velocity: Vec3::ZERO,
            on_ground: false,
            selected_block: crate::voxel::BlockType::Dirt,
            interaction_timer: timer,
        },
    ));
}

fn camera_movement(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut q_windows: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mouse_btn: Res<ButtonInput<MouseButton>>,
    mut q_camera: Query<(&mut Transform, &mut FlyCamera)>,
    mut voxel_world: Option<ResMut<VoxelWorld>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut gizmos: Gizmos,
) {
    let Ok(mut cursor_options) = q_windows.single_mut() else { return };
    
    let mut just_locked = false;
    // Toggle mouse grab with left click / escape
    if mouse_btn.just_pressed(MouseButton::Left) {
        if cursor_options.grab_mode != CursorGrabMode::Locked {
            cursor_options.grab_mode = CursorGrabMode::Locked;
            cursor_options.visible = false;
            just_locked = true;
        }
    }
    if keys.just_pressed(KeyCode::Escape) {
        cursor_options.grab_mode = CursorGrabMode::None;
        cursor_options.visible = true;
    }

    if cursor_options.grab_mode == CursorGrabMode::None {
        return;
    }

    let Ok((mut transform, mut fly_cam)) = q_camera.single_mut() else { return };

    let blocks = [
        crate::voxel::BlockType::Grass,
        crate::voxel::BlockType::Dirt,
        crate::voxel::BlockType::Stone,
        crate::voxel::BlockType::Wood,
        crate::voxel::BlockType::Glass,
    ];
    let mut current_idx = blocks.iter().position(|&b| b == fly_cam.selected_block).unwrap_or(0);

    // Block selection (keyboard)
    if keys.just_pressed(KeyCode::Digit1) { current_idx = 0; }
    if keys.just_pressed(KeyCode::Digit2) { current_idx = 1; }
    if keys.just_pressed(KeyCode::Digit3) { current_idx = 2; }
    if keys.just_pressed(KeyCode::Digit4) { current_idx = 3; }
    if keys.just_pressed(KeyCode::Digit5) { current_idx = 4; }

    // Block selection (scroll wheel)
    for ev in mouse_wheel.read() {
        if ev.y > 0.0 {
            // Scroll UP -> move right
            current_idx = (current_idx + 1) % blocks.len();
        } else if ev.y < 0.0 {
            // Scroll DOWN -> move left
            current_idx = (current_idx + blocks.len() - 1) % blocks.len();
        }
    }
    
    fly_cam.selected_block = blocks[current_idx];

    let delta = mouse_motion.delta;

    fly_cam.yaw -= delta.x * fly_cam.sensitivity;
    fly_cam.pitch -= delta.y * fly_cam.sensitivity;
    
    fly_cam.pitch = fly_cam.pitch.clamp(-1.54, 1.54);

    transform.rotation = Quat::from_axis_angle(Vec3::Y, fly_cam.yaw) * Quat::from_axis_angle(Vec3::X, fly_cam.pitch);

    let mut direction = Vec3::ZERO;
    let mut forward = *transform.forward();
    forward.y = 0.0;
    forward = forward.normalize_or_zero();
    let mut right = *transform.right();
    right.y = 0.0;
    right = right.normalize_or_zero();

    if keys.pressed(KeyCode::KeyW) { direction += forward; }
    if keys.pressed(KeyCode::KeyS) { direction -= forward; }
    if keys.pressed(KeyCode::KeyD) { direction += right; }
    if keys.pressed(KeyCode::KeyA) { direction -= right; }

    if direction.length_squared() > 0.0 {
        direction = direction.normalize();
    }

    let dt = time.delta_secs();
    
    // Physics & Movement
    let speed = fly_cam.speed;
    let target_velocity_x = direction.x * speed;
    let target_velocity_z = direction.z * speed;

    // Apply gravity
    fly_cam.velocity.y -= 25.0 * dt;

    if fly_cam.on_ground && keys.just_pressed(KeyCode::Space) {
        fly_cam.velocity.y = 8.0; // Jump
    }

    fly_cam.velocity.x = target_velocity_x;
    fly_cam.velocity.z = target_velocity_z;

    let Some(world) = &mut voxel_world else { return };

    let mut new_pos = transform.translation;

    // Helper for collision
    // Výška postavy 1.8 bloku (jako v Minecraftu), aby se bezpečně vešla do 2 bloků vysoké mezery
    let player_min = Vec3::new(-0.3, -1.6, -0.3);
    let player_max = Vec3::new(0.3, 0.2, 0.3);
    let is_colliding = |pos: Vec3| -> bool {
        world.is_solid(pos + player_min, pos + player_max)
    };

    // Move Y
    new_pos.y += fly_cam.velocity.y * dt;
    if is_colliding(new_pos) {
        if fly_cam.velocity.y < 0.0 {
            fly_cam.on_ground = true;
        } else {
            fly_cam.on_ground = false; // Hit head
        }
        new_pos.y -= fly_cam.velocity.y * dt;
        fly_cam.velocity.y = 0.0;
    } else {
        fly_cam.on_ground = false;
    }

    // Move X
    new_pos.x += fly_cam.velocity.x * dt;
    if is_colliding(new_pos) {
        new_pos.x -= fly_cam.velocity.x * dt;
    }

    // Move Z
    new_pos.z += fly_cam.velocity.z * dt;
    if is_colliding(new_pos) {
        new_pos.z -= fly_cam.velocity.z * dt;
    }

    // Ensure we don't fall forever if we clip through
    if new_pos.y < -100.0 {
        new_pos = Vec3::new(16.0, 40.0, 16.0);
        fly_cam.velocity = Vec3::ZERO;
    }

    transform.translation = new_pos;

    fly_cam.interaction_timer.tick(time.delta());

    // Interaction
    if !just_locked {
        let left_pressed = mouse_btn.pressed(MouseButton::Left);
        let right_pressed = mouse_btn.pressed(MouseButton::Right);

        if !left_pressed && !right_pressed {
            // Reset timer so next click is instant
            fly_cam.interaction_timer.set_elapsed(std::time::Duration::from_secs_f32(0.2));
        }

        if let Some((hit_pos, normal)) = world.raycast(transform.translation, *transform.forward(), 5.0) {
            // Nakreslení zvýraznění (outline) bloku
            let center = hit_pos + Vec3::new(0.5, 0.5, 0.5);
            gizmos.cube(Transform::from_translation(center).with_scale(Vec3::splat(1.02)), Color::BLACK);

            if (left_pressed || right_pressed) && fly_cam.interaction_timer.is_finished() {
                let mut modified = false;
                if left_pressed {
                    world.set_block(hit_pos.x as i32, hit_pos.y as i32, hit_pos.z as i32, crate::voxel::BlockType::Air);
                    modified = true;
                } else if right_pressed {
                    let place_pos = Vec3::new(
                        (hit_pos.x + normal.x).floor(),
                        (hit_pos.y + normal.y).floor(),
                        (hit_pos.z + normal.z).floor(),
                    );
                    
                    let player_box_min = new_pos + player_min;
                    let player_box_max = new_pos + player_max;
                    let block_min = place_pos;
                    let block_max = place_pos + Vec3::ONE;
                    
                    let intersect_x = player_box_min.x < block_max.x && player_box_max.x > block_min.x;
                    let intersect_y = player_box_min.y < block_max.y && player_box_max.y > block_min.y;
                    let intersect_z = player_box_min.z < block_max.z && player_box_max.z > block_min.z;
                    
                    if !(intersect_x && intersect_y && intersect_z) {
                        world.set_block(place_pos.x as i32, place_pos.y as i32, place_pos.z as i32, fly_cam.selected_block);
                        modified = true;
                    }
                }
                
                if modified {
                    fly_cam.interaction_timer.reset(); // Start cooldown

                    let new_opaque = world.chunk.generate_mesh(true);
                    let new_transparent = world.chunk.generate_mesh(false);
                    
                    let h_opaque = meshes.add(new_opaque);
                    let h_transparent = meshes.add(new_transparent);
                    
                    commands.entity(world.opaque_entity).insert(Mesh3d(h_opaque));
                    commands.entity(world.transparent_entity).insert(Mesh3d(h_transparent));
                }
            }
        }
    }
}
