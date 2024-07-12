use bevy::prelude::*;
use serde::*;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use bevy::render::{camera::Camera, mesh::Indices, render_resource::PrimitiveTopology, RenderPlugin};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use serde::ser::SerializeSeq;
use hexasphere::shapes::IcoSphere;
use std::io::Read;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use icosahedron::Polyhedron;
use rand::Rng;
use rehexed::rehexed;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::default(),
            brightness: 1000.,
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
        // .add_systems(Update, (keyboard_controls))
        .run();
}

fn setup(mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>,
         asset_server: Res<AssetServer>) {
    info!("Setup function called");
    // Basic 3D camera. Add keyboard_controls to update above
    // commands.spawn(Camera3dBundle {
    //     transform: Transform::from_translation(Vec3::new(-10.0, 15., 0.0))
    //         .looking_at(Vec3::default(), Vec3::Y),
    //     ..default()
    // });
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
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
    let mut file = File::open("hexsphere_r10_d2.json").expect("file should open read only");
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    // Has json fields:
    // positions [-5.3716536, 0.0, 3.3198638],
    // cells [0, 1, 2],
    // normals: [-0.5, -0.30901703, 0.80901694],
    // colors [0.6898445, 0.40806764, 0.51248604]
    // faces [90, 91, 92, 93, 94, 95, 96, 97, 98, 99]
    // `jq '.faces|length' hexsphere_r10_d0.json` was 12? cells: 120, pos: 360
    let json_data: serde_json::Value = serde_json::from_str(&data).expect("Unable to parse");

    println!("{:?}", json_data);
    println!("Please call {} at the number {}", json_data["positions"][0][0], json_data["normals"][0][2]);

    let positions = json_data["positions"].as_array().expect("Should be array");
    let positions_vec: Vec<Vec<f32>> = positions
        .iter()
        .map(|vec| vec![
            vec[0].as_f64().unwrap() as f32,
            vec[1].as_f64().unwrap() as f32,
            vec[2].as_f64().unwrap() as f32])
        .collect();
    let posses: Vec<Vec3> = positions.iter().map(|vec| Vec3::new(vec[0].as_f64().unwrap() as f32, vec[1].as_f64().unwrap() as f32, vec[2].as_f64().unwrap() as f32)).collect();;
    println!("Poss: {:?}", positions_vec[0]);

    let nor = json_data["normals"].as_array().expect("Should be array");
    let normals: Vec<Vec3> = nor.iter().map(|vec| Vec3::new(vec[0].as_f64().unwrap() as f32, vec[1].as_f64().unwrap() as f32, vec[2].as_f64().unwrap() as f32)).collect();

    let ind = json_data["cells"].as_array().expect("Should be array");
    let indices: Vec<u32> = ind.iter().map(|vec| vec![vec[0].as_f64().unwrap() as u32, vec[1].as_f64().unwrap() as u32, vec[2].as_f64().unwrap() as u32]).flatten().collect();
    println!("indices: {:?}", indices[0]);
    println!("\nCreate the mesh");

    // spawn mesh from icosahedron
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD);

    // mesh.set_indices(Some(Indices::U32(indices))); // was bevy 12.1
    mesh.insert_indices(Indices::U32(indices)); // 13.2
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, posses);
    let mesh_handle = meshes.add(mesh);

    // let mut cmd = commands.spawn(PbrBundle {
    //     mesh: mesh_handle.clone(),
    //     material: materials.add(Color::BISQUE),
    //     transform: Transform::from_translation(Vec3::ZERO),
    //     ..Default::default()
    // });

    // Hexasphere rust crate
    println!("\n\n ____________________________________\nTest out the hexasphere crate\n____________________________________");
    // At 20 subdivisions. Points: 4412, indices: 26_460
    // At 12 subdivisions. Points: 1692, indices: 10_140
    // At 1 subdivisions. Points: 42, indices: 240
    // At 0 subdivisions. Points: 12, indices: 60  is a D12. 60 indices are 12 sides * 5 points each
    let sphere = IcoSphere::new(5, |_| ());
    // adjacency allows the user to create neighbour maps from the indices provided by the Subdivided struct

    let indices = sphere.get_all_indices();
    println!("\n\nRaw indices are: {:?}\n", indices);
    let adjacency_list = rehexed(&indices, sphere.raw_points().len());
    println!("\n\nRehexed: {:?}\n", adjacency_list);

    println!("Rehexed len: {:?}", adjacency_list.len());

    // let json = serde_json::to_string(&sphere).expect("Problem serializing");
    let points = sphere.raw_points();
    // let point = points[0];
    // map Vec3A to Vec3 from bevy
    let vec_points: Vec<Vec3> = points.iter().map(|p| Vec3::new(p.x, p.y, p.z)).collect();

    // Instantiate game Obj at each "point"
    // for point in vec_points {
    //     let mut x =  Transform::from_translation(point * 2.); //.looking_at(Vec3::ZERO, Vec3::Y);
    //     // x.rotate_y(180_f32);
    //     x.rotate_x(180_f32);
    //     x.rotate_z(90_f32);
    //     commands.spawn(
    //         SceneBundle {
    //             scene: asset_server.load("pent.glb#Scene0"),
    //             transform: x, //Transform::from_translation(point * 2.),
    //                 // .looking_at(Vec3::ZERO, Vec3::Y)
    //             ..default()
    //         }
    //     );
    // }

    // println!("Points len: {}", points.len());
    // let mut i = 0;
    // for p in points {
    //     if i > 5 { break; }
    //     println!("{:?} is a point on the sphere!", p);
    //     i += 1;
    // }
    // let indices = sphere.get_all_indices();
    // println!("\nindices len: {}", indices.len());
    // for triangle in indices.chunks(3) {
    //     if i > 10 { break; }
    //     println!(
    //         "[{}, {}, {}] is a triangle on the resulting shape",
    //         triangle[0],
    //         triangle[1],
    //         triangle[2],
    //     );
    //     i += 1;
    // }

    // Generate mesh as 1 entity.
    // // Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
    // let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD); //, RenderAssetUsages::new() 13.2
    // // // mesh.set_indices(Some(Indices::U32(indices))); // was bevy 12.1
    // let ind = Indices::U32(indices);
    // mesh.insert_indices(ind);
    // mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec_points);

    // // mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    // // mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    // let mesh_handle = meshes.add(mesh);

    // adjacency_list  [87, 92, 95, 98, 101, 18446744073709551615], [13, 102, 26, 0, 15, 105],
    let mut pents = 12;
    for (i, x) in vec_points.iter().enumerate() {
        println!("{i}: {x:?}");
    }
    // Randomizer for colors
    let mut ranr = rand::thread_rng();
    for x in adjacency_list {
        println!("x inside forloop is: {:?}", x);
        if pents > 0 {
            // Spawn the pentagon
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD); //, RenderAssetUsages::new() 13.2
            // println!("points[0] from vec_points: {:?}", vec_points[x[0]]);
            let my_points = vec![vec_points[x[0]], vec_points[x[1]], vec_points[x[2]], vec_points[x[3]], vec_points[x[4]]];

            let mut center: Vec3 = my_points
                .iter()
                .fold(Vec3::ZERO, |sum, i| sum + *i) / 5.0;
            //.map(jason).map(|x| x / 5).collect(); // (point1 + point2 + point3) / 3.0;
            println!("my points; {:?}", my_points);
            println!("center; {:?}", center);
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, my_points);
            // Pentagon Indices
            let pent = vec![
                0, 1, 2,
                0, 2, 3,
                0, 3, 4];
            mesh.insert_indices(Indices::U32(pent));
            let mesh_handle = meshes.add(mesh);

            let mut cmd = commands.spawn(PbrBundle {
                mesh: mesh_handle.clone(),
                material: materials.add(Color::BISQUE),
                transform: Transform::from_translation(center),
                ..Default::default()
            });
            pents -= 1;
        } else {
            // Hexagons?
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD); //, RenderAssetUsages::new() 13.2
            // insert points and indicies
            // println!("points[0] from vec_points: {:?}", vec_points[x[0]]);
            let my_points = vec![vec_points[x[0]], vec_points[x[1]], vec_points[x[2]], vec_points[x[3]], vec_points[x[4]], vec_points[x[5]]];
            println!("my points; {:?}", my_points);
            let center: Vec3 = my_points
                .iter()
                .fold(Vec3::ZERO, |sum, i| sum + *i) / 6.0;
            println!("Center is {center}");
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, my_points);

            // Pentagon Indices
            let hex = vec![
                0, 1, 2,
                0, 2, 3,
                0, 3, 4,
                0, 4, 5];
            mesh.insert_indices(Indices::U32(hex));

            let mesh_handle = meshes.add(mesh);
            let random = &ranr.gen_range(0..4);
            let col = match random {
                0 => Color::PINK,
                1 => Color::ORANGE_RED,
                2 => Color::BLUE,
                3 => Color::LIME_GREEN,
                _ => Color::BISQUE,
            };

            let mut cmd = commands.spawn(PbrBundle {
                mesh: mesh_handle.clone(),
                material: materials.add(col),
                transform: Transform::from_translation(center),
                ..Default::default()
            });
            // break;
        }
    }
}

