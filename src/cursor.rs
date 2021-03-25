use dotrix::{
    assets::{ Wires },
    ecs::{ Const, Mut },
    components::{ Model, WireFrame },
    renderer::{ Transform },
    services::{ Assets, Ray, World },
    terrain::{ Block, Terrain, VoxelMap },
    math::{ Vec3, Vec3i },
};

use crate::editor::Editor;

/// Cursor component
pub struct Cursor {
    pub visible: bool,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            visible: true,
        }
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

pub struct State {
    /// Intersection with terrain
    pub position: Vec3,
    /// Terrain block containing the voxel
    pub block: Vec3i,

    pub cursor_size: f32,
}

/// Cursor tracking system
pub fn track(
    mut editor: Mut<Editor>,
    ray: Const<Ray>,
    terrain: Const<Terrain>,
    world: Const<World>,
) {
    let mut intersection = None;

    let voxel_select = editor.voxel_select;
    let query = world.query::<(&mut Model, &Block)>();
    editor.cursor = None;
    for (model, block) in query {
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

        if let Some((distance_min, distance_max)) = ray.intersect_aligned_box(bounds) {
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

                    if let Some((point, distance)) = binary_search(
                        distance_min,
                        distance_max,
                        &ray,
                        &block,
                        map,
                        0
                    ) {
                        if let Some((_position, saved_distance, _cursor_size)) = intersection {
                            if distance >= saved_distance {
                                continue;
                            }
                        }
                        let (point, cursor_size) = if voxel_select {
                            let voxel_size = block.voxel_size as f32;
                            let voxel_half_size = voxel_size / 2.0;
                            (
                                Vec3::new(
                                    (point.x / voxel_size).floor() * voxel_size + voxel_half_size,
                                    (point.y / voxel_size).floor() * voxel_size + voxel_half_size,
                                    (point.z / voxel_size).floor() * voxel_size + voxel_half_size,
                                ),
                                voxel_half_size
                            )
                        } else {
                            (point, 32.0)
                        };
                        intersection = Some((point, distance, cursor_size));
                        editor.cursor = Some(State {
                            position: point,
                            block: block.position,
                            cursor_size
                        });
                    }
                }
            }
        }
    }

    if let Some((point, _distance, cursor_size)) = intersection {
        let query = world.query::<(&mut WireFrame, &Cursor)>();
        for (wire_frame, _) in query {
            wire_frame.transform.translate = point;
            wire_frame.transform.scale = Vec3::new(cursor_size, cursor_size, cursor_size);
        }
    }
}

fn binary_search(
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


pub fn spawn(assets: &mut Assets, world: &mut World) {

    let wires = assets.store(Wires::cube([0.0; 3]));

    let transform = Transform {
        translate: Vec3::new(0.0, 0.5, 0.0),
        scale: Vec3::new(0.05, 0.05, 0.05),
        ..Default::default()
    };

    world.spawn(
        Some((
            WireFrame { wires, transform, ..Default::default() },
            Cursor::default()
        ))
    );
}

