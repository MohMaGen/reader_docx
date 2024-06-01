#![feature(more_qualified_paths)]

use std::{
    fmt::{Debug, Display},
    io::Read,
    path::PathBuf,
    sync::Arc,
};

use docx_document::DocxDocument;
use docx_editor::DocxEditor;
use iced::{
    executor,
    keyboard::{self, key::Named},
    widget::{self, row},
    Application, Command, Settings, Theme,
};
use minidom::Element;

pub mod docx_document;
pub mod docx_editor;
pub mod traits;

fn main() -> iced::Result {
    App::run(Settings::default())
}

#[derive(Default)]
pub struct App {
    pub command_line: CommandLine,
    pub document: Option<Arc<Document>>,
    pub ui_mode: UiMode,
}

#[derive(Clone, Debug)]
pub enum Message {
    EnterCommand(CommandInputAction),
    DoCommand(String),
    ToMode(UiMode),
    OpenDocx(Result<Arc<Document>, ReaderDocxError>),
    PickDocx(Option<PathBuf>),
}

#[derive(Clone, Debug)]
pub enum CommandInputAction {
    Enter,
    Input(String),
    Backspace,
}

#[derive(Debug, Clone)]
pub enum ReaderDocxError {
    ReadDocx(String),
}

impl Application for App {
    type Executor = executor::Default;

    type Message = Message;

    type Theme = Theme;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        if let Some(document) = &self.document {
            format!("reader docx {:?}", document.path)
        } else {
            "reader docx".into()
        }
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::EnterCommand(action) => self.update_command_line_action(action),
            Message::ToMode(mode) => self.update_mode(mode),
            Message::DoCommand(command) => match &command.trim()[1..] {
                "view" => Command::perform(async move { UiMode::View }, Message::ToMode),
                "open" => Command::perform(pick_docx(), Message::PickDocx),
                _ => Command::none(),
            },
            Message::PickDocx(Some(file)) => {
                Command::perform(open_and_parse(file), Message::OpenDocx)
            }
            Message::OpenDocx(Ok(document)) => {
                println!("{}", document.document);
                self.document = Some(document);
                Command::none()
            }
            Message::OpenDocx(Err(ReaderDocxError::ReadDocx(err))) => {
                eprintln!("{}", err);
                Command::none()
            }
            _ => Command::none(),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        let command_line = row![
            widget::container(widget::text(format!("{}", self.ui_mode)))
                .padding(5)
                .style(UiModeContainerStyle(self.ui_mode)),
            widget::container(widget::text(&self.command_line.content)).padding(5)
        ]
        .padding(5);
        
        if let Some(document) = &self.document {
            widget::column![DocxEditor::new(&document.document, self.ui_mode), command_line].into()
        } else {
            widget::column![widget::vertical_space(), command_line].into()
        }

    }

    fn theme(&self) -> Self::Theme {
        Self::Theme::default()
    }

    fn style(&self) -> <Self::Theme as iced::application::StyleSheet>::Style {
        <Self::Theme as iced::application::StyleSheet>::Style::default()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        match self.ui_mode {
            UiMode::CommandInput => command_input_mode_keys(),
            UiMode::Command => command_mode_keys(),
            UiMode::View | UiMode::Edit => keyboard::on_key_press(|key, _modifiers| match key {
                keyboard::Key::Named(Named::Escape) => Some(Message::ToMode(UiMode::Command)),
                _ => None,
            }),
        }
    }

    fn scale_factor(&self) -> f64 {
        1.0
    }
}

fn command_mode_keys() -> iced::Subscription<Message> {
    keyboard::on_key_press(|key, modifiers| match key {
        keyboard::Key::Character(s) if s == ";" && modifiers.shift() => {
            Some(Message::ToMode(UiMode::CommandInput))
        }
        keyboard::Key::Character(s) if s == "i" || s == "a" || s == "s" => {
            Some(Message::ToMode(UiMode::Edit))
        }
        _ => None,
    })
}

