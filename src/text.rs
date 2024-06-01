use crate::docx_document::Justification;
use sdl2::surface::Surface;


pub struct Paragraph<'a> {
    pub justification: Justification,
    pub texts: Vec<TextInstance<'a>>,
}

pub struct TextInstance<'a> {
    pub content: String,
    pub font: super::FontHandle,
    pub color: sdl2::pixels::Color,
    pub texture: Surface<'a>,
}
