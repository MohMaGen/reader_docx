use std::{path::PathBuf, sync::{Arc, Mutex}};


use crate::{colorscheme::ColorScheme, docx_document::DocxDocument};

#[derive(Clone, Default)]
pub struct State {
    pub value: f32,
    pub mode: Mode,
    pub console_input: String,
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

