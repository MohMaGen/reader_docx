use std::sync::Arc;

use crate::{docx_document::Color, App};

pub struct DrawState<'window> {
    pub window: Arc<winit::window::Window>,
    pub surface: wgpu::Surface<'window>,
    pub config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub fill_pipeline: FillPipeline,
}

pub struct FillPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl App<'_> {
    pub fn draw(&mut self) {
        let Some(draw_state) = self.draw_state.as_ref() else {
            return;
        };

        let Some(ui_primitives) = self.ui_primitives.as_mut() else {
            return;
        };

        let (w_width, w_height) = (
            draw_state.config.width as f32,
            draw_state.config.height as f32,
        );

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

            draw_state.draw_and_update(
                &mut rpass,
                (
                    (0., w_height - 60., w_width, w_height),
                    Color::rgb(0.5, 0.5, 0.5),
                ),
                &mut ui_primitives.console_rect,
            );
        }

        draw_state.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
