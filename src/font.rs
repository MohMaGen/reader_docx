use std::sync::OnceLock;

use anyhow::Context;
use fontconfig::Fontconfig;
use raylib::{RaylibHandle, RaylibThread};

static FC: OnceLock<Fontconfig> = OnceLock::new();

fn init_fc() -> anyhow::Result<Fontconfig> {
    Fontconfig::new().context("Can't intit fontconfig.")
}
pub fn find_font<'a> (
    rl: &'a mut RaylibHandle,
    tread: &'a RaylibThread,
    name: &'a str,
    style: Option<&'a str>,
) -> anyhow::Result<raylib::prelude::Font> {
    let fc = FC.get_or_try_init(init_fc)?;

    let font = fc.find(name, style).context(format!(
        "Can't find font {name} with style {style:?}, or it's alternatives"
    ))?;

    match rl.load_font(
        tread,
        font.path.to_str().context("Invalid font path encoding")?,
    ) {
        Ok(font) => Ok(font),
        Err(err) => Err(anyhow::Error::msg(err)),
    }
}