fn write_to_json_file(polyhedron: Polyhedron, path: &Path) {
    let mut json_file = File::create(path).expect("Can't create file");
    let json = serde_json::to_string(&polyhedron).expect("Problem serializing");
    json_file
        .write_all(json.as_bytes())
        .expect("Can't write to file");
}

/// Move the camera around with the keyboard
pub fn keyboard_controls(
    input: Res<ButtonInput<KeyCode>>,
    // input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Camera)>,
) {
    if let Some((mut transform, _camera)) = query.iter_mut().next() {
        let speed = 10.;
        let forward = Vec3::new(1., 0., 0.);
        let left = Vec3::new(0., 0., -1.);
        let up = Vec3::new(0., 1., 0.);
        let mut pos = transform.translation;
        // if input.pressed(KeyCode::KeyW) {
        if input.pressed(KeyCode::KeyW) {
            pos += speed * forward * time.delta_seconds();
        } else if input.pressed(KeyCode::KeyS) {
            pos -= speed * forward * time.delta_seconds();
        }
        if input.pressed(KeyCode::KeyA) {
            pos += speed * left * time.delta_seconds();
        } else if input.pressed(KeyCode::KeyD) {
            pos -= speed * left * time.delta_seconds();
        }
        if input.pressed(KeyCode::KeyQ) {
            pos += speed * up * time.delta_seconds();
        } else if input.pressed(KeyCode::KeyE) {
            pos -= speed * up * time.delta_seconds();
        }

        transform.translation = pos;
    }
}