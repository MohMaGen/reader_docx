use std::sync::{Arc, Mutex};

use draw::DrawState;
use winit::{
    application::ApplicationHandler,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowAttributes},
};
pub mod draw;
pub mod init;
pub mod traits;
pub mod math;
pub mod buffers;
pub mod vertex;
pub mod docx_document;

#[derive(Clone)]
pub struct State {
    pub value: f32,
}

pub struct App<'window> {
    pub window: Option<Arc<Window>>,
    pub state: Arc<Mutex<State>>,
    pub draw_state: Option<DrawState<'window>>,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::init();
    event_loop.run_app(&mut app)?;

    Ok(())
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let Ok(window) = event_loop.create_window(WindowAttributes::default()) else {
            return;
        };
        let window = Arc::new(window);
        self.window = Some(Arc::clone(&window));
        self.draw_state = Some(DrawState::init(Arc::clone(&window)));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::RedrawRequested => {
                self.draw();
            }
            winit::event::WindowEvent::Resized(size) => {
                if let Some(draw_state) = &mut self.draw_state {
                    draw_state.config.width = size.width;
                    draw_state.config.height = size.height;
                    draw_state
                        .surface
                        .configure(&draw_state.device, &draw_state.config);

                    draw_state.window.request_redraw();
                }
            }
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }
}