fn command_input_mode_keys() -> iced::Subscription<Message> {
    keyboard::on_key_press(|key, modifiers| match key {
        keyboard::Key::Named(Named::Escape) => Some(Message::ToMode(UiMode::Command)),
        keyboard::Key::Named(Named::Enter) => {
            Some(Message::EnterCommand(CommandInputAction::Enter))
        }
        keyboard::Key::Named(Named::Backspace) => {
            Some(Message::EnterCommand(CommandInputAction::Backspace))
        }
        keyboard::Key::Named(Named::Space) => {
            Some(Message::EnterCommand(CommandInputAction::Input(" ".into())))
        }
        keyboard::Key::Character(s) => Some(Message::EnterCommand(CommandInputAction::Input(
            if modifiers.shift() {
                s.to_string().to_uppercase()
            } else {
                s.to_string().to_lowercase()
            },
        ))),
        _ => None,
    })
}

impl App {
    fn update_command_line_action(&mut self, action: CommandInputAction) -> Command<Message> {
        match action {
            CommandInputAction::Enter => {
                let content = self.command_line.content.clone();
                self.command_line.content = String::new();

                Command::perform(async move { content }, Message::DoCommand)
            }
            CommandInputAction::Input(s) => {
                self.command_line.content.push_str(&s);
                Command::none()
            }
            CommandInputAction::Backspace => {
                self.command_line.content.pop();
                Command::none()
            }
        }
    }

    fn update_mode(&mut self, mode: UiMode) -> Command<Message> {
        if mode == UiMode::CommandInput {
            self.command_line.content = ":".into();
        }
        self.ui_mode = mode;
        Command::none()
    }
}

#[derive(Default)]
pub struct CommandLine {
    pub content: String,
    pub histroy: Vec<String>,
}

#[derive(Debug)]
pub struct Document {
    pub document: DocxDocument,
    pub path: PathBuf,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum UiMode {
    Command,
    CommandInput,
    #[default]
    View,
    Edit,
}

impl From<UiMode> for iced::Color {
    fn from(value: UiMode) -> Self {
        match value {
            UiMode::Command | UiMode::CommandInput => Self::new(0.4, 0.5, 0.8, 1.0),
            UiMode::View => Self::new(0.8, 0.75, 0.56, 1.0),
            UiMode::Edit => Self::new(0.4, 0.8, 0.56, 1.0),
        }
    }
}

impl std::fmt::Display for UiMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UiMode::Command | UiMode::CommandInput => write![f, "command"],
            UiMode::View => write![f, "view"],
            UiMode::Edit => write![f, "edit"],
        }
    }
}

pub struct UiModeContainerStyle(pub UiMode);

impl From<UiModeContainerStyle> for iced::theme::Container {
    fn from(value: UiModeContainerStyle) -> Self {
        Self::Custom(Box::new(value))
    }
}

impl widget::container::StyleSheet for UiModeContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> widget::container::Appearance {
        let palete = style.palette();

        widget::container::Appearance {
            background: Some(iced::Background::Color(self.0.clone().into())),
            text_color: Some(palete.text),
            ..Default::default()
        }
    }
}

async fn pick_docx() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title("Open a docx file...")
        .pick_file()
        .await
        .map(|v| v.path().to_path_buf())
}

async fn open_and_parse(file: PathBuf) -> Result<Arc<Document>, ReaderDocxError> {
    let archive = tokio::fs::read(file.clone())
        .await
        .map_err(|err| ReaderDocxError::ReadDocx(err.to_string()))?;

    let document = get_element(&archive, "word/document.xml")?;
    let fonts = get_element(&archive, "word/fontTable.xml")?;

    Ok(Arc::new(Document {
        document: (&document, &fonts)
            .try_into()
            .read_docx_err("Failed to create docx document")?,
        path: file,
    }))
}

fn get_element(archive: &Vec<u8>, file: &str) -> Result<Element, ReaderDocxError> {
    let archive = std::io::Cursor::new(archive);

    let mut document = String::new();
    zip::ZipArchive::new(archive)
        .read_docx_err("Failed to parse archive")?
        .by_name(file)
        .read_docx_err(format!("Failed to get {} file", file))?
        .read_to_string(&mut document)
        .read_docx_err(format!("Failed to read to string."))?;

    document
        .parse()
        .read_docx_err("Failed to parse document.xml file")
}

pub trait ToReaderDocxError {
    type Item;

    fn read_docx_err(self, context: impl Display) -> Result<Self::Item, ReaderDocxError>;
}

impl<T, E> ToReaderDocxError for Result<T, E>
where
    E: Display,
{
    type Item = T;

    fn read_docx_err(self, context: impl Display) -> Result<T, ReaderDocxError> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(ReaderDocxError::ReadDocx(format!("{}: `{}`", context, err))),
        }
    }
}
