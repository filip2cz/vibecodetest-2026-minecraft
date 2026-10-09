use bevy::prelude::*;
use crate::camera::FlyCamera;
use crate::voxel::BlockType;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(Update, update_ui);
    }
}

#[derive(Component)]
pub struct HotbarSlot {
    pub index: usize,
    pub block_type: BlockType,
}

fn setup_ui(mut commands: Commands) {
    // Hotbar container
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Px(80.0),
        position_type: PositionType::Absolute,
        bottom: Val::Px(0.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        column_gap: Val::Px(10.0),
        ..default()
    }).with_children(|parent| {
        let blocks = [
            BlockType::Grass,
            BlockType::Dirt,
            BlockType::Stone,
            BlockType::Wood,
            BlockType::Glass,
        ];

        for (i, &block_type) in blocks.iter().enumerate() {
            let color = block_type.color();
            parent.spawn((
                Node {
                    width: Val::Px(60.0),
                    height: Val::Px(60.0),
                    border: UiRect::all(Val::Px(4.0)),
                    ..default()
                },
                BorderColor {
                    top: Color::BLACK,
                    right: Color::BLACK,
                    bottom: Color::BLACK,
                    left: Color::BLACK,
                },
                BackgroundColor(Color::srgba(color[0], color[1], color[2], color[3])),
                HotbarSlot { index: i, block_type },
            ));
        }
    });

    // Crosshair
    commands.spawn((
        Node {
            width: Val::Px(4.0),
            height: Val::Px(4.0),
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            margin: UiRect::new(Val::Px(-2.0), Val::Auto, Val::Px(-2.0), Val::Auto),
            ..default()
        },
        BackgroundColor(Color::WHITE),
    ));
}

fn update_ui(
    q_camera: Query<&FlyCamera>,
    mut q_slots: Query<(&mut BorderColor, &HotbarSlot)>,
) {
    let Ok(camera) = q_camera.single() else { return };

    for (mut border, slot) in q_slots.iter_mut() {
        let color = if slot.block_type == camera.selected_block {
            Color::WHITE
        } else {
            Color::BLACK
        };
        border.top = color;
        border.right = color;
        border.bottom = color;
        border.left = color;
    }
}
