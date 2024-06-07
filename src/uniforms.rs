use bytemuck::{Pod, Zeroable};

use crate::docx_document::Color;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Uniforms2d {
    pub transform: [f32; 16],
    pub color: [f32; 4],
}

impl Default for Uniforms2d {
    fn default() -> Self {
        Self {
            transform: *glam::Mat4::IDENTITY.as_ref(),
            color: Color::rgb(0.5, 0.5, 0.5).as_array(),
        }
    }
}

