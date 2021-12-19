use cgmath::{Angle, Deg};
use glium::{Display, VertexBuffer};

#[derive(Copy, Clone, glium_derive::Vertex)]
pub struct Vertex {
    pos: [f32; 3],
    color: [f32; 4],
    normal: [f32; 3],
    uv: [f32; 2]
}

pub fn triangle(display: &Display) -> (VertexBuffer<Vertex>, Vec<u16>) {
    let vertices = VertexBuffer::new(display, &[
        Vertex { pos: [-1.0, -1.0, 0.0], color: [1.0, 0.0, 0.0, 1.0], normal: [1.0, 0.0, 0.0], uv: [0.0, 0.0] },
        Vertex { pos: [-1.0,  1.0, 0.0], color: [0.0, 1.0, 0.0, 1.0], normal: [1.0, 0.0, 0.0], uv: [0.0, 1.0] },
        Vertex { pos: [ 1.0,  1.0, 0.0], color: [0.0, 0.0, 1.0, 1.0], normal: [1.0, 0.0, 0.0], uv: [1.0, 0.0] },
    ]).unwrap();
    let indices = vec![0, 1, 2];
    (vertices, indices)
}

pub fn square(display: &Display) -> (VertexBuffer<Vertex>, Vec<u16>) {
    let vertices = VertexBuffer::new(display, &[
        Vertex { pos: [-1.0, -1.0, 0.0], color: [1.0, 0.0, 0.0, 1.0], normal: [1.0, 0.0, 0.0], uv: [0.0, 0.0] },
        Vertex { pos: [-1.0,  1.0, 0.0], color: [0.0, 1.0, 0.0, 1.0], normal: [1.0, 0.0, 0.0], uv: [0.0, 1.0] },
        Vertex { pos: [ 1.0,  1.0, 0.0], color: [0.0, 0.0, 1.0, 1.0], normal: [1.0, 0.0, 0.0], uv: [1.0, 1.0] },
        Vertex { pos: [ 1.0, -1.0, 0.0], color: [0.0, 0.0, 0.0, 1.0], normal: [1.0, 0.0, 0.0], uv: [1.0, 0.0] }
    ]).unwrap();
    let indices = vec![0, 1, 2, 3];
    (vertices, indices)
}

pub fn circle(display: &Display) -> (VertexBuffer<Vertex>, Vec<u16>) {
    let mut vertices = Vec::with_capacity(360);
    let mut indices = Vec::with_capacity(360);
    for i in 0..360 {
        let angle = Deg(i as f32);
        let x = angle.cos();
        let y = angle.sin();
        vertices.push(Vertex { pos: [x, y, 0.0], color: [1.0, 1.0, 1.0, 1.0], normal: [1.0, 0.0, 0.0], uv: [(x + 1.0) / 2.0, (y + 1.0) / 2.0] });
        indices.push(i as u16);
    }
    let vertices = VertexBuffer::new(display, &vertices).unwrap();
    (vertices, indices)
}

pub fn cube(display: &Display) -> (VertexBuffer<Vertex>, Vec<u16>) {
    let vertices = VertexBuffer::new(display, &[
        // Max X
        Vertex { pos: [ 0.5, -0.5, -0.5], normal: [1.0,  0.0,  0.0], uv: [0.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5, -0.5,  0.5], normal: [1.0,  0.0,  0.0], uv: [0.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5,  0.5,  0.5], normal: [1.0,  0.0,  0.0], uv: [1.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5,  0.5, -0.5], normal: [1.0,  0.0,  0.0], uv: [1.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        // Min X
        Vertex { pos: [-0.5, -0.5, -0.5], normal: [-1.0, 0.0,  0.0], uv: [0.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [-0.5,  0.5, -0.5], normal: [-1.0, 0.0,  0.0], uv: [1.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [-0.5,  0.5,  0.5], normal: [-1.0, 0.0,  0.0], uv: [1.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [-0.5, -0.5,  0.5], normal: [-1.0, 0.0,  0.0], uv: [0.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        // Max Y
        Vertex { pos: [-0.5,  0.5, -0.5], normal: [0.0,  1.0,  0.0], uv: [0.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5,  0.5, -0.5], normal: [0.0,  1.0,  0.0], uv: [1.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5,  0.5,  0.5], normal: [0.0,  1.0,  0.0], uv: [1.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [-0.5,  0.5,  0.5], normal: [0.0,  1.0,  0.0], uv: [0.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        // Min Y
        Vertex { pos: [-0.5, -0.5, -0.5], normal: [0.0, -1.0,  0.0], uv: [0.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [-0.5, -0.5,  0.5], normal: [0.0, -1.0,  0.0], uv: [0.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5, -0.5,  0.5], normal: [0.0, -1.0,  0.0], uv: [1.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5, -0.5, -0.5], normal: [0.0, -1.0,  0.0], uv: [1.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        // Max Z
        Vertex { pos: [-0.5, -0.5,  0.5], normal: [0.0,  0.0,  1.0], uv: [0.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [-0.5,  0.5,  0.5], normal: [0.0,  0.0,  1.0], uv: [0.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5,  0.5,  0.5], normal: [0.0,  0.0,  1.0], uv: [1.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5, -0.5,  0.5], normal: [0.0,  0.0,  1.0], uv: [1.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        // Min Z
        Vertex { pos: [-0.5, -0.5, -0.5], normal: [0.0,  0.0, -1.0], uv: [0.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5, -0.5, -0.5], normal: [0.0,  0.0, -1.0], uv: [1.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [ 0.5,  0.5, -0.5], normal: [0.0,  0.0, -1.0], uv: [1.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [-0.5,  0.5, -0.5], normal: [0.0,  0.0, -1.0], uv: [0.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
    ]).unwrap();
    let mut indices = Vec::new();
    for face in 0..6u16 {
        for i in &[0, 1, 2, 0, 2, 3] {
            indices.push(4 * face + *i);
        }
    }
    (vertices, indices)
}