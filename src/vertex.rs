use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex2d {
    pub pos: [f32; 2],
}


impl Vertex2d {
    const ATTRIBUTES: [wgpu::VertexAttribute; 1] = [wgpu::VertexAttribute {
        offset: 0,
        shader_location: 0,
        format: wgpu::VertexFormat::Float32x2,
    }];
    pub fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex2d>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

    pub fn rect<'a>(l: f32, t: f32, r: f32, b: f32) -> [Vertex2d; 6] {
        [
            Vertex2d { pos: [l, b] },
            Vertex2d { pos: [r, b] },
            Vertex2d { pos: [l, t] },
            Vertex2d { pos: [l, t] },
            Vertex2d { pos: [r, b] },
            Vertex2d { pos: [r, t] },
        ]
    }

    pub const RECT: &'static [Vertex2d; 6] = &[
        Vertex2d { pos: [-1.0, -1.0] },
        Vertex2d { pos: [1.0, -1.0] },
        Vertex2d { pos: [-1.0, 1.0] },
        Vertex2d { pos: [-1.0, 1.0] },
        Vertex2d { pos: [1.0, -1.0] },
        Vertex2d { pos: [1.0, 1.0] },
    ];
}

