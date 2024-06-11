use std::sync::Arc;

use anyhow::Context;

use crate::{
    traits::AsAnyhow,
    App,
};

pub struct DrawState<'window> {
    pub window: Arc<winit::window::Window>,
    pub surface: wgpu::Surface<'window>,
    pub config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub fill_pipeline: FillPipeline,
    pub text_pipeline: TextPipeline,
}

pub struct FillPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}
pub struct TextPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl App<'_> {
    pub fn draw(&mut self) -> anyhow::Result<()> {
        let draw_state = self
            .draw_state
            .as_ref()
            .context("Draw state isnot inited yet")?;

        let state_copy = Arc::clone(&self.state).lock().as_anyhow()?.clone();

        let frame = draw_state
            .surface
            .get_current_texture()
            .context("Failed to acquire next swap chain texture")?;
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

            draw_state.draw_ui(&mut self.ui_primitives, &state_copy, &mut rpass);
        }

        draw_state.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
}
