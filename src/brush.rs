use dotrix::{
    assets::{ Texture, Wires },
    ecs::{ Const, Mut },
    components::{ Model, WireFrame },
    services::{ Assets, Input, Ray, World },
    terrain::{ Block, Terrain, VoxelMap },
    math::{ Vec3 },
};

use crate::{
    controls::Action,
    editor::{ Editor, VoxelPicker },
};

pub fn apply(mut editor: Mut<Editor>, input: Const<Input>, terrain: Mut<Terrain>) {
    if !input.is_action_activated(Action::Brush) {
        return;
    }

    let (cursor_position, block_position) = if let Some(cursor) = editor.cursor.as_ref() {
        (cursor.position, cursor.block)
    } else {
        return;
    };

    if editor.voxel_select {
        if let Some(node) = terrain.octree.load(&block_position) {
            let half_block_size = (node.size / 2) as f32;
            let block_base = Vec3::new(
                block_position.x as f32 - half_block_size,
                block_position.y as f32 - half_block_size,
                block_position.z as f32 - half_block_size,
            );
            let voxel_half_size = half_block_size / 16.0;
            let voxel_base = Vec3::new(
                cursor_position.x - voxel_half_size as f32,
                cursor_position.y - voxel_half_size as f32,
                cursor_position.z - voxel_half_size as f32,
            );
            let voxel_offset = (voxel_base - block_base) / voxel_half_size / 2.0;
            editor.voxel = Some(
                VoxelPicker::new(block_position, voxel_offset, node.payload.as_ref().unwrap())
            );
        }
    }

    println!("Brush: !");
}
