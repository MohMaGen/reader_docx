use std::{
    io::Read,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use minidom::Element;
use winit::{
    event::ElementState,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{
    document_draw::{CursorPos, DocumentCommand},
    docx_document::DocxDocument,
    log_helper::LogHelper,
    state::{self, Mode, State},
    traits::AsAnyhow,
};

pub fn keyboard_input(
    state: Arc<Mutex<State>>,
    event: winit::event::KeyEvent,
    document_commands: &mut Vec<DocumentCommand>,
) -> anyhow::Result<()> {
    if event.repeat {
        return Ok(());
    }
    if event.state == ElementState::Released {
        return Ok(());
    }

    let mode = state.lock().to_anyhow()?.mode;

    match mode {
        Mode::View => {
            if normal_mode_on_escape(&event, Arc::clone(&state))? {
                return Ok(());
            }

            scale(&event, document_commands);
            scroll(&event, document_commands);
        }
        Mode::Normal => {
            if normal_movement(&event, document_commands) {
                return Ok(());
            }

            match event.physical_key {
                PhysicalKey::Code(KeyCode::KeyI) => {
                    let mut state = state.lock().to_anyhow()?;
                    state.mode = Mode::Edit;
                    return Ok(());
                }
                _ => {}
            }

            match event.text {
                Some(s) if s == ":" => {
                    let mut state = state.lock().to_anyhow()?;
                    state.mode = Mode::CommandInput;
                    state.console_input = ":".into();
                }
                _ => {}
            }
        }

        Mode::CommandInput => {
            if normal_mode_on_escape(&event, Arc::clone(&state))? {
                return Ok(());
            }

            if process_command_enter(&event, Arc::clone(&state))? {
                return Ok(());
            }

            process_command_input(&event, Arc::clone(&state))?;
        }

        Mode::Edit => {
            if normal_mode_on_escape(&event, Arc::clone(&state))? {
                return Ok(());
            }

            match event.physical_key {
                PhysicalKey::Code(KeyCode::Backspace) => {
                    document_commands.push(DocumentCommand::Remove);
                    return Ok(());
                }
                _ => {}
            }

            match event.text {
                Some(s) if !s.trim().is_empty() => {
                    document_commands.push(DocumentCommand::Add(s.to_string()));
                }
                Some(s) if s.trim().is_empty() => {
                    document_commands.push(DocumentCommand::AddSpace);
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}

fn normal_movement(
    event: &winit::event::KeyEvent,
    document_commands: &mut Vec<DocumentCommand>,
) -> bool {
    match event.physical_key {
        PhysicalKey::Code(KeyCode::Backspace) => {
            document_commands.push(DocumentCommand::Remove);
            true
        }
        PhysicalKey::Code(KeyCode::KeyL) => {
            document_commands.push(DocumentCommand::ChangeCharIdx(1));
            true
        }
        PhysicalKey::Code(KeyCode::KeyH) => {
            document_commands.push(DocumentCommand::ChangeCharIdx(-1));
            true
        }
        PhysicalKey::Code(KeyCode::KeyJ) => {
            document_commands.push(DocumentCommand::ChangeLineIdx(1));
            true
        }
        PhysicalKey::Code(KeyCode::KeyK) => {
            document_commands.push(DocumentCommand::ChangeLineIdx(-1));
            true
        }
        _ => false,
    }
}

fn process_command_enter(
    event: &winit::event::KeyEvent,
    state: Arc<Mutex<State>>,
) -> Result<bool, anyhow::Error> {
    if let PhysicalKey::Code(KeyCode::Enter) = event.physical_key {
        let command = {
            let mut state = state.lock().to_anyhow()?;
            state.mode = Mode::Normal;
            let command = state.console_input.clone();
            state.console_input = String::new();

            command
        };

        match &command.trim()[1..5] {
            "view" => {
                let mut state = state.lock().to_anyhow()?;
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

    Ok(false)
}

fn scale(event: &winit::event::KeyEvent, document_commands: &mut Vec<DocumentCommand>) {
    match event.text.as_ref() {
        Some(input) if input == "-" => {
            document_commands.push(DocumentCommand::RatioScale(0.8));
        }
        Some(input) if input == "=" => {
            document_commands.push(DocumentCommand::NewScale(0.5));
        }
        Some(input) if input == "+" => {
            document_commands.push(DocumentCommand::RatioScale(1.2));
        }
        _ => {}
    }
}

fn scroll(event: &winit::event::KeyEvent, document_draw: &mut Vec<DocumentCommand>) {
    match event.physical_key {
        PhysicalKey::Code(KeyCode::KeyK) => document_draw.push(DocumentCommand::DeltaScroll(100.)),
        PhysicalKey::Code(KeyCode::KeyJ) => document_draw.push(DocumentCommand::DeltaScroll(-100.)),
        _ => {}
    }
}

fn load_file_and_write_to_state(state: Arc<Mutex<State>>) -> impl FnOnce() {
    move || {
        (|| {
            let (document, path) = pollster::block_on(load_docx())?;

            println!("{}", document);

            {
                let mut state = state.lock().to_anyhow()?;
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
    if let PhysicalKey::Code(KeyCode::Backspace) = event.physical_key {
        let mut state = state.lock().to_anyhow()?;
        if state.console_input.len() > 1 {
            state.console_input.pop();
        }
        return Ok(());
    }

    if let Some(s) = event.clone().text {
        let mut state = state.lock().to_anyhow()?;
        state.console_input = format!("{}{}", state.console_input, s);
    }

    Ok(())
}

fn normal_mode_on_escape(
    event: &winit::event::KeyEvent,
    state: Arc<Mutex<State>>,
) -> Result<bool, anyhow::Error> {
    use winit::keyboard::{KeyCode, PhysicalKey};
    Ok(match event.physical_key {
        PhysicalKey::Code(KeyCode::Escape) => {
            {
                let mut state = state.lock().to_anyhow()?;
                state.mode = Mode::Normal;
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
        .context("Failed to read to string.")?;

    document
        .parse()
        .context("Failed to parse document.xml file")
}
