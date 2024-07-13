use nalgebra::Vector3;
use rand::Rng;
use std::collections::HashMap;
use bevy::render::RenderPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy::{prelude::*, render::{mesh::{Indices, PrimitiveTopology}, render_asset::RenderAssetUsages}};
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::default(),
            brightness: 1200.,
        })
        // .add_plugins(DefaultPlugins) // For macbook
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                backends: Some(Backends::VULKAN),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup)
        .run();
}

#[derive(Debug)]
struct Tile {
    vertices: Vec<Vector3<f32>>,
    faces: Vec<(usize, usize, usize)>,
    is_pentagon: bool,
    color: Color,
}

struct GeodesicSphere {
    tiles: Vec<Tile>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Light
    // commands.spawn(PointLightBundle {
    //     transform: Transform::from_xyz(4.0, 8.0, 4.0),
    //     ..Default::default()
    // });

    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 5., 10.0)).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
    ));

    // Generate the geodesic sphere
    let geodesic_sphere = generate_geodesic_sphere(2);

    for tile in geodesic_sphere.tiles {
        // Create mesh from the tile vertices and faces
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());

        let vertices: Vec<[f32; 3]> = tile.vertices.iter()
            .map(|v| [v.x, v.y, v.z])
            .collect();

        // let mut indices = Vec::new();
        // for &(i1, i2, i3) in &tile.faces {
        //     indices.push(i1 as u32);
        //     indices.push(i2 as u32);
        //     indices.push(i3 as u32);
        // }
        let mut indices = Vec::new();
        for &(i1, i2, i3) in &tile.faces {
            let base_index = i1 as u32 * 3;
            indices.push(base_index);
            indices.push(base_index + 1);
            indices.push(base_index + 2);
        }
        println!("Indices are: {:?}", indices);

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

        // Spawn the tile entity
        commands.spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(StandardMaterial {
                base_color: tile.color,
                ..Default::default()
            }),
            ..Default::default()
        });
    }
}

fn generate_geodesic_sphere(subdivisions: u32) -> GeodesicSphere {
    // Define the initial icosahedron vertices and faces
    let t = (1.0 + 5.0f32.sqrt()) / 2.0;
    let vertices = vec![
        Vector3::new(-1.0, t, 0.0),
        Vector3::new( 1.0, t, 0.0),
        Vector3::new(-1.0, -t, 0.0),
        Vector3::new( 1.0, -t, 0.0),
        Vector3::new( 0.0, -1.0,  t),
        Vector3::new( 0.0,  1.0,  t),
        Vector3::new( 0.0, -1.0, -t),
        Vector3::new( 0.0,  1.0, -t),
        Vector3::new( t, 0.0, -1.0),
        Vector3::new( t, 0.0,  1.0),
        Vector3::new(-t, 0.0, -1.0),
        Vector3::new(-t, 0.0,  1.0),
    ];

    let faces = vec![
        (0, 11, 5),
        (0, 5, 1),
        (0, 1, 7),
        (0, 7, 10),
        (0, 10, 11),
        (1, 5, 9),
        (5, 11, 4),
        (11, 10, 2),
        (10, 7, 6),
        (7, 1, 8),
        (3, 9, 4),
        (3, 4, 2),
        (3, 2, 6),
        (3, 6, 8),
        (3, 8, 9),
        (4, 9, 5),
        (2, 4, 11),
        (6, 2, 10),
        (8, 6, 7),
        (9, 8, 1),
    ];
    println!("Faces len: {:?}", faces.len());

    // Subdivide the faces
    let mut vertices: Vec<Vector3<f32>> = vertices.iter().map(|v| v.normalize()).collect();
    let mut faces: Vec<(usize, usize, usize)> = faces;
    println!("Faces len: {:?}", faces.len());
    let mut middle_point_cache: HashMap<(usize, usize), usize> = HashMap::new();

    for _ in 0..subdivisions {
        let mut faces_subdivided = Vec::new();

        for &(v1, v2, v3) in &faces {
            let a = get_middle_point(v1, v2, &mut vertices, &mut middle_point_cache);
            let b = get_middle_point(v2, v3, &mut vertices, &mut middle_point_cache);
            let c = get_middle_point(v3, v1, &mut vertices, &mut middle_point_cache);

            faces_subdivided.push((v1, a, c));
            faces_subdivided.push((v2, b, a));
            faces_subdivided.push((v3, c, b));
            faces_subdivided.push((a, b, c));
        }

        faces = faces_subdivided;
    }

    // Create tiles from the faces
    let mut tiles: Vec<Tile> = Vec::new();
    let mut vertex_to_faces: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut rng = rand::thread_rng();
    let mut ranr = rand::thread_rng();

    for (face_index, &(v1, v2, v3)) in faces.iter().enumerate() {
        vertex_to_faces.entry(v1).or_default().push(face_index);
        vertex_to_faces.entry(v2).or_default().push(face_index);
        vertex_to_faces.entry(v3).or_default().push(face_index);
    }

    for (&vertex, face_indices) in &vertex_to_faces {
        let is_pentagon = face_indices.len() == 5;
        let tile_faces = face_indices.iter().map(|&i| faces[i]).collect::<Vec<_>>();
        let tile_vertices = tile_faces.iter().flat_map(|&(i1, i2, i3)| vec![vertices[i1], vertices[i2], vertices[i3]]).collect::<Vec<_>>();

        // Generate a random color for the tile
        let random = &ranr.gen_range(0..4);
        let color = match random {
            0 => Color::PINK,
            1 => Color::ORANGE_RED,
            2 => Color::BLUE,
            3 => Color::LIME_GREEN,
            _ => Color::BISQUE,
        };
        // rng.gen_range(0..1);
        let color = Color::rgb(
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
        );

        tiles.push(Tile {
            vertices: tile_vertices,
            faces: tile_faces,
            is_pentagon,
            color,
        });
    }

    GeodesicSphere { tiles }
}

fn get_middle_point(
    p1: usize,
    p2: usize,
    vertices: &mut Vec<Vector3<f32>>,
    cache: &mut HashMap<(usize, usize), usize>
) -> usize {
    let key = if p1 < p2 { (p1, p2) } else { (p2, p1) };

    if let Some(&index) = cache.get(&key) {
        return index;
    }

    let midpoint = (vertices[p1] + vertices[p2]) / 2.0;
    let index = vertices.len();
    vertices.push(midpoint.normalize());
    cache.insert(key, index);

    index
}
