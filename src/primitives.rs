use wgpu::util::DeviceExt;

use crate::{docx_document::Color, draw::DrawState, math, uniforms::Uniforms2d};

pub enum Primitive {
    Rect {
        uniform: Uniforms2d,
        buffer: wgpu::Buffer,
        bindgroup: wgpu::BindGroup,
    },
}

impl DrawState<'_> {
    pub fn new_rect(&self, rect: impl Into<math::Rectangle>, color: Color) -> Primitive {
        let uniform = self.calc_rect_uniform(rect, color);
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bindgroup = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.fill_pipeline.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
        });

        Primitive::Rect {
            uniform,
            buffer,
            bindgroup,
        }
    }

    pub fn update_rect(
        &self,
        rect: impl Into<math::Rectangle>,
        color: Color,
        primitive: &mut Primitive,
    ) {
        let uniform_value = self.calc_rect_uniform(rect, color);
        match primitive {
            Primitive::Rect {
                uniform,
                buffer: uniform_buffer,
                ..
            } => {
                *uniform = uniform_value;
                self.queue
                    .write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&[uniform_value]));
            }
        }
    }

    fn calc_rect_uniform(&self, rect: impl Into<math::Rectangle>, color: Color) -> Uniforms2d {
        let rect: math::Rectangle = rect.into();
        let (math::Point { x, y }, math::Size { width, height }) = rect.get_point_and_size();

        let (w_width, w_height) = (self.config.width as f32, self.config.height as f32);
        let (x, y, width, height) = (
            (x + width / 2.) / w_width * 2. - 1.,
            1. - (y + height / 2.) / w_height * 2.,
            width / w_width,
            height / w_height,
        );

        let translation = glam::Mat4::from_translation(glam::Vec3 { x, y, z: 0. });
        let scale = glam::Mat4::from_scale(glam::Vec3 {
            x: width,
            y: height,
            z: 1.,
        });

        let uniform = Uniforms2d {
            color: color.as_array(),
            transform: *(translation * scale).as_ref(),
        };
        uniform
    }

    pub fn draw_primitive<'a, 'b: 'a>(
        &'b self,
        rpass: &mut wgpu::RenderPass<'a>,
        primitive: &'a Primitive,
    ) {
        match primitive {
            Primitive::Rect { bindgroup, .. } => {
                rpass.set_pipeline(&self.fill_pipeline.pipeline);
                rpass.set_bind_group(0, &bindgroup, &[]);
                rpass.set_vertex_buffer(0, self.fill_pipeline.vertex_buffer.slice(..));
                rpass.draw(0..6, 0..1);
            }
        }
    }
}
