use bytemuck::{Pod, Zeroable};

use crate::docx_document::Color;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex2d {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex2d {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] = [
        wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x2,
        },
        wgpu::VertexAttribute {
            offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
            shader_location: 1,
            format: wgpu::VertexFormat::Float32x4,
        },
    ];
    pub fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex2d>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

    pub fn rect<'a>(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
    ) -> [Vertex2d; 6] {
        [
            Vertex2d {
                position: [x, y - height],
                color: color.as_array(),
            },
            Vertex2d {
                position: [x + width, y - height],
                color: color.as_array(),
            },
            Vertex2d {
                position: [x, y],
                color: color.as_array(),
            },
            Vertex2d {
                position: [x, y],
                color: color.as_array(),
            },
            Vertex2d {
                position: [x + width, y - height],
                color: color.as_array(),
            },
            Vertex2d {
                position: [x + width, y],
                color: color.as_array(),
            },
        ]
    }

    pub const RECT: &'static [Vertex2d; 6] = &[
        Vertex2d {
            position: [-0.5, -0.5],
            color: [0., 0., 0., 0.5],
        },
        Vertex2d {
            position: [0.5, -0.5],
            color: [0., 0., 0., 0.5],
        },
        Vertex2d {
            position: [-0.5, 0.5],
            color: [0., 0., 0., 0.5],
        },
        Vertex2d {
            position: [-0.5, 0.5],
            color: [0., 0., 0., 0.5],
        },
        Vertex2d {
            position: [0.5, -0.5],
            color: [0., 0., 0., 0.5],
        },
        Vertex2d {
            position: [0.5, 0.5],
            color: [0., 0., 0., 0.5],
        },
    ];
}
