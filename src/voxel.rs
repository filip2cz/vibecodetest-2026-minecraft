use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::render::mesh::PrimitiveTopology;
use noise::{NoiseFn, Perlin};

pub struct VoxelPlugin;

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_voxels);
    }
}

const CHUNK_SIZE_X: usize = 32;
const CHUNK_SIZE_Y: usize = 32;
const CHUNK_SIZE_Z: usize = 32;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Air,
    Grass,
    Dirt,
    Stone,
    Wood,
    Glass,
}

impl BlockType {
    pub fn color(&self) -> [f32; 4] {
        match self {
            BlockType::Air => [0.0, 0.0, 0.0, 0.0],
            BlockType::Grass => [0.2, 0.8, 0.2, 1.0], // Zelená
            BlockType::Dirt => [0.4, 0.25, 0.1, 1.0],  // Hnědá
            BlockType::Stone => [0.5, 0.5, 0.5, 1.0], // Šedá
            BlockType::Wood => [0.6, 0.4, 0.2, 1.0],  // Světle hnědá
            BlockType::Glass => [0.7, 0.9, 1.0, 0.4], // Poloprůhledná světle modrá
        }
    }

    pub fn is_transparent(&self) -> bool {
        matches!(self, BlockType::Air | BlockType::Glass)
    }
}

