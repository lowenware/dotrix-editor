use dotrix::{
    assets::{ Wires },
    components::{ SimpleLight },
    ecs::{ Mut, Const },
    egui::{
        Egui,
        CollapsingHeader,
        DragValue,
        Grid,
        Label,
        TopPanel,
        Separator,
        Slider,
        Window
    },
    math::{ Vec3, Vec3i },
    input::{ Button, State as InputState, Mapper, KeyCode },
    services::{ Assets, Camera, Frame, Input, World, Ray, Renderer },
    terrain::{ Terrain, Voxel, VoxelMap },
};

use crate::{
    controls,
    cursor,
};

use noise::{ Fbm, MultiFractal };
use std::f32::consts::PI;

pub struct VoxelPicker {
    /// Terrain block containing the voxel
    parent: Vec3i,
    /// Voxel index in the Voxel Map
    index: Vec3i,
    /// Voxel density values
    values: [f32; 8],
}

impl VoxelPicker {
    pub fn new(parent: Vec3i, index: Vec3, map: &VoxelMap) -> Self {
        let x = index.x as usize;
        let y = index.y as usize;
        let z = index.z as usize;
        let index = Vec3i::new(x as i32, y as i32, z as i32);
        let values = [
            map.density[x][y][z],
            map.density[x + 1][y][z],
            map.density[x + 1][y][z + 1],
            map.density[x][y][z + 1],
            map.density[x][y + 1][z],
            map.density[x + 1][y + 1][z],
            map.density[x + 1][y + 1][z + 1],
            map.density[x][y + 1][z + 1],
        ];
        Self {
            parent,
            index,
            values,
        }
    }
}

pub struct Editor {
    pub sea_level: u8,
    pub terrain_size: usize,
    pub terrain_size_changed: bool,
    pub noise_octaves: usize,
    pub noise_frequency: f64,
    pub noise_lacunarity: f64,
    pub noise_persistence: f64,
    pub noise_scale: f64,
    pub noise_amplitude: f64,
    pub show_toolbox: bool,
    pub show_info: bool,
    pub brush_x: f32,
    pub brush_y: f32,
    pub brush_z: f32,
    pub brush_radius: f32,
    pub brush_add: bool,
    pub brush_sub: bool,
    pub brush_changed: bool,
    pub apply_noise: bool,
    pub lod: usize,
    pub cursor: Option<cursor::State>,
    pub voxel_select: bool,
    pub voxel: Option<VoxelPicker>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            sea_level: 0,
            terrain_size: 64,
            terrain_size_changed: true,
            noise_octaves: 8,
            noise_frequency: 1.1,
            noise_lacunarity: 4.5,
            noise_persistence: 0.1,
            noise_scale: 512.0,
            noise_amplitude: 512.0,
            show_toolbox: false,
            show_info: true,
            brush_x: 0.0,
            brush_y: 10.0,
            brush_z: 0.0,
            brush_radius: 5.0,
            brush_add: false,
            brush_sub: false,
            brush_changed: false,
            apply_noise: true,
            lod: 2,
            cursor: None,
            voxel_select: true,
            voxel: None,
        }
    }

    pub fn noise(&self) -> Fbm {
        let noise = Fbm::new();
        let noise = noise.set_octaves(self.noise_octaves);
        let noise = noise.set_frequency(self.noise_frequency);
        let noise = noise.set_lacunarity(self.noise_lacunarity);
        noise.set_persistence(self.noise_persistence)
    }
}

