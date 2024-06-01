use std::{
    io::Read,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use minidom::Element;
use sdl2::{
    event::Event,
    keyboard::{Keycode, Mod},
    EventPump,
};

use crate::{AsAnyhow, Command, Document, Message, State, UiMode};

pub fn update_events<'a>(
    state: Arc<Mutex<State>>,
    event_pump: &mut EventPump,
) -> anyhow::Result<Vec<Command>> {
    let mut state = match state.lock().as_anyhow() {
        Ok(state) => state,
        Err(err) => return Err(err),
    };

    Ok(event_pump
        .poll_iter()
        .filter_map(|event| {
            match event {
                Event::Quit { .. } => state.should_exit = true,
                Event::KeyDown {
                    keycode, keymod, ..
                } => match state.mode {
                    UiMode::View => match keycode {
                        Some(Keycode::Escape) => state.mode = UiMode::Command,
                        Some(Keycode::J) => {
                            state.scroll -= if is_shift(keymod) { 100. } else { 10. }
                        }
                        Some(Keycode::K) => {
                            state.scroll += if is_shift(keymod) { 100. } else { 10. }
                        }
                        Some(Keycode::Minus) => state.scale *= 0.66,
                        Some(Keycode::Equals) if is_shift(keymod) => {
                            state.scale *= 1.5;
                        }
                        Some(Keycode::Equals) => state.scale = 1.,
                        _ => {}
                    },
                    UiMode::Command => {}
                    UiMode::CommandInput => match keycode {
                        Some(Keycode::Escape) => {
                            state.mode = UiMode::Command;
                            state.console.input = "".into();
                        }
                        Some(Keycode::Return) => {
                            let console_input = state.console.input.clone();
                            state.console.input = "".into();
                            state.mode = UiMode::Command;
                            return process_command(&mut state, console_input.as_str());
                        }
                        Some(Keycode::Backspace) => {
                            if state.console.input.len() > 1 {
                                state.console.input = (&state.console.input.as_str()
                                    [..state.console.input.len() - 1])
                                    .to_string();
                            }
                        }
                        _ => {}
                    },
                    UiMode::Edit => match keycode {
                        Some(Keycode::Escape) => state.mode = UiMode::Command,
                        _ => {}
                    },
                },
                Event::TextInput { text, .. } => match state.mode {
                    UiMode::CommandInput => state.console.input.push_str(text.as_str()),
                    UiMode::View => {}
                    UiMode::Command => match text.as_str() {
                        ":" => {
                            state.mode = UiMode::CommandInput;
                            state.console.input = ":".into();
                        }
                        _ => {}
                    },
                    UiMode::Edit => {}
                },
                _ => {}
            };

            None
        })
        .collect::<Vec<_>>())
}

fn is_shift(keymod: Mod) -> bool {
    keymod == Mod::LSHIFTMOD || keymod == Mod::RSHIFTMOD
}

pub fn process_command(state: &mut State, command: &str) -> Option<Command> {
    match &command.trim()[1..] {
        "open" => Some(load_docx()),
        "view" => {
            state.mode = UiMode::View;
            None
        }
        _ => None,
    }
}

pub fn load_docx() -> Command {
    Box::pin(async move {
        let file = rfd::FileDialog::new()
            .set_title("Open a docx file...")
            .add_filter("", &["docx"])
            .pick_file()
            .context("Failed to pick file.")?;

        println!("{:?}", file);

        let archive = std::fs::read(file.clone()).context("Can't read archive")?;

        let document = get_element(&archive, "word/document.xml")?;
        let fonts = get_element(&archive, "word/fontTable.xml")?;

        Ok(Message::LoadDocx(Arc::new(Ok(Document {
            docx_document: Arc::new(Box::new(
                (&document, &fonts)
                    .try_into()
                    .context("failed to parse docx documnet")?,
            )),
            path: file,
        }))))
    })
}

fn get_element(archive: &Vec<u8>, file: &str) -> anyhow::Result<Element> {
    let archive = std::io::Cursor::new(archive);

    let mut document = String::new();
    zip::ZipArchive::new(archive)
        .context("Failed to parse archive")?
        .by_name(file)
        .context(format!("Failed to get {} file", file))?
        .read_to_string(&mut document)
        .context(format!("Failed to read to string."))?;

    document
        .parse()
        .context("Failed to parse document.xml file")
}
