use bevy::prelude::*;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use bevy::render::{
    camera::Camera,
    mesh::Indices,
    RenderPlugin,
    render_resource::PrimitiveTopology,
    render_asset::RenderAssetUsages,
    settings::{Backends, RenderCreation, WgpuSettings},
};
use bevy::window::WindowTheme;
use std::io::Read;
use std::slice::Windows;
use bevy::math::bounding::{RayCast2d, RayCast3d};
use bevy::render::mesh::VertexAttributeValues;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use rand::Rng;
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
        // 3rd party plugins
        .add_plugins(PanOrbitCameraPlugin)
        // Our plugins
        .add_systems(Startup, setup)
        .add_systems(First, (update_cursor_pos).chain())
        .add_systems(Update, (muh_update, muh_update_2))
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
            // transform: Transform::from_translation(Vec3::new(12.0, 12.5, 8.0)).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        // todo prevent right click moving camera in PanOrbitCamera
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

    let mut map = BTreeMap::new();

    // Unity has 92 tiles: [ Unity_4:162, Unity_6:362, Unity_9:812, Unity_10:1002, Unity_20:4002 ]
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

    for tile in p.tiles {  // Vec<Tile>
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD); //, RenderAssetUsages::new() 13.2

        // points
        let my_points: Vec<Vec3> = tile.boundary.iter().map(|b| Vec3::new(b.x, b.y, b.z)).collect();
        // let my_points = vec![vec_points[x[0]], vec_points[x[1]], vec_points[x[2]], vec_points[x[3]], vec_points[x[4]]];

        // Center comes from tile
        let center = Vec3::new(tile.center_point.x, tile.center_point.y, tile.center_point.z);

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, my_points);
        mesh.insert_indices(Indices::U32(tile.indices));

        // Uvs came from Chat Gpt
        let hex_uvs = if tile.is_hex {
            vec![
                [1.0, 0.5],
                [0.75, 0.9330127],
                [0.24999997, 0.9330127],
                [0.0, 0.49999997],
                [0.25000006, 0.066987276],
                [0.74999994, 0.066987276]
            ]
        } else {
            vec![
                [1.0, 0.5],
                [0.6545085, 0.97552824],
                [0.09549147, 0.7938926],
                [0.09549153, 0.20610732],
                [0.6545086, 0.02447176]
            ]
        };

        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, hex_uvs);
        let mesh_handle = meshes.add(mesh);

        // Colors randomization
        let mut ranr = rand::thread_rng();
        let random = &ranr.gen_range(0..4);

        // Srgba::hex("#ffd891").unwrap().into(),
        // let mut col = match random {
        //     0 => Color::WhPINK,
        //     1 => Color::ORANGE_RED,
        //     2 => Color::BLUE,
        //     3 => Color::LIME_GREEN,
        //     _ => Color::BISQUE,
        // };
        // Truly random rainbow colors.
        let mut col = Color::rgb(
            ranr.gen_range(0.0..1.0),
            ranr.gen_range(0.0..1.0),
            ranr.gen_range(0.0..1.0),
        );
        if !tile.is_hex {
            col = Color::BLACK;
        }

        // set metallic quality: https://bevyengine.org/examples/3d-rendering/pbr/
        let metal_mat = materials.add(StandardMaterial {
            base_color: col,
            // vary key PBR parameters on a grid of spheres to show the effect
            metallic: 0.4,
            perceptual_roughness: 0.9,
            ..default()
        });

        // Import the custom texture with the number on it.
        let img_path = format!("num_textures/{}.png", tile.guid);
        // let img_handle: Handle<Image> = asset_server.load(img_path);
        // let img_handle = asset_server.load("0.png");

        let material_handle = materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load(img_path)), // img_handle.clone()),
            // alpha_mode: AlphaMode::Blend,
            // unlit: true,
            ..default()
        });
        // let stand_mat = materials.add(StandardMaterial {
        //     base_color_texture: Some(img_handle),
        //     ..default()
        // });
        // let mat = asset_server.add(stand_mat);

        // render ui to texture: https://bevyengine.org/examples/ui-user-interface/render-ui-to-texture/

        // todo mesh would be part of our GameComponent bundle?
        // Create a Tile bundle to spawn that has its own pbrBundle as a field
        // This struct also has the tile id and type of hex/Biome.
        let tile_id = commands.spawn((TileBundle {
            mesh: PbrBundle {
                mesh: mesh_handle,
                material: material_handle,
                // material: materials.add(col),
                // material: metal_mat,
                // transform: Transform::from_translation(center),
                ..Default::default()
            },
            biome: Biome::new("col".to_string()),
            id: TileId::new(tile.guid),
            tile_obj: TileObj::new(tile.guid, "col".to_string()),
        },
          Ground,
          // TileObj::new(tile.guid, "col".to_string()),
         ),
        ).id();

        // todo put ent into an array to hold all tiles? or HashMap<Hex, Entity>
        // insert entity id into a hashmap of all tiles?
        map.insert(tile.guid, tile_id);

        // info!("Spawned tile {} with entity id {}", tile.guid, tile_id.index());
    }
    // info!("Print out the hashmap of all tiles: {:?}", map);
}

