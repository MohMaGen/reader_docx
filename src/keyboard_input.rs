use std::{
    io::Read,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use minidom::Element;
use winit::{
    event::ElementState,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::{
    document_draw::DocumentCommand,
    log_helper::LogHelper,
    state::{self, Mode, State},
    traits::AsAnyhow,
    App, DocumentCommands,
};

impl App<'_> {
    pub fn keyboard_input(&mut self, event: winit::event::KeyEvent) -> anyhow::Result<()> {
        if event.repeat {
            return Ok(());
        }
        if event.state == ElementState::Released {
            return Ok(());
        }

        let mode = self.state.lock().to_anyhow()?.mode;

        match mode {
            Mode::View => {
                if self.normal_mode_on_escape(&event)? {
                    return Ok(());
                }

                self.scale(&event)?;
                self.scroll(&event)?;
            }
            Mode::Normal => {
                if self.normal_movement(&event)? {
                    return Ok(());
                }

                if let PhysicalKey::Code(KeyCode::KeyI) = event.physical_key {
                    let mut state = self.state.lock().to_anyhow()?;
                    state.mode = Mode::Edit;
                    return Ok(());
                }

                match event.text {
                    Some(s) if s == ":" => {
                        let mut state = self.state.lock().to_anyhow()?;
                        state.mode = Mode::CommandInput;
                        state.console_input = ":".into();
                    }
                    _ => {}
                }
            }

            Mode::CommandInput => {
                if self.normal_mode_on_escape(&event)? {
                    return Ok(());
                }

                if self.process_command_enter(&event)? {
                    return Ok(());
                }

                self.process_command_input(&event)?;
            }

            Mode::Edit => {
                if self.normal_mode_on_escape(&event)? {
                    return Ok(());
                }

                if let PhysicalKey::Code(KeyCode::Backspace) = event.physical_key {
                    self.document_commands
                        .lock()
                        .to_anyhow()?
                        .push(DocumentCommand::Remove);
                    return Ok(());
                }

                match event.text {
                    Some(s) if !s.trim().is_empty() => {
                        self.document_commands
                            .lock()
                            .to_anyhow()?
                            .push(DocumentCommand::Add(s.to_string()));
                    }
                    Some(s) if s.trim().is_empty() => {
                        self.document_commands
                            .lock()
                            .to_anyhow()?
                            .push(DocumentCommand::AddSpace);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn normal_movement(&mut self, event: &winit::event::KeyEvent) -> anyhow::Result<bool> {
        match event.physical_key {
            PhysicalKey::Code(KeyCode::Backspace) => {
                self.document_commands
                    .lock()
                    .to_anyhow()?
                    .push(DocumentCommand::Remove);
                Ok(true)
            }
            PhysicalKey::Code(KeyCode::KeyL) => {
                self.document_commands
                    .lock()
                    .to_anyhow()?
                    .push(DocumentCommand::ChangeCharIdx(1));
                Ok(true)
            }
            PhysicalKey::Code(KeyCode::KeyH) => {
                self.document_commands
                    .lock()
                    .to_anyhow()?
                    .push(DocumentCommand::ChangeCharIdx(-1));
                Ok(true)
            }
            PhysicalKey::Code(KeyCode::KeyJ) => {
                self.document_commands
                    .lock()
                    .to_anyhow()?
                    .push(DocumentCommand::ChangeLineIdx(1));
                Ok(true)
            }
            PhysicalKey::Code(KeyCode::KeyK) => {
                self.document_commands
                    .lock()
                    .to_anyhow()?
                    .push(DocumentCommand::ChangeLineIdx(-1));
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn process_command_enter(
        &mut self,
        event: &winit::event::KeyEvent,
    ) -> Result<bool, anyhow::Error> {
        if let PhysicalKey::Code(KeyCode::Enter) = event.physical_key {
            let command = {
                let mut state = self.state.lock().to_anyhow()?;
                state.mode = Mode::Normal;
                let command = state.console_input.clone();
                state.console_input = String::new();

                command
            };

            match &command.trim()[1..5] {
                "view" => {
                    let mut state = self.state.lock().to_anyhow()?;
                    state.console_input = "".into();
                    state.mode = Mode::View;
                }
                "open" => {
                    let state = Arc::clone(&self.state);
                    std::thread::spawn(load_file_and_write_to_state(
                        state,
                        Arc::clone(&self.draw_state.as_ref().context("no draw state")?.window),
                    ));
                }
                "save" => {
                    std::thread::spawn(save_document(
                        Arc::clone(&self.document_commands),
                        Arc::clone(&self.draw_state.as_ref().context("no draw state")?.window),
                    ));
                }
                _ => {}
            }
        }

        Ok(false)
    }
    fn scale(&self, event: &winit::event::KeyEvent) -> anyhow::Result<()> {
        match event.text.as_ref() {
            Some(input) if input == "-" => {
                self.document_commands
                    .lock()
                    .to_anyhow()?
                    .push(DocumentCommand::RatioScale(0.8));
            }
            Some(input) if input == "=" => {
                self.document_commands
                    .lock()
                    .to_anyhow()?
                    .push(DocumentCommand::NewScale(0.5));
            }
            Some(input) if input == "+" => {
                self.document_commands
                    .lock()
                    .to_anyhow()?
                    .push(DocumentCommand::RatioScale(1.2));
            }
            _ => {}
        };
        Ok(())
    }

    fn scroll(&self, event: &winit::event::KeyEvent) -> anyhow::Result<()> {
        match event.physical_key {
            PhysicalKey::Code(KeyCode::KeyK) => self
                .document_commands
                .lock()
                .to_anyhow()?
                .push(DocumentCommand::DeltaScroll(100.)),
            PhysicalKey::Code(KeyCode::KeyJ) => self
                .document_commands
                .lock()
                .to_anyhow()?
                .push(DocumentCommand::DeltaScroll(-100.)),
            _ => {}
        };
        Ok(())
    }
    fn process_command_input(
        &mut self,
        event: &winit::event::KeyEvent,
    ) -> Result<(), anyhow::Error> {
        if let PhysicalKey::Code(KeyCode::Backspace) = event.physical_key {
            let mut state = self.state.lock().to_anyhow()?;
            if state.console_input.len() > 1 {
                state.console_input.pop();
            }
            return Ok(());
        }

        if let Some(s) = event.clone().text {
            let mut state = self.state.lock().to_anyhow()?;
            state.console_input = format!("{}{}", state.console_input, s);
        }

        Ok(())
    }

    fn normal_mode_on_escape(
        &mut self,
        event: &winit::event::KeyEvent,
    ) -> Result<bool, anyhow::Error> {
        use winit::keyboard::{KeyCode, PhysicalKey};
        Ok(match event.physical_key {
            PhysicalKey::Code(KeyCode::Escape) => {
                {
                    let mut state = self.state.lock().to_anyhow()?;
                    state.mode = Mode::Normal;
                    state.console_input = "".into();
                }
                true
            }
            _ => false,
        })
    }
}

fn load_file_and_write_to_state(state: Arc<Mutex<State>>, window: Arc<Window>) -> impl FnOnce() {
    move || {
        (|| {
            let document = pollster::block_on(load_docx())?;

            println!("{}", document.document);

            {
                let mut state = state.lock().to_anyhow()?;
                state.document = Some(document);
            }
            window.request_redraw();

            anyhow::Result::Ok(())
        })()
        .log_if_error();
    }
}

pub async fn load_docx() -> anyhow::Result<state::Document> {
    let file = rfd::FileDialog::new()
        .set_title("Open a docx file...")
        .add_filter("", &["docx"])
        .pick_file()
        .context("Failed to pick file.")?;

    let archive = std::fs::read(file.clone()).context("Can't read archive")?;

    let document = get_element(&archive, "word/document.xml")?;
    let fonts = get_element(&archive, "word/fontTable.xml")?;

    Ok(state::Document {
        document: Arc::new(Box::new(
            (&document, &fonts)
                .try_into()
                .context("failed to parse docx documnet")?,
        )),
        zip_document: archive,
        path: file,
    })
}

fn get_element(archive: &Vec<u8>, file: &str) -> anyhow::Result<word_xml::WordXMLDocument> {
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

fn save_document(commands: DocumentCommands, window: Arc<Window>) -> impl FnOnce() {
    move || {
        (|| {
            let file = rfd::FileDialog::new()
                .set_title("Chose file to save the docx file...")
                .add_filter("", &["docx"])
                .set_can_create_directories(true)
                .save_file()
                .context("Failed to choose file to create")?;

            commands
                .lock()
                .to_anyhow()
                .context("[save document]")?
                .push(DocumentCommand::Save(file));

            window.request_redraw();

            Ok(())
        })()
        .log_if_error()
    }
}
