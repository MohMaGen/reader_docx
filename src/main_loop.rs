use super::Command;
use super::Fonts;
use super::State;
use crate::draw;
use crate::update_events;
use crate::AsAnyhow;
use crate::StateMutex;
use anyhow::Context;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::EventPump;
use std::sync::Arc;
use std::sync::Mutex;

pub(crate) fn main_loop(
    state: Arc<Mutex<State>>,
    event_pump: &mut EventPump,
    commands: Arc<Mutex<Vec<Command>>>,
    canvas: &mut Canvas<Window>,
    fonts: &Fonts<'_, '_>,
) -> anyhow::Result<bool> {
    if Arc::clone(&state).should_exit()? {
        return Ok(true);
    }

    let mut new_commands = update_events::update_events(Arc::clone(&state), event_pump)?;
    {
        commands.lock().as_anyhow()?.append(&mut new_commands);
    }

    let state_copy = state.get_copy()?;

    draw::draw(canvas, &state_copy, &fonts).context("while do draw state")?;
    Ok(false)
}
