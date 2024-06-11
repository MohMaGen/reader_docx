use std::sync::{Arc, Mutex};

use crate::colorscheme::ColorScheme;

#[derive(Clone, Default)]
pub struct State {
    pub value: f32,
    pub mode: Mode,
    pub console_input: String,
    pub colorscheme: ColorScheme,
}

#[derive(Clone, Copy, Default)]
pub enum Mode {
    #[default]
    View,
    Edit,
    Command,
    CommandInput,
}
impl State {
    pub fn init() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::default()))
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Mode::View => "view",
                Mode::Edit => "edit",
                Mode::Command => "command",
                Mode::CommandInput => "command",
            }
        )
    }
}

