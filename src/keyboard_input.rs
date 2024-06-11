use std::sync::{Arc, Mutex};

use winit::event::ElementState;

use crate::{
    state::{Mode, State},
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

            process_command_input(event, state)?;
        }
        _ => {}
    }
    Ok(())
}

fn process_command_input(
    event: winit::event::KeyEvent,
    state: Arc<Mutex<State>>,
) -> Result<(), anyhow::Error> {
    Ok(match event.text {
        Some(s) => {

            let mut state = state.lock().as_anyhow()?;
            state.console_input = format!("{}{}", state.console_input, s);
        }
        _ => {}
    })
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
