use crate::*;
use std::{io::{prelude::*, BufReader}, path::Path, fs::File};


pub fn parse_OBJ(path: &Path, )->Mesh{
    let mut positions: Vec<Vec<f32>> = Vec::new();
    let mut norms: Vec<Vec<f32>> = Vec::new();
    let mut texs: Vec<Vec<f32>> = Vec::new();
    let mut indices: Vec<VertIndicies> = Vec::new();
    println!("Parsing OBJ file: {}", path.as_os_str().to_str().unwrap());
    let mut file = BufReader::new(File::open(path.as_os_str()).expect("Failed to open file"));
    for line in file.lines(){
        let line = line.unwrap();
        let mut words = line.split_whitespace();
        match words.next(){
            Some("v") => {
                let x = words.next().unwrap().parse::<f32>().unwrap();
                let y = words.next().unwrap().parse::<f32>().unwrap();
                let z = words.next().unwrap().parse::<f32>().unwrap();
                positions.push(vec![x, y, z]);
            },
            Some("vn") => {
                let x = words.next().unwrap().parse::<f32>().unwrap();
                let y = words.next().unwrap().parse::<f32>().unwrap();
                let z = words.next().unwrap().parse::<f32>().unwrap();
                norms.push(vec![x, y, z]);
            },
            Some("vt") => {
                let x = words.next().unwrap().parse::<f32>().unwrap();
                let y = words.next().unwrap().parse::<f32>().unwrap();
                texs.push(vec![x, y]);
            },
            Some("f") => {
                let mut face: Vec<VertIndicies> = Vec::new();
                for word in words{
                    let mut indices = word.split("/");
                    let v = indices.next().unwrap().parse::<u32>().unwrap();
                    let t = indices.next().unwrap().parse::<u32>().unwrap();
                    let n = indices.next().unwrap().parse::<u32>().unwrap();
                    face.push([v, t, n]);
                }
                indices.push(face[0]);
                indices.push(face[1]);
                indices.push(face[2]);
            },
            _ => {}
        }
    }
    //Construct the mesh
    let mut vertices: Vec<Vertex> = vec![Vertex::default(); positions.len()];
    

    for i in 0..positions.len(){
        vertices[i].0 = positions[i].as_slice().try_into().unwrap();
    }

    for i in 0..norms.len(){
        vertices[i].1 = norms[i].as_slice().try_into().unwrap();
    }


    let mut mesh = Mesh::new(vertices, indices);

    mesh
}
