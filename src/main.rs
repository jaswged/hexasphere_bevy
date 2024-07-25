use std::f32::consts::PI;
use bevy::prelude::*;
use serde::*;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use bevy::render::{
    camera::Camera,
    mesh::Indices,
    RenderPlugin,
    camera::RenderTarget,
    render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, PrimitiveTopology,
    },
    render_asset::RenderAssetUsages,
    settings::{Backends, RenderCreation, WgpuSettings},
};
use bevy::window::WindowTheme;
use serde::ser::SerializeSeq;
use hexasphere::shapes::IcoSphere;
use std::io::Read;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use icosahedron::Polyhedron;
use rand::Rng;
use rehexed::rehexed;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

// region from mouse to tile example
#[derive(Debug, Resource)]
pub struct SelectedTile {
    pub tile: Option<Tile>,
    pub entity: Option<Entity>,
}

impl Default for SelectedTile {
    fn default() -> Self {
        Self {
            tile: None,
            entity: None,
        }
    }
}

// From mouse to tile example
#[derive(Resource, Debug)]
pub struct CursorPos(Vec2);

#[derive(Component)]
struct HighlightedLabel;

#[derive(Component)]
struct TileLabel(Entity);

impl Default for CursorPos {
    fn default() -> Self {
        // Initialize the cursor pos at some far away place. It will get updated
        // correctly when the cursor moves.
        Self(Vec2::new(-1000.0, -1000.0))
    }
}

// We need to keep the cursor position updated based on any `CursorMoved` events.
pub fn update_cursor_pos(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    for cursor_moved in cursor_moved_events.read() {
        // To get the mouse's world position, we have to transform its window position by
        // any transforms on the camera. This is done by projecting the cursor position into
        // camera space (world space).
        for (cam_t, cam) in camera_q.iter() {
            if let Some(pos) = cam.viewport_to_world_2d(cam_t, cursor_moved.position) {
                *cursor_pos = CursorPos(pos);
            }
        }
    }
    // info!("Cursor_pos: {:?}", cursor_pos);
}
// endregion

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::default(),
            brightness: 1000.,
        })
        .init_resource::<CursorPos>()
        .init_resource::<SelectedTile>()
        // .add_plugins(DefaultPlugins) // For macbook
        .add_plugins(DefaultPlugins
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    backends: Some(Backends::VULKAN),
                    ..default()
                }),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Web Hex-".to_string() + env!("CARGO_PKG_VERSION"),
                    resolution: (1280.0, 720.0).into(),
                    resizable: false,
                    window_theme: Some(WindowTheme::Dark),
                    ..Default::default()
                }),
                ..default()
            })
        )
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup)
        .add_systems(First, (update_cursor_pos).chain())
        .add_systems(Update, (muh_update, muh_update_2)) // highlight_tile_labels
        .run();
}

