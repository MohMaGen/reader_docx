use crate::{docx_document::Color, state};

#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub statusline_bg_color: Color,
    pub statusline_fg_color: Color,

    pub view_mode_color: Color,
    pub command_mode_color: Color,
    pub edit_mode_color: Color,

    pub page_color: Color,
    pub page_bg_color: Color,
    pub page_border_color: Color,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            statusline_bg_color: Color::from(0x384b55ff),
            statusline_fg_color: Color::from(0xd3c6aaff),

            view_mode_color: Color::from(0xe67e80ff),
            command_mode_color: Color::from(0x7fbbb3ff),
            edit_mode_color: Color::from(0xdbbc7fff),

            page_color: Color::from(0xd3c6aaff),
            page_bg_color: Color::from(0x4f5b58ff),
            page_border_color: Color::from(0xe67e80ff),
        }
    }
}

impl ColorScheme {
    pub fn get_mode_color(&self, mode: state::Mode) -> Color {
        match mode {
            state::Mode::View => self.view_mode_color,
            state::Mode::Edit => self.edit_mode_color,
            state::Mode::Command | state::Mode::CommandInput => self.command_mode_color,
        }
    }
}
