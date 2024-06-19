use anyhow::Context;

pub fn find_font<'a>(
    name: &'a str,
    style: Option<&'a str>,
) -> anyhow::Result<rusttype::Font<'static>> {
    let builder = font_loader::system_fonts::FontPropertyBuilder::new().family(name);

    let builder = match style {
        Some("Italic") => builder.italic(),
        Some("Bold") => builder.bold(),
        _ => builder,
    };

    let property = builder.build();
    let (bytes, index) = font_loader::system_fonts::get(&property).context("Failed to get font")?;

    Ok(rusttype::Font::try_from_vec_and_index(bytes, index as u32)
        .context("Failed to create font")?)
}