pub fn ui(
    mut editor: Mut<Editor>,
    renderer: Mut<Renderer>,
    mut terrain: Mut<Terrain>,
    camera: Const<Camera>,
    frame: Const<Frame>,
    ray: Const<Ray>,
) {
    let egui = renderer.overlay_provider::<Egui>()
        .expect("Renderer does not contain an Overlay instance");

    TopPanel::top("side_panel").show(&egui.ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("🗋").clicked { println!("New"); }
            if ui.button("🖴").clicked { println!("Save"); }
            if ui.button("🗁").clicked { println!("Open"); }
            if ui.button("🛠").clicked { editor.show_toolbox = !editor.show_toolbox; }
            if ui.button("ℹ").clicked { editor.show_info = !editor.show_info; }
        });
    });

    let mut show_window = editor.show_toolbox;

    Window::new("Toolbox").open(&mut show_window).show(&egui.ctx, |ui| {

        CollapsingHeader::new("View").default_open(true).show(ui, |ui| {
            ui.add(Label::new("LOD"));
            ui.add(Slider::usize(&mut editor.lod, 1..=16).text("level"));
        });

        CollapsingHeader::new("Terrain")
            .default_open(true)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.add(Label::new("Size:"));
                        if ui.button("Resize").clicked { editor.terrain_size_changed = true; }
                    });
                    ui.add(Slider::usize(&mut editor.terrain_size, 8..=256).text("Meters"));

                    ui.add(Separator::new());

                    ui.add(Label::new("Brush:"));
                    if let Some(voxel) = editor.voxel.as_mut() {
                        ui.horizontal(|ui| {
                            ui.add(DragValue::f32(&mut voxel.values[4]));
                            ui.add(DragValue::f32(&mut voxel.values[5]));
                            ui.add(DragValue::f32(&mut voxel.values[6]));
                            ui.add(DragValue::f32(&mut voxel.values[7]));
                        });
                        ui.horizontal(|ui| {
                            ui.add(DragValue::f32(&mut voxel.values[0]));
                            ui.add(DragValue::f32(&mut voxel.values[1]));
                            ui.add(DragValue::f32(&mut voxel.values[2]));
                            ui.add(DragValue::f32(&mut voxel.values[3]));
                        });
                        if let Some(voxel) = editor.voxel.as_ref() {
                            if ui.button("Apply").clicked {
                                if let Some(node) = terrain.octree.load_mut(&voxel.parent) {
                                    if let Some(map) = node.payload.as_mut() {
                                        let x = voxel.index.x as usize;
                                        let y = voxel.index.y as usize;
                                        let z = voxel.index.z as usize;
                                        map.density[x][y][z] = voxel.values[0];
                                        map.density[x + 1][y][z] = voxel.values[1];
                                        map.density[x + 1][y][z + 1] = voxel.values[2];
                                        map.density[x][y][z + 1] = voxel.values[3];
                                        map.density[x][y + 1][z] = voxel.values[4];
                                        map.density[x + 1][y + 1][z] = voxel.values[5];
                                        map.density[x + 1][y + 1][z + 1] = voxel.values[6];
                                        map.density[x][y + 1][z + 1] = voxel.values[7];
                                    }
                                }
                                terrain.changed = true;
                            }
                        }
                    }

                    ui.add(Separator::new());

                    ui.add(Label::new("Noise:"));
                    ui.add(Slider::f64(&mut editor.noise_scale, 1.0..=256.0).text("Scale"));
                    ui.add(Slider::f64(&mut editor.noise_amplitude, 1.0..=256.0).text("Amplitude"));
                    ui.add(Slider::usize(&mut editor.noise_octaves, 1..=10).text("Octaves"));
                    ui.add(Slider::f64(&mut editor.noise_frequency, 0.1..=10.0).text("Frequency"));
                    ui.add(Slider::f64(&mut editor.noise_lacunarity, 0.1..=10.0).text("Lacunarity"));
                    ui.add(Slider::f64(&mut editor.noise_persistence, 0.1..=10.0).text("Persistence"));
                    if ui.button("Apply").clicked {
                        terrain.populate(
                            &editor.noise(),
                            editor.noise_amplitude,
                            editor.noise_scale
                        );
                    }
                });
            });
    });

    editor.show_toolbox = show_window;

    let mut show_window = editor.show_info;

    Window::new("Info").open(&mut show_window).show(&egui.ctx, |ui| {
        let grid = Grid::new("info")
            .striped(true)
            .spacing([40.0, 4.0]);

        grid.show(ui, |ui| {
            ui.label("FPS");
            ui.label(format!("{}", frame.fps()));
            ui.end_row();

            let vec = ray.origin.as_ref()
                .map(|v| format!("x: {:.4}, y: {:.4}, z: {:.4}", v.x, v.y, v.z));

            ui.label("Camera Position");
            ui.label(vec.as_deref().unwrap_or("-"));
            ui.end_row();

            let vec = format!("x: {:.4}, y: {:.4}, z: {:.4}",
                camera.target.x, camera.target.y, camera.target.z);

            ui.label("Camera Target");
            ui.label(vec);
            ui.end_row();

            let vec = ray.direction.as_ref()
                .map(|v| format!("x: {:.4}, y: {:.4}, z: {:.4}", v.x, v.y, v.z));

            ui.label("Mouse Ray");
            ui.label(vec.as_deref().unwrap_or("-"));
            ui.end_row();

            ui.label("Generated in");
            ui.label(format!("{} us", terrain.generated_in.as_micros()));
            ui.end_row();


            let vec = editor.cursor.as_ref()
                .map(|v| format!("x: {:.4}, y: {:.4}, z: {:.4}",
                        v.position.x, v.position.y, v.position.z));
            ui.label("Cursor");
            ui.label(vec.as_deref().unwrap_or("None"));
            ui.end_row();

            if let Some(voxel) = editor.voxel.as_ref() {
                ui.label("Voxel Parent");
                let vec = format!("x: {:.4}, y: {:.4}, z: {:.4}",
                        voxel.parent.x, voxel.parent.y, voxel.parent.z);
                ui.label(vec);
                ui.end_row();
                ui.label("Voxel Index");
                let vec = format!("x: {:.4}, y: {:.4}, z: {:.4}",
                        voxel.index.x, voxel.index.y, voxel.index.z);
                ui.label(vec);
            }
        });
    });

    editor.show_info = show_window;
}

