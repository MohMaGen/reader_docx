#![feature(let_chains)]
#![feature(once_cell_try)]
use std::sync::{Arc, Mutex};

use document_draw::{DocumentCommand, DocumentDraw};
use draw::DrawState;
use log_helper::LogHelper;
use ui::UiState;
use winit::{
    application::ApplicationHandler,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowAttributes},
};

pub mod colorscheme;
pub mod document_draw;
pub mod docx_document;
pub mod draw;
pub mod font;
pub mod init;
pub mod keyboard_input;
pub mod log_helper;
pub mod math;
pub mod primitives;
pub mod state;
pub mod traits;
pub mod ui;
pub mod uniforms;
pub mod vertex;

pub type DocumentCommands = Arc<Mutex<Vec<DocumentCommand>>>;

pub struct App<'window> {
    pub window: Option<Arc<Window>>,
    pub state: Arc<Mutex<state::State>>,
    pub draw_state: Option<DrawState<'window>>,
    pub document_draw: Option<Box<DocumentDraw>>,
    pub document_commands: DocumentCommands,
    pub ui_primitives: UiState,
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

        let draw_state = DrawState::init(Arc::clone(&window));

        self.draw_state = Some(draw_state);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::RedrawRequested => {
                self.draw().log_if_error();
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
            winit::event::WindowEvent::KeyboardInput { event, .. } => {
                if self.draw_state.is_some() {
                    self.keyboard_input(event).log_if_error();
                    self.draw_state.as_ref().unwrap().window.request_redraw();
                }
            }
            _ => {}
        }
    }
}
