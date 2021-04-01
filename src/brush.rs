use dotrix::{
//    assets::{ Texture, Wires },
    ecs::{ Const, Mut },
//    components::{ Model, WireFrame },
    math::{ Point3, Vec3i, MetricSpace },
    services::{ /* Assets,*/ Input, Ray, World },
    terrain::{ Terrain },
};

use crate::{
    controls::Action,
    editor::{ Editor, VoxelPicker },
};

pub struct Brush {
    size: i32,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            size: 1,
        }
    }
}

pub fn apply(
    editor: Const<Editor>,
    input: Const<Input>,
    mut terrain: Mut<Terrain>,
    world: Const<World>,
) {
    if !input.is_action_hold(Action::Brush) {
        return;
    }

    let (cursor_position, _block_position) = if let Some(cursor) = editor.cursor.as_ref() {
        (cursor.position, cursor.block)
    } else {
        return;
    };

    let brush = Brush::default();
    let base = Vec3i::new(
        (cursor_position.x - (brush.size) as f32).ceil() as i32,
        (cursor_position.y - (brush.size) as f32).ceil() as i32,
        (cursor_position.z - (brush.size) as f32).ceil() as i32,
    );
    let size = brush.size as usize * 2 + 1;
    let mut density = terrain.grid.load(base, size)
        .values()
        .expect("Should have some values");

    let size_sq = size * size;
    for x in 0..size {
        let xx = x * size_sq;
        for y in 0..size {
            let xy = xx + size * y;
            for z in 0..size {
                let xyz = xy + z;
                let distance_sq = Point3::new(
                    base.x as f32 + (x) as f32,
                    base.y as f32 + (y) as f32,
                    base.z as f32 + (z) as f32
                ).distance2(
                    Point3::new(cursor_position.x, cursor_position.y, cursor_position.z)
                );

                if distance_sq <= size_sq as f32 {
                    density[xyz] += 4 - 4 * distance_sq as i8 / size_sq as i8;
                }
            }
        }
    }

    terrain.grid.save(base, size, density);
    terrain.changed = true;

}
