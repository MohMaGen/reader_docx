use anyhow::Context;
use fontconfig::Fontconfig;
use std::{io::Read, sync::OnceLock};

static FC: OnceLock<Fontconfig> = OnceLock::new();

fn init_fc() -> anyhow::Result<Fontconfig> {
    Fontconfig::new().context("Can't innit fontconfig.")
}

pub fn find_font<'a>(
    name: &'a str,
    style: Option<&'a str>,
) -> anyhow::Result<rusttype::Font<'static>> {
    let fc = FC.get_or_try_init(init_fc)?;

    let font = fc.find(name, style).context(format!(
        "Can't find font {name} with style {style:?}, or it's alternatives"
    ))?;

    let mut bytes = Vec::new();
    std::fs::File::open(font.path)
        .context("Failed to read font's file.")?
        .read(&mut bytes)?;

    if let Some(index) = font.index {
        rusttype::Font::try_from_vec_and_index(bytes, index as u32)
            .context("Failed to load font with index")
    } else {
        rusttype::Font::try_from_vec(bytes).context("Failed to load font")
    }
}
