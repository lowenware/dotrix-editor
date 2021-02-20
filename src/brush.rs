use dotrix::{
    ecs::{ Const, Mut },
    components::{ Model, WireFrame },
    services::{ Assets, Ray, World },
    terrain::{ Block },
    math::{ Vec3 },
};

use crate::editor::Editor;

pub fn picker(
    mut editor: Mut<Editor>,
    assets: Const<Assets>,
    ray: Const<Ray>,
    world: Const<World>,
) {

    let gray = assets.find("terrain").unwrap();
    let red = assets.find("red").unwrap();

    let wires_gray = assets.find("wires_gray").unwrap();
    let wires_red = assets.find("wires_red").unwrap();

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
        let (texture, wires) = if let Some((t_min, t_max)) = ray.intersect_box(bounds) {
            editor.picked_block = Some(block.position);
            // println!("Intersects in {:?} and {:?}", t_min, t_max);
            (red, wires_red)
        } else {
            (gray, wires_gray)
        };

        if model.texture != texture {
            model.texture = texture;
            model.buffers = None;
            wire_frame.wires = wires;
        }
    }

}
