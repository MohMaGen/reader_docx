use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{colorscheme::ColorScheme, docx_document::DocxDocument};

#[derive(Clone, Default)]
pub struct State {
    pub mode: Mode,
    pub console_input: String,
    pub command_in_process: Vec<String>,
    pub colorscheme: ColorScheme,
    pub document: Option<Document>,
}

#[derive(Clone, Default)]
pub struct Document {
    pub document: Arc<Box<DocxDocument>>,
    pub zip_document: Vec<u8>,
    pub path: PathBuf,
}

#[derive(Clone, Copy, Default)]
pub enum Mode {
    #[default]
    View,
    Edit,
    Normal,
    CommandInput,
}
impl State {
    pub fn init() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::default()))
    }

    pub fn load_console_input(&mut self) {
        self.command_in_process = self
            .console_input
            .clone()
            .split(char::is_whitespace)
            .filter(|s| s.len() != 0)
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        self.console_input = String::new();
        self.mode = Mode::Normal;
    }

    pub fn get_console_command_arg(&self, idx: usize) -> Option<&str> {
        self.command_in_process.get(idx).map(String::as_str)
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
                Mode::Normal => "normal",
                Mode::CommandInput => "command",
            }
        )
    }
}
