use std::{
    io::Read,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use minidom::Element;
use winit::{event::ElementState, keyboard::KeyCode};

use crate::{
    docx_document::DocxDocument,
    log_helper::LogHelper,
    state::{self, Mode, State},
    traits::AsAnyhow,
};

pub fn keyboard_input(
    state: Arc<Mutex<State>>,
    event: winit::event::KeyEvent,
) -> anyhow::Result<()> {
    if event.repeat {
        return Ok(());
    }
    if event.state == ElementState::Released {
        return Ok(());
    }

    let mode = {
        let mode = state.lock().as_anyhow()?.mode;
        mode
    };

    match mode {
        Mode::View => {
            if command_mode_on_escape(&event, Arc::clone(&state))? {
                return Ok(());
            }
        }
        Mode::Command => match event.text {
            Some(s) if s == ":" => {
                let mut state = state.lock().as_anyhow()?;
                state.mode = Mode::CommandInput;
                state.console_input = ":".into();
            }
            _ => {}
        },

        Mode::CommandInput => {
            if command_mode_on_escape(&event, Arc::clone(&state))? {
                return Ok(());
            }

            if process_command_enter(&event, Arc::clone(&state))? {
                return Ok(());
            }

            process_command_input(&event, Arc::clone(&state))?;
        }
        _ => {}
    }
    Ok(())
}

fn process_command_enter(
    event: &winit::event::KeyEvent,
    state: Arc<Mutex<State>>,
) -> Result<bool, anyhow::Error> {
    match event.physical_key {
        winit::keyboard::PhysicalKey::Code(KeyCode::Enter) => {
            let command = {
                let mut state = state.lock().as_anyhow()?;
                let command = state.console_input.clone();
                state.console_input = String::new();

                command
            };

            match &command.trim()[1..5] {
                "view" => {
                    let mut state = state.lock().as_anyhow()?;
                    state.console_input = "".into();
                    state.mode = Mode::View;
                }
                "open" => {
                    let state = Arc::clone(&state);
                    std::thread::spawn(load_file_and_write_to_state(state));
                }
                _ => {}
            }
        }
        _ => {}
    }

    Ok(false)
}

fn load_file_and_write_to_state(state: Arc<Mutex<State>>) -> impl FnOnce() {
    move || {
        (|| {
            let (document, path) = pollster::block_on(load_docx())?;

            println!("{}", document);

            {
                let mut state = state.lock().as_anyhow()?;
                state.document = Some(state::Document { document, path });
            }

            anyhow::Result::Ok(())
        })()
        .log_if_error();
    }
}

fn process_command_input(
    event: &winit::event::KeyEvent,
    state: Arc<Mutex<State>>,
) -> Result<(), anyhow::Error> {
    match event.clone().text {
        Some(s) => {
            let mut state = state.lock().as_anyhow()?;
            state.console_input = format!("{}{}", state.console_input, s);
        }
        _ => {}
    }

    Ok(())
}

fn command_mode_on_escape(
    event: &winit::event::KeyEvent,
    state: Arc<Mutex<State>>,
) -> Result<bool, anyhow::Error> {
    use winit::keyboard::{KeyCode, PhysicalKey};
    Ok(match event.physical_key {
        PhysicalKey::Code(KeyCode::Escape) => {
            {
                let mut state = state.lock().as_anyhow()?;
                state.mode = Mode::CommandInput;
                state.console_input = "".into();
            }
            true
        }
        _ => false,
    })
}

pub async fn load_docx() -> anyhow::Result<(Arc<Box<DocxDocument>>, PathBuf)> {
    let file = rfd::FileDialog::new()
        .set_title("Open a docx file...")
        .add_filter("", &["docx"])
        .pick_file()
        .context("Failed to pick file.")?;

    let archive = std::fs::read(file.clone()).context("Can't read archive")?;

    let document = get_element(&archive, "word/document.xml")?;
    let fonts = get_element(&archive, "word/fontTable.xml")?;

    Ok((
        Arc::new(Box::new(
            (&document, &fonts)
                .try_into()
                .context("failed to parse docx documnet")?,
        )),
        file,
    ))
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