fn setup(mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>,
         asset_server: Res<AssetServer>,
         mut images: ResMut<Assets<Image>>,
         windows: Query<&Window>,
) {
    info!("Setup function called");
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(12.0, 1.5, 8.0)),
            ..default()
        },
        PanOrbitCamera::default(),
    ));

    // Spawn a single hex at the origin of the sphere
    // commands.spawn((
    //     SceneBundle {
    //         scene: asset_server.load("hex.glb#Scene0"),
    //         transform: Transform::from_translation(Vec3::ZERO),
    //         ..default()
    //     },
    // ));

    let mut x = Polyhedron::new_isocahedron(10.0, 1);
    x.compute_triangle_normals();

    // From icosahedron github
    println!("Read in our json file");
    let mut file = File::open("unity.json").expect("file should open read only");
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    // let json_data: serde_json::Value = serde_json::from_str(&data).expect("Unable to parse");
    info!("Parse json to our object");
    let p: Tiles = serde_json::from_str(&data).expect("Json data should conform to our object yo");

    // println!("{:?}", json_data);
    // println!("Object 0: {}\n has [0][1]: {}", json_data["tiles"][0], json_data["tiles"][0][1]);
    println!("Object 0: {:?}\n has [0][1]: {:?}", p.tiles[0], p.tiles[0].center_point);

    let tiles: Vec<Tile> = p.tiles;
    // for (i, tile) in tiles.iter().enumerate() {
    //     info!("Tile {} has center at {:?}", i, tile.center_point);
    // }

    for tile in tiles {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD); //, RenderAssetUsages::new() 13.2

        // points
        let my_points: Vec<Vec3> = tile.boundary.iter().map(|b| Vec3::new(b.x, b.y, b.z)).collect();
        // let my_points = vec![vec_points[x[0]], vec_points[x[1]], vec_points[x[2]], vec_points[x[3]], vec_points[x[4]]];

        // Center comes from tile
        let center = Vec3::new(tile.center_point.x, tile.center_point.y, tile.center_point.z);

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, my_points);
        mesh.insert_indices(Indices::U32(tile.indices));
        let mesh_handle = meshes.add(mesh);

        // Colors randomization
        let mut ranr = rand::thread_rng();
        let random = &ranr.gen_range(0..4);

        // todo material from image
        // let texture_handle = asset_server.load("branding/bevy_logo_dark_big.png");
        // let material_handle = materials.add(StandardMaterial {
        //     base_color_texture: Some(texture_handle.clone()),
        //     alpha_mode: AlphaMode::Blend,
        //     unlit: true,
        //     ..default()
        // });

        // Srgba::hex("#ffd891").unwrap().into(),
        let mut col = match random {
            0 => Color::PINK,
            1 => Color::ORANGE_RED,
            2 => Color::BLUE,
            3 => Color::LIME_GREEN,
            _ => Color::BISQUE,
        };
        // Truly random rainbow colors.
        let mut col = Color::rgb(
            ranr.gen_range(0.0..1.0),
            ranr.gen_range(0.0..1.0),
            ranr.gen_range(0.0..1.0),
        );
        if !tile.is_hex {
            col = Color::BISQUE
        }

        // set metallic quality: https://bevyengine.org/examples/3d-rendering/pbr/
        let metal_mat = materials.add(StandardMaterial {
            base_color: col,
            // vary key PBR parameters on a grid of spheres to show the effect
            metallic: 0.4,
            perceptual_roughness: 0.9,
            ..default()
        });

        // render ui to texture: https://bevyengine.org/examples/ui-user-interface/render-ui-to-texture/

        // todo put ent into an array to hold all tiles? or HashMap<Hex, Entity>

        // todo mesh would be part of our GameComponent bundle?
        //asset_server.load("fonts/FiraCodeNerdFontPropo-Regular.ttf")
        // let ent =
        commands.spawn((
            PbrBundle {
                mesh: mesh_handle,
                // material: materials.add(col),
                material: metal_mat,
                // transform: Transform::from_translation(center),
                ..Default::default()
            },
            Ground,
        )).with_children(|parent| {
            // Add a text label above the mesh
            parent.spawn(Text2dBundle {
                text: Text::from_section(
                    "My Mesh Label",
                    TextStyle {
                        font: asset_server.load("fonts/FiraCodeNerdFontPropo-Regular.ttf"),
                        font_size: 50.0,
                        color: Color::WHITE,
                    },
                    // Default::default(),
                ),
                transform: Transform {
                    translation: Vec3::from(center + Vec3::new(0., 1.0, 0.)), // Vec3::new(0.0, 1.0, 0.0), // Position above the mesh
                    ..Default::default()
                },
                ..Default::default()
            });
        });
        //.id();
    }
}

#[derive(Component)]
struct Ground;

fn muh_update(keyboard_input: Res<ButtonInput<KeyCode>>, cursor_pos: Res<CursorPos>) {
    // cursor position
    // https://bevyengine.org/examples/ui-user-interface/relative-cursor-position/
    if keyboard_input.just_pressed(KeyCode::Space) {
        info!("Space pressed");
        // test out things here

        // print out cursor position
        // Grab the cursor position from the `Res<CursorPos>`
        let cursor_pos: Vec2 = cursor_pos.0;
        // We need to make sure that the cursor's world position is correct relative to the map
        // due to any map transformation.
        println!("Cursor_pos: {:?}", cursor_pos);

        // get and print out all Tile Ids?
    }
}

fn muh_update_2(keyboard_input: Res<ButtonInput<KeyCode>>,
                ground_query: Query<&GlobalTransform, With<Ground>>) {
    // cursor position
    // https://bevyengine.org/examples/ui-user-interface/relative-cursor-position/
    if keyboard_input.just_pressed(KeyCode::Enter) {
        info!("Space pressed");
        // test out things here

        // get and print out all Tile Ids?
        for trans in &ground_query {
            info!("Trans: {:?}", trans);
        }
    }
}

fn draw_cursor(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    ground_query: Query<&GlobalTransform, With<Ground>>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = camera_query.single();
    let ground = ground_query.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    // Calculate a ray pointing from the camera into the world based on the cursor's position.
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Calculate if and where the ray is hitting the ground plane.
    // let Some(distance) =
    // ray.intersect_plane(ground.translation(), plane)
    //     //ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up()))
    // else {
    //     return;
    // };
    // let point = ray.get_point(distance);

    // Draw a circle just above the ground plane at that position.
    // gizmos.circle(point + ground.up() * 0.01, ground.up(), 0.2, Color::WHITE);
}

fn write_to_json_file(polyhedron: Polyhedron, path: &Path) {
    let mut json_file = File::create(path).expect("Can't create file");
    let json = serde_json::to_string(&polyhedron).expect("Problem serializing");
    json_file
        .write_all(json.as_bytes())
        .expect("Can't write to file");
}

// Tile struct wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tiles {
    radius: u32,
    tiles: Vec<Tile>,
}

// Tile struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub guid: u32,
    pub center_point: Point,
    pub is_hex: bool,
    pub boundary: Vec<Point>,
    pub indices: Vec<u32>,
    pub neighbours: Vec<u32>,
}

// Point struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    // pub guid: String,
    // pub position: Vec3,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point {
    pub fn new(guid: String, x: f32, y: f32, z: f32) -> Self {
        // Point {guid, x, y, z }
        Point { x, y, z }
    }
}