pub struct Chunk {
    pub blocks: Box<[[[BlockType; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X]>,
}

#[derive(Resource)]
pub struct VoxelWorld {
    pub chunk: Chunk,
    pub opaque_entity: Entity,
    pub transparent_entity: Entity,
}

impl VoxelWorld {
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> BlockType {
        self.chunk.get_block(x, y, z)
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: BlockType) {
        self.chunk.set_block(x, y, z, block);
    }

    pub fn is_solid(&self, min: Vec3, max: Vec3) -> bool {
        let min_x = min.x.floor() as i32;
        let max_x = (max.x - 0.001).floor() as i32;
        let min_y = min.y.floor() as i32;
        let max_y = (max.y - 0.001).floor() as i32;
        let min_z = min.z.floor() as i32;
        let max_z = (max.z - 0.001).floor() as i32;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    // Glass is solid for collision, Air is not
                    if self.get_block(x, y, z) != BlockType::Air {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn raycast(&self, origin: Vec3, direction: Vec3, max_dist: f32) -> Option<(Vec3, Vec3)> {
        let mut pos = origin;
        let step = 0.05;
        let mut dist = 0.0;
        
        let mut last_voxel = Vec3::new(pos.x.floor(), pos.y.floor(), pos.z.floor());
        
        while dist < max_dist {
            pos += direction * step;
            dist += step;
            
            let current_voxel = Vec3::new(pos.x.floor(), pos.y.floor(), pos.z.floor());
            
            if self.get_block(current_voxel.x as i32, current_voxel.y as i32, current_voxel.z as i32) != BlockType::Air {
                let normal = last_voxel - current_voxel;
                return Some((current_voxel, normal));
            }
            last_voxel = current_voxel;
        }
        None
    }
}

impl Chunk {
    pub fn new() -> Self {
        let perlin = Perlin::new(1);
        let mut blocks = Box::new([[[BlockType::Air; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X]);

        for x in 0..CHUNK_SIZE_X {
            for z in 0..CHUNK_SIZE_Z {
                // scale coordinates for noise
                let nx = x as f64 * 0.1;
                let nz = z as f64 * 0.1;
                let height_val = perlin.get([nx, nz]); // returns -1.0 to 1.0
                let height = ((height_val + 1.0) * 0.5 * (CHUNK_SIZE_Y as f64 * 0.5)) as usize;
                let height = height.clamp(1, CHUNK_SIZE_Y - 1);

                for y in 0..CHUNK_SIZE_Y {
                    if y > height {
                        blocks[x][y][z] = BlockType::Air;
                    } else if y == height {
                        blocks[x][y][z] = BlockType::Grass;
                    } else if y > height.saturating_sub(3) {
                        blocks[x][y][z] = BlockType::Dirt;
                    } else {
                        blocks[x][y][z] = BlockType::Stone;
                    }
                }
            }
        }

        Self { blocks }
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> BlockType {
        if x < 0 || x >= CHUNK_SIZE_X as i32 || 
           y < 0 || y >= CHUNK_SIZE_Y as i32 || 
           z < 0 || z >= CHUNK_SIZE_Z as i32 {
            return BlockType::Air;
        }
        self.blocks[x as usize][y as usize][z as usize]
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: BlockType) {
        if x >= 0 && x < CHUNK_SIZE_X as i32 && 
           y >= 0 && y < CHUNK_SIZE_Y as i32 && 
           z >= 0 && z < CHUNK_SIZE_Z as i32 {
            self.blocks[x as usize][y as usize][z as usize] = block;
        }
    }

    pub fn generate_mesh(&self, opaque: bool) -> Mesh {
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut colors: Vec<[f32; 4]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let mut add_face = |x: f32, y: f32, z: f32, face_idx: usize, b_type: BlockType| {
            // Face definitions (positions, normals)
            // 0: Top (+Y), 1: Bottom (-Y), 2: Right (+X), 3: Left (-X), 4: Front (+Z), 5: Back (-Z)
            let base_idx = positions.len() as u32;

            let s = 1.0; // full block size
            match face_idx {
                0 => { // Top
                    positions.push([x, y + s, z + s]);
                    positions.push([x + s, y + s, z + s]);
                    positions.push([x + s, y + s, z]);
                    positions.push([x, y + s, z]);
                    for _ in 0..4 { normals.push([0.0, 1.0, 0.0]); }
                }
                1 => { // Bottom
                    positions.push([x, y, z]);
                    positions.push([x + s, y, z]);
                    positions.push([x + s, y, z + s]);
                    positions.push([x, y, z + s]);
                    for _ in 0..4 { normals.push([0.0, -1.0, 0.0]); }
                }
                2 => { // Right
                    positions.push([x + s, y, z + s]);
                    positions.push([x + s, y, z]);
                    positions.push([x + s, y + s, z]);
                    positions.push([x + s, y + s, z + s]);
                    for _ in 0..4 { normals.push([1.0, 0.0, 0.0]); }
                }
                3 => { // Left
                    positions.push([x, y, z]);
                    positions.push([x, y, z + s]);
                    positions.push([x, y + s, z + s]);
                    positions.push([x, y + s, z]);
                    for _ in 0..4 { normals.push([-1.0, 0.0, 0.0]); }
                }
                4 => { // Front
                    positions.push([x, y, z + s]);
                    positions.push([x + s, y, z + s]);
                    positions.push([x + s, y + s, z + s]);
                    positions.push([x, y + s, z + s]);
                    for _ in 0..4 { normals.push([0.0, 0.0, 1.0]); }
                }
                5 => { // Back
                    positions.push([x + s, y, z]);
                    positions.push([x, y, z]);
                    positions.push([x, y + s, z]);
                    positions.push([x + s, y + s, z]);
                    for _ in 0..4 { normals.push([0.0, 0.0, -1.0]); }
                }
                _ => {}
            }

            let c = b_type.color();
            for _ in 0..4 { colors.push(c); }

            indices.push(base_idx);
            indices.push(base_idx + 1);
            indices.push(base_idx + 2);
            indices.push(base_idx);
            indices.push(base_idx + 2);
            indices.push(base_idx + 3);
        };

        for x in 0..CHUNK_SIZE_X {
            for y in 0..CHUNK_SIZE_Y {
                for z in 0..CHUNK_SIZE_Z {
                    let current_block = self.blocks[x][y][z];
                    if current_block == BlockType::Air {
                        continue;
                    }
                    
                    if opaque {
                        if current_block.is_transparent() { continue; }
                    } else {
                        if !current_block.is_transparent() { continue; }
                    }

                    let px = x as f32;
                    let py = y as f32;
                    let pz = z as f32;
                    let ix = x as i32;
                    let iy = y as i32;
                    let iz = z as i32;

                    let check_face = |nx, ny, nz| {
                        let neighbor = self.get_block(nx, ny, nz);
                        // Render face if neighbor is transparent (Air or Glass)
                        // If we are Glass, we don't render faces between two Glass blocks
                        neighbor.is_transparent() && !(current_block == BlockType::Glass && neighbor == BlockType::Glass)
                    };

                    if check_face(ix, iy + 1, iz) { add_face(px, py, pz, 0, current_block); }
                    if check_face(ix, iy - 1, iz) { add_face(px, py, pz, 1, current_block); }
                    if check_face(ix + 1, iy, iz) { add_face(px, py, pz, 2, current_block); }
                    if check_face(ix - 1, iy, iz) { add_face(px, py, pz, 3, current_block); }
                    if check_face(ix, iy, iz + 1) { add_face(px, py, pz, 4, current_block); }
                    if check_face(ix, iy, iz - 1) { add_face(px, py, pz, 5, current_block); }
                }
            }
        }

        if indices.is_empty() {
            // Fix for Bevy empty mesh allocator panic (Use-after-free)
            for _ in 0..3 {
                positions.push([0.0, 0.0, 0.0]);
                normals.push([0.0, 1.0, 0.0]);
                colors.push([0.0, 0.0, 0.0, 0.0]);
            }
            indices.push(0);
            indices.push(1);
            indices.push(2);
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
        mesh
    }
}

fn setup_voxels(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let chunk = Chunk::new();
    
    let opaque_mesh = chunk.generate_mesh(true);
    let transparent_mesh = chunk.generate_mesh(false);
    
    let opaque_mesh_handle = meshes.add(opaque_mesh);
    let transparent_mesh_handle = meshes.add(transparent_mesh);

    // Opaque Mesh
    let opaque_entity = commands.spawn((
        Mesh3d(opaque_mesh_handle),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            alpha_mode: AlphaMode::Opaque,
            ..default()
        })),
        Transform::default(),
    )).id();

    // Transparent Mesh
    let transparent_entity = commands.spawn((
        Mesh3d(transparent_mesh_handle),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::default(),
    )).id();

    commands.insert_resource(VoxelWorld { chunk, opaque_entity, transparent_entity });
}
