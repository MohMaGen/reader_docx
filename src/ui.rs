use crate::{docx_document::Color, draw::DrawState, primitives::Primitive};

pub struct UiState {
    pub console_rect: Primitive,
}

impl UiState {
    pub fn init(draw_state: &DrawState) -> Self {
        Self {
            console_rect: draw_state.new_rect((100., 100., 200., 200.), Color::rgb(1., 0., 0.))
        }
    }
}