const ROTATE_SPEED: f32 = PI / 10.0;
const ZOOM_SPEED: f32 = 10.0;
const MOVE_SPEED: f32 = 64.0;

pub fn startup(
    mut assets: Mut<Assets>,
    editor: Const<Editor>,
    mut input: Mut<Input>,
    mut renderer: Mut<Renderer>,
    mut terrain: Mut<Terrain>,
    mut world: Mut<World>,
) {
    assets.import("assets/terrain.png");
    renderer.add_overlay(Box::new(Egui::default()));

    world.spawn(Some((SimpleLight{
        position: Vec3::new(0.0, 756.0, 0.0), ..Default::default()
    },)));

    controls::init(&mut input);

    assets.store_as(Wires::cube([0.4; 3]), "wires_gray");

    cursor::spawn(&mut assets, &mut world);

    terrain.populate(&editor.noise(), editor.noise_amplitude, editor.noise_scale);
}

pub fn camera_control(
    mut camera: Mut<Camera>,
    input: Const<Input>,
    frame: Const<Frame>,
    // world: Const<World>,
) {
    let time_delta = frame.delta().as_secs_f32();
    let mouse_delta = input.mouse_delta();
    let mouse_scroll = input.mouse_scroll();

    let distance = camera.distance - ZOOM_SPEED * mouse_scroll * time_delta;
    camera.distance = if distance > -1.0 { distance } else { -1.0 };

    if input.button_state(Button::MouseRight) == Some(InputState::Hold) {
        camera.y_angle += mouse_delta.x * ROTATE_SPEED * time_delta;

        let xz_angle = camera.xz_angle + mouse_delta.y * ROTATE_SPEED * time_delta;
        let half_pi = PI / 2.0;

        camera.xz_angle = if xz_angle >= half_pi {
            half_pi - 0.01
        } else if xz_angle <= -half_pi {
            -half_pi + 0.01
        } else {
            xz_angle
        };
    }

    // move
    let distance = if input.is_action_hold(controls::Action::Move) {
        MOVE_SPEED * frame.delta().as_secs_f32()
    } else {
        0.0
    };

    if distance > 0.00001 {
        let y_angle = camera.y_angle;

        let dx = distance * y_angle.cos();
        let dz = distance * y_angle.sin();

        camera.target.x -= dx;
        camera.target.z -= dz;
    }

    camera.set_view();
}
