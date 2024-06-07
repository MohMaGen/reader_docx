use std::sync::Arc;

use wgpu::RenderPass;

use crate::{buffers::{BufferName, UiBuffers}, docx_document, vertex::Vertex2d, App};

pub struct DrawState<'window> {
    pub window: Arc<winit::window::Window>,
    pub surface: wgpu::Surface<'window>,
    pub config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub fill_pipeline: FillPipeline,
    pub ui_buffers: UiBuffers,
}

pub struct FillPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
}

impl App<'_> {
    pub fn draw(&self) {
        let Some(draw_state) = self.draw_state.as_ref() else {
            return;
        };

        let frame = draw_state
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = draw_state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],

                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.push_debug_group("Setup pipeline");
            rpass.set_pipeline(&draw_state.fill_pipeline.pipeline);
            rpass.set_vertex_buffer(0, draw_state.fill_pipeline.vertex_buffer.slice(..));
            rpass.pop_debug_group();
            rpass.push_debug_group("Draw");
            rpass.draw(0..6, 0..1);

            rpass.draw_rect(&draw_state, BufferName::ConsoleBg, (-0.1, 0.1, 0.1, -0.1), docx_document::Color::rgb(0.5, 0.5, 0.5));
        }

        draw_state.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

pub trait BasicDraw<'a> {
    fn draw_rect<'state: 'a>(
        &mut self,
        draw_state: &'state DrawState,
        buffer_name: BufferName,
        rect: impl Into<crate::math::Rectangle>,
        color: crate::docx_document::Color,
    );
}

impl<'a> BasicDraw<'a> for RenderPass<'a> {
    fn draw_rect<'state: 'a>(
        &mut self,
        draw_state: &'state DrawState,
        buffer_name: BufferName,
        rect: impl Into<crate::math::Rectangle>,
        color: crate::docx_document::Color,
    ) {
        let rect: crate::math::Rectangle = rect.into();
        let (crate::math::Point { x, y }, crate::math::Size { width, height }) =
            rect.get_point_and_size();

        draw_state.queue.write_buffer(
            &draw_state.get_buffer(buffer_name),
            0,
            bytemuck::cast_slice(&Vertex2d::rect(x, y, width, height, color)),
        );
        self.push_debug_group("Setup draw Rect");
        self.set_pipeline(&draw_state.fill_pipeline.pipeline);
        self.set_vertex_buffer(0, draw_state.get_buffer(buffer_name).slice(..));
        self.pop_debug_group();
        self.push_debug_group("Draw!!");
        self.draw(0..6, 0..1);
    }
}
