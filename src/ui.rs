use crate::{
    draw::DrawState,
    math,
    primitives::{PlainTextProperties, Primitive},
    state::State,
};

pub struct UiState {
    pub statusline_rect: Primitive,
    pub mode_rect: Primitive,
    pub mode_text: Primitive,
    pub hello_text: Primitive,
    pub console_input: Primitive,
    pub ui_font: rusttype::Font<'static>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            ui_font: rusttype::Font::try_from_bytes(include_bytes!("../fonts/small_pixel-7.ttf"))
                .unwrap(),
            statusline_rect: Default::default(),
            console_input: Default::default(),
            mode_rect: Default::default(),
            mode_text: Default::default(),
            hello_text: Default::default(),
        }
    }
}

impl DrawState<'_> {
    pub fn draw_ui<'a, 'b: 'a>(
        &'b self,
        ui_primitives: &'a mut UiState,
        state: &State,
        rpass: &mut wgpu::RenderPass<'a>,
    ) {
        let colorscheme = state.colorscheme.clone();

        let (w_width, w_height) = (self.config.width as f32, self.config.height as f32);

        let status_line_rect = math::Rectangle::from((0., w_height - 40., w_width, w_height));
        self.draw_and_update(
            rpass,
            (status_line_rect, colorscheme.statusline_bg_color),
            &mut ui_primitives.statusline_rect,
        );

        let mode_rect = self.draw_mode(
            status_line_rect,
            colorscheme.clone(),
            state,
            &mut ui_primitives.mode_rect,
            &mut ui_primitives.mode_text,
            ui_primitives.ui_font.clone(),
            rpass,
        );
        self.draw_and_update(
            rpass,
            PlainTextProperties::new(
                status_line_rect
                    .add_paddings(7.)
                    .move_left_top((mode_rect.width() , 0.)),
                colorscheme.statusline_fg_color,
                state.console_input.clone(),
                ui_primitives.ui_font.clone(),
            ),
            &mut ui_primitives.console_input,
        );

        self.draw_and_update(
            rpass,
            PlainTextProperties {
                left_top: (100., 100.).into(),
                content: String::from("Hello, world! Привет Мир"),
                font: rusttype::Font::try_from_bytes(include_bytes!("../fonts/small_pixel-7.ttf"))
                    .unwrap(),
                color: 0x00000ff.into(),
                scale: 40.,
            },
            &mut ui_primitives.hello_text,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_mode<'a, 'b: 'a>(
        &'b self,
        status_line_rect: math::Rectangle,
        colorscheme: crate::colorscheme::ColorScheme,
        state: &State,
        mode_rect: &'a mut Primitive,
        mode_text: &'a mut Primitive,
        ui_font: rusttype::Font<'static>,
        rpass: &mut wgpu::RenderPass<'a>,
    ) -> math::Rectangle {
        let rect = status_line_rect.add_paddings(5.);
        self.update_prim(
            PlainTextProperties::new(
                status_line_rect.add_paddings(7.).move_left_top((3., 0.)),
                colorscheme.statusline_bg_color,
                state.mode.to_string(),
                ui_font.clone(),
            ),
            mode_text,
        );

        let rect = rect.with_width(mode_text.get_rect().width() + 10.);
        self.draw_and_update(
            rpass,
            (rect, colorscheme.get_mode_color(state.mode)),
            mode_rect,
        );
        self.draw_prim(rpass, mode_text);

        rect
    }
}
