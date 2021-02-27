use dotrix::{
    assets::{ Texture, Wires },
    ecs::{ Const, Mut },
    components::{ Model, WireFrame },
    services::{ Assets, Ray, World },
    terrain::{ Block, Terrain, VoxelMap },
    math::{ Vec3 },
};

use super::editor::Cursor;

use crate::editor::Editor;

// TODO: add treshhold 
pub fn picker(
    mut editor: Mut<Editor>,
    assets: Const<Assets>,
    ray: Const<Ray>,
    terrain: Const<Terrain>,
    world: Const<World>,
) {
    let gray = assets.find::<Texture>("terrain").unwrap();
    let red = assets.find::<Texture>("red").unwrap();

    let wires_gray = assets.find::<Wires>("wires_gray").unwrap();
    let wires_red = assets.find::<Wires>("wires_red").unwrap();

    let mut cursor_position = None;

    let query = world.query::<(&mut Model, &mut WireFrame, &Block)>();
    editor.picked_block = None;
    for (model, wire_frame, block) in query {
        if model.disabled { continue; }
        let bounds = [
            Vec3::new(
                block.bound_min.x as f32,
                block.bound_min.y as f32,
                block.bound_min.z as f32
            ),
            Vec3::new(
                block.bound_max.x as f32,
                block.bound_max.y as f32,
                block.bound_max.z as f32
            ),
        ];
        let mut texture = gray;
        let mut wires = wires_gray;

        if let Some((distance_min, distance_max)) = ray.intersect_aligned_box(bounds) {
            editor.picked_block = None;
            if let Some(node) = terrain.octree.load(&block.position) {
                if let Some(map) = node.payload.as_ref() {
                    /* println!(
                        "Intersects {:?} - {:?}\n   in {:?} and {:?}\n  origin: {:?}",
                        block.bound_min,
                        block.bound_max,
                        distance_min * ray.direction.unwrap() + ray.origin.unwrap(),
                        distance_max * ray.direction.unwrap() + ray.origin.unwrap(),
                        ray.origin.unwrap()
                    ); */

                    if let Some((point, _distance)) = binary_search(
                        distance_min,
                        distance_max,
                        &ray,
                        &block,
                        map,
                        0
                    ) {
                        texture = red;
                        wires = wires_red;
                        editor.picked_block = Some(block.position);
                        cursor_position = Some(point);
                        // println!("---> {:?}, {}\n",point, distance);
                    }

                }
            }
            // println!("Intersects in {:?} and {:?}", t_min, t_max);
        }

        if model.texture != texture {
            model.texture = texture;
            model.buffers = None;
            wire_frame.wires = wires;
        }
    }

    if let Some(cursor_position) = cursor_position {
        let query = world.query::<(&mut WireFrame, &Cursor)>();
        for (wire_frame, _) in query {
            wire_frame.transform.translate = cursor_position;
            wire_frame.transform.scale = Vec3::new(32.0, 32.0, 32.0);
        }
    }
}

pub fn binary_search(
    distance_min: f32,
    distance_max: f32,
    ray: &Ray,
    block: &Block,
    map: &VoxelMap,
    count: usize,
) -> Option<(Vec3, f32)> {
    let ray_direction = ray.direction.unwrap();
    let ray_origin = ray.origin.unwrap();
    let distance = distance_min + (distance_max - distance_min) / 2.0;
    let offset = Vec3::new(
        block.bound_min.x as f32,
        block.bound_min.y as f32,
        block.bound_min.z as f32,
    );
    let point = ray_direction * distance + ray_origin;
    let value = map.value(block.voxel_size, &(point - offset))
        .expect("ray cast has to be inside of the block");

    if value.abs() < 0.001 {
        return Some((point, distance));
    }

    if count == 200 {
        return None;
    }

    if value < 0.0 {
        binary_search(distance, distance_max, ray, block, map, count + 1)
    } else {
        binary_search(distance_min, distance, ray, block, map, count + 1)
    }
}
