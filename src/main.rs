use bevy::prelude::*;
use serde::*;
use std::fs::File;
use std::path::Path;
use serde_json::*;

fn main() {
    App::new()
    .add_systems(Startup, setup)
    .run();
}

fn setup(mut commands: Commands) {
    println!("hello jason");

    let poly = Polyhedron::regular_isocahedron();
    println!("Our Poly {:?}", poly);
    write_to_json_file(poly, "test.json");
}

fn write_to_json_file(polyhedron: Polyhedron, path: &Path) {
    let mut json_file = File::create(path).expect("Can't create file");
    let json = serde_json::to_string(&polyhedron).expect("Problem serializing");
    json_file
        .write_all(json.as_bytes())
        .expect("Can't write to file");
}


#[derive(Debug)]
pub struct Triangle {
    // We use usize for the three points of the triangle because 
    // they are indices into a Vec of Vector3s.
    pub a: usize,
    pub b: usize,
    pub c: usize,
}

impl Triangle {
    fn new(a: usize, b: usize, c: usize) -> Triangle {
        Triangle { a, b, c }
    }
}

impl Serialize for Triangle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let vec_indices = vec![self.a, self.b, self.c];
        let mut seq = serializer.serialize_seq(Some(vec_indices.len()))?;
        for index in vec_indices {
            seq.serialize_element(&index)?;
        }
        seq.end()
    }
}

#[derive(Debug, Serialize)]
pub struct Polyhedron {
    pub positions: Vec<Vec3>,
    pub cells: Vec<Triangle>,
}

impl Polyhedron {
    pub fn regular_isocahedron() -> Polyhedron {
        let t = (1.0 + (5.0 as f32).sqrt()) / 2.0; // 1.618
        println!("t is: {t}");
        Polyhedron {
            positions: vec![
                Vec3::new(-1.0, t, 0.0),
                Vec3::new(1.0, t, 0.0),
                Vec3::new(-1.0, -t, 0.0),
                Vec3::new(1.0, -t, 0.0),
                Vec3::new(0.0, -1.0, t),
                Vec3::new(0.0, 1.0, t),
                Vec3::new(0.0, -1.0, -t),
                Vec3::new(0.0, 1.0, -t),
                Vec3::new(t, 0.0, -1.0),
                Vec3::new(t, 0.0, 1.0),
                Vec3::new(-t, 0.0, -1.0),
                Vec3::new(-t, 0.0, 1.0),
            ],
            cells: vec![
                Triangle::new(0, 11, 5),
                Triangle::new(0, 5, 1),
                Triangle::new(0, 1, 7),
                Triangle::new(0, 7, 10),
                Triangle::new(0, 10, 11),
                Triangle::new(1, 5, 9),
                Triangle::new(5, 11, 4),
                Triangle::new(11, 10, 2),
                Triangle::new(10, 7, 6),
                Triangle::new(7, 1, 8),
                Triangle::new(3, 9, 4),
                Triangle::new(3, 4, 2),
                Triangle::new(3, 2, 6),
                Triangle::new(3, 6, 8),
                Triangle::new(3, 8, 9),
                Triangle::new(4, 9, 5),
                Triangle::new(2, 4, 11),
                Triangle::new(6, 2, 10),
                Triangle::new(8, 6, 7),
                Triangle::new(9, 8, 1),
            ],
        }
    }
}
