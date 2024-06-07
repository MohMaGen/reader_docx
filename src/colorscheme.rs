
#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub console_bg_color: Color,
    pub console_fg_color: Color,
    pub console_border_color: Color,

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
            console_bg_color: Color::rgb(0x38, 0x4b, 0x55),
            console_fg_color: Color::rgb(0xd3, 0xc6, 0xaa),
            console_border_color: Color::rgb(0xd3, 0xc6, 0xaa),

            view_mode_color: Color::rgb(0xe6, 0x7e, 0x80),
            command_mode_color: Color::rgb(0x7f, 0xbb, 0xb3),
            edit_mode_color: Color::rgb(0xdb, 0xbc, 0x7f),

            page_color: Color::rgb(0xd3, 0xc6, 0xaa),
            page_bg_color: Color::rgb(0x4f, 0x5b, 0x58),
            page_border_color: Color::rgb(0xe6, 0x7e, 0x80),
        }
    }
}