#[derive(Component, Debug)]
pub struct Biome {
    pub value: String,
}

impl Biome {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

#[derive(Component, Debug)]
pub struct TileId {
    pub value: u32,
}

impl TileId {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

#[derive(Bundle)]
pub struct TileBundle {
    pub mesh: PbrBundle,
    pub biome: Biome,
    pub id: TileId,
    pub tile_obj: TileObj,
}

#[derive(Component)]
struct Ground;

fn muh_update(keyboard_input: Res<ButtonInput<KeyCode>>, cursor_pos1: Res<CursorPos>) {
    // cursor position
    // https://bevyengine.org/examples/ui-user-interface/relative-cursor-position/
    if keyboard_input.just_pressed(KeyCode::Space) {
        info!("Space pressed");

        // print out cursor position
        // Grab the cursor position from the `Res<CursorPos>`
        let cursor_pos2: Vec2 = cursor_pos1.0;
        // We need to make sure that the cursor's world position is correct relative to the map
        // due to any map transformation.
        println!("Cursor_pos1: {:?}", cursor_pos1);
        println!("Cursor_pos2: {:?}", cursor_pos2);

        // get and print out all Tile Ids?
    }
}

fn muh_update_2(keyboard_input: Res<ButtonInput<KeyCode>>,
                windows: Query<&Window>,
                camera_query: Query<(&Camera, &GlobalTransform)>,
                mut gizmos: Gizmos,
                // ground_query: Query<&TileObj, With<Ground>>) {
                // ground_query: Query<&Biome>) {
                query: Query<&TileObj>) {
    // ground_query: Query<&GlobalTransform, With<Ground>>) {
    // cursor position
    // https://bevyengine.org/examples/ui-user-interface/relative-cursor-position/
    if keyboard_input.just_pressed(KeyCode::Enter) {
        info!("Enter pressed");
        // test out things here
        println!("Len of ground tile objects: {}", query.iter().len());
        // get and print out all Tile Ids?
        let mut i = 0;
        for trans in &query {
            // info!("Trans: id: {:?} biome: {}", trans.id, trans.biome);
            i += 1;
        }
        println!("Count of ground objects: {}", i);

        // Do a camera raycast
        info!("Do a raycast from camera");
        let (camera, camera_transform) = camera_query.single();
        // let ground = query.single();

        let Some(cursor_position) = windows.single().cursor_position() else {
            return;
        };
        info!("Cursor position: {:?}", cursor_position);
        // Calculate a ray pointing from the camera into the world based on the cursor's position.
        let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
            return;
        };
        info!("Ray is: {:?}", ray);

        // Draw a circle just above the ground plane at that position.
        // gizmos.circle(point + ground.up() * 0.01, ground.up(), 0.2, Color::WHITE);
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
    // ray.intersect_plane(ground.translation(), ground)
    //     //ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up()))
    // else {
    //     return;
    // };
    // let point = ray.get_point(distance);

    // Draw a circle just above the ground plane at that position.
    // gizmos.circle(point + ground.up() * 0.01, ground.up(), 0.2, Color::WHITE);
}

#[derive(Component, Debug)]
pub struct TileObj {
    pub id: u32,
    pub biome: String,
}

impl TileObj {
    pub fn new(id: u32, biome: String) -> Self {
        Self { id, biome }
    }
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
