use std::ops::Range;

use crate::primitives::Primitive;

pub struct DocumentDraw {
    pub paragraphes: Vec<Paragraph>
}

pub struct Paragraph {
    pub words: Vec<Word>,
}

pub struct Word {
    pub word: String,
    pub glyph_views: Vec<GlyphView>
}

pub struct GlyphView {
    pub word_range: Range<usize>,
    pub glyphs: Vec<rusttype::PositionedGlyph<'static>>,
    pub primitive: Primitive,
}
