extern crate sdl2;

use std::collections::HashMap;
use std::path::PathBuf;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{Arc, LockResult, Mutex};

use anyhow::Context;
use colored::Colorize;
use colorscheme::ColorScheme;
use docx_document::DocxDocument;
use sdl2::pixels::Color;
use sdl2::video::Window;

pub mod colorscheme;
pub mod commands_apply_thread;
pub mod docx_document;
pub mod draw;
pub mod main_loop;
pub mod math;
pub mod text;
pub mod traits;
pub mod update_events;

pub type Fonts<'ttf, 'wrops> = HashMap<u16, Rc<sdl2::ttf::Font<'ttf, 'wrops>>>;
pub type Command = Pin<Box<dyn Future<Output = anyhow::Result<Message>> + Send>>;

#[non_exhaustive]
pub enum Message {
    LoadDocx(Arc<anyhow::Result<Document>>),
    Aboba,
}

#[derive(Clone)]
pub struct State {
    pub should_exit: bool,
    pub colorscheme: colorscheme::ColorScheme,
    pub console: Console,
    pub mode: UiMode,
    pub cursor: Cursor,
    pub scroll: f32,
    pub scale: f32,
    pub document: Option<Arc<Box<Document>>>,
}

#[derive(Clone)]
pub struct Document {
    pub docx_document: Arc<Box<DocxDocument>>,
    pub path: PathBuf,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum UiMode {
    #[default]
    View,
    Command,
    CommandInput,
    Edit,
}

#[derive(Debug, Default, Clone)]
pub struct Cursor {
    pub paragraph_id: usize,
    pub text_id: usize,
    pub grapheme: usize,
}

#[derive(Clone)]
pub struct Console {
    pub input: String,
    pub font: FontHandle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontHandle {
    name: String,
    size: u16,
    path: PathBuf,
}

pub const ML: f32 = 2.;

pub fn main() -> anyhow::Result<()> {
    let ttf_context = sdl2::ttf::init().context("Failed to initialize ttf context")?;
    let mut fonts = HashMap::<u16, _>::new();

    let font_src = "./fonts/small_pixel-7.ttf";
    for size_pt in 1..150 {
        fonts.insert(
            size_pt,
            Rc::new(
                ttf_context
                                         /*        conver pt to px      */
                    .load_font(font_src, (size_pt as f32 * 96. / 76. * ML) as u16)
                    .as_anyhow()?,
            ),
        );
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = init_video_subsystem(video_subsystem)?;
    let mut canvas = window.into_canvas().build().unwrap();

    let state = Arc::new(Mutex::new(State::init()));
    let commands = Arc::new(Mutex::new(Vec::new()));

    let mut event_pump = sdl_context.event_pump().as_anyhow()?;

    let command_apply_thread =
        commands_apply_thread::spawn(Arc::clone(&state), Arc::clone(&commands));

    canvas.set_scale(1. / ML, 1. / ML).as_anyhow()?;

    loop {
        match main_loop::main_loop(
            Arc::clone(&state),
            &mut event_pump,
            Arc::clone(&commands),
            &mut canvas,
            &fonts,
        ) {
            Ok(true) => break,
            Err(err) => display_error(&err),
            _ => {}
        }
    }

    let _ = command_apply_thread.join();

    Ok(())
}

fn init_video_subsystem(video_subsystem: sdl2::VideoSubsystem) -> anyhow::Result<Window> {
    video_subsystem
        .window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .context("Failed to init video subsystem")
}

impl State {
    pub fn init() -> Self {
        Self {
            should_exit: false,
            colorscheme: Default::default(),
            console: Default::default(),
            mode: Default::default(),
            cursor: Default::default(),
            document: None,
            scale: 0.5,
            scroll: 1.,
        }
    }
}

impl Default for Console {
    fn default() -> Self {
        Self {
            input: String::new(),
            font: FontHandle {
                name: "console font".into(),
                size: 10,
                path: PathBuf::from("./fonts/VT323-Regular.ttf"),
            },
        }
    }
}

pub trait AsAnyhow {
    type Item;
    fn as_anyhow(self) -> anyhow::Result<Self::Item>;
}

impl<T> AsAnyhow for Result<T, String> {
    type Item = T;

    fn as_anyhow(self) -> anyhow::Result<Self::Item> {
        self.map_err(anyhow::Error::msg)
    }
}

impl<T> AsAnyhow for LockResult<T> {
    type Item = T;

    fn as_anyhow(self) -> anyhow::Result<Self::Item> {
        self.map_err(|err| anyhow::Error::msg(err.to_string()))
    }
}

pub fn display_error(err: &anyhow::Error) {
    eprintln!(
        "{}: `{}`\n\n{:?}\n\n",
        "[ error ]".on_red().bold(),
        err.to_string().red().bold(),
        err
    );
}

impl State {
    pub fn console_bg(&self) -> Color {
        self.colorscheme.console_bg_color
    }

    pub fn console_border(&self) -> Color {
        self.colorscheme.console_border_color
    }

    pub fn console_fg(&self) -> Color {
        self.colorscheme.console_fg_color
    }

    pub fn page_color(&self) -> Color {
        self.colorscheme.page_color
    }

    pub fn page_bg_color(&self) -> Color {
        self.colorscheme.page_bg_color
    }

    pub fn page_border_color(&self) -> Color {
        self.colorscheme.page_border_color
    }
}

pub trait StateMutex {
    fn should_exit(&self) -> anyhow::Result<bool>;

    fn console_bg(&self) -> anyhow::Result<Color>;

    fn console_border(&self) -> anyhow::Result<Color>;

    fn console_fg(&self) -> anyhow::Result<Color>;

    fn get_copy(&self) -> anyhow::Result<Box<State>>;
}

impl StateMutex for Arc<Mutex<State>> {
    fn should_exit(&self) -> anyhow::Result<bool> {
        let state = self.lock().as_anyhow()?;
        Ok(state.should_exit.clone())
    }

    fn console_bg(&self) -> anyhow::Result<Color> {
        let state = self.lock().as_anyhow()?;
        Ok(state.colorscheme.console_bg_color.clone())
    }

    fn console_border(&self) -> anyhow::Result<Color> {
        let state = self.lock().as_anyhow()?;
        Ok(state.colorscheme.console_border_color.clone())
    }

    fn console_fg(&self) -> anyhow::Result<Color> {
        let state = self.lock().as_anyhow()?;
        Ok(state.colorscheme.console_fg_color.clone())
    }

    fn get_copy(&self) -> Result<Box<State>, anyhow::Error> {
        let state = self.lock().as_anyhow()?;
        Ok(Box::new(state.clone()))
    }
}

impl UiMode {
    pub fn get_bg_color(&self, scheme: ColorScheme) -> Color {
        match self {
            UiMode::View => scheme.view_mode_color,
            UiMode::Command | UiMode::CommandInput => scheme.command_mode_color,
            UiMode::Edit => scheme.edit_mode_color,
        }
    }
}

impl std::fmt::Display for UiMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UiMode::View => write!(f, "view"),
            UiMode::Command | UiMode::CommandInput => write!(f, "command"),
            UiMode::Edit => write!(f, "edit"),
        }
    }
}
