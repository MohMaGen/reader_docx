use std::{collections::HashMap, ops::Range, path::PathBuf, sync::Arc};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    colorscheme::ColorScheme,
    docx_document::{self, Color, TextNode, TextProperties, TextSize},
    draw::DrawState,
    font::{self, find_font},
    math,
    primitives::{PlainTextProperties, Primitive},
};

#[derive(Debug)]
pub struct DocumentDraw {
    pub paragraphes: Vec<Paragraph>,
    pub fonts: HashMap<FontIdx, rusttype::Font<'static>>,
    pub scroll: f32,
    pub scale: f32,
    pub pages: Vec<Page>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct FontIdx {
    pub name: String,
    pub mode: String,
}

#[derive(Debug)]
pub struct Page {
    pub page_properties: PageProperties,
    pub primitive: Primitive,
}

#[derive(Debug)]
pub struct Paragraph {
    pub words: Vec<Word>,
}

#[derive(Debug, Default)]
pub struct Word {
    pub word: String,
    pub glyphs_views: Vec<GlyphsView>,
}

#[derive(Debug, Default)]
pub struct GlyphsView {
    pub word_range: Range<usize>,
    pub properties: TextProperties,
    pub glyphs: Vec<rusttype::PositionedGlyph<'static>>,
    pub primitive: Primitive,
}

struct Context {
    page_content_rect: math::Rectangle,
    page_properties: PageProperties,
    bg_color: docx_document::Color,
    page_rect: math::Rectangle,
    v_width: f32,
    scroll: f32,
    scale: f32,
}

pub enum DocumentCommand {
    NewScroll(f32),
    DeltaScroll(f32),
    NewScale(f32),
    RatioScale(f32),
}

impl DrawState<'_> {
    const DEFAULT_VERTICAL_SPACING: f32 = 1.5;
    const DEFAULT_SPACING_BEFORE: f32 = 20.;
    const DEFAULT_SPACING_AFTER: f32 = 20.;
    const PAGE_SPACE_BETWEEN: f32 = 100.;
    const DEFAULT_FONT_SIZE: f32 = 12.;

    pub fn new_document_draw(
        &self,
        colorscheme: ColorScheme,
        document: Arc<Box<docx_document::DocxDocument>>,
    ) -> anyhow::Result<DocumentDraw> {
        let mut document_draw = DocumentDraw::default();

        let page_properties = PageProperties::from(document.get_properties());
        let (v_width, v_height) = (self.config.width as f32, self.config.height as f32);

        let bg_color = colorscheme.page_bg_color;

        let scroll = document_draw.scroll;
        let scale = document_draw.scale;

        let first_page =
            self.new_page_with_offset(&page_properties, v_width, bg_color, scroll, scale);

        let page_rect = first_page.primitive.get_rect();
        let page_content_rect = page_rect.add_paddings(page_properties.paddings);

        let mut ctx = Context {
            page_content_rect,
            page_properties,
            bg_color,
            page_rect,
            v_width,
            scroll,
            scale,
        };

        document_draw.pages = vec![first_page];

        let Some(nodes) = &Arc::clone(&document).content.nodes else {
            return Ok(document_draw);
        };

        for paragraph in nodes.iter() {
            let docx_document::DocxNode::Paragrapth {
                properties,
                texts,
                ..
            } = paragraph
            else {
                continue;
            };
            let paragraph_tp = properties.text_properties.clone().unwrap_or_default();

            let delta = properties
                .spacing
                .before
                .unwrap_or(Self::DEFAULT_SPACING_BEFORE * ctx.scale);

            self.vertical_offset(&mut ctx, &mut document_draw, delta);

            let mut words = get_words(texts);

            self.create_words_prims(&mut words, &mut document_draw, paragraph_tp, &ctx)?;

            let lines = get_lines(words, &ctx);

            for line in lines.iter() {

            }

            let delta = properties
                .spacing
                .after
                .unwrap_or(Self::DEFAULT_SPACING_AFTER * ctx.scale);

            self.vertical_offset(&mut ctx, &mut document_draw, delta);
        }

        Ok(document_draw)
    }

    fn create_words_prims(
        &self,
        words: &mut Vec<Word>,
        document_draw: &mut DocumentDraw,
        paragraph_tp: TextProperties,
        ctx: &Context,
    ) -> Result<(), anyhow::Error> {
        Ok(for word in words.iter_mut() {
            for glyphs_view in word.glyphs_views.iter_mut() {
                let content = (&word.word[glyphs_view.word_range.clone()]).to_string();

                let font = document_draw.get_or_load_font(glyphs_view.properties.get_font_idx())?;

                let color = glyphs_view
                    .properties
                    .color
                    .unwrap_or(paragraph_tp.color.unwrap_or(Color::BLACK));

                let left_top = (0., 0.).into();

                let scale = ctx.scale
                    * glyphs_view
                        .properties
                        .size
                        .clone()
                        .map(|sz| sz.0)
                        .unwrap_or(
                            paragraph_tp
                                .size
                                .clone()
                                .map(|sz| sz.0)
                                .unwrap_or(Self::DEFAULT_FONT_SIZE),
                        );

                glyphs_view.primitive = self.new_prim(PlainTextProperties {
                    left_top,
                    content,
                    font,
                    color,
                    scale,
                });
            }
        })
    }

    fn vertical_offset(&self, ctx: &mut Context, document_draw: &mut DocumentDraw, delta: f32) {
        if delta < ctx.page_content_rect.height() {
            ctx.page_content_rect = ctx.page_content_rect.move_left_top((0., delta));
        } else {
            let offset = ctx.scroll + ctx.page_rect.right_bottom.y + Self::PAGE_SPACE_BETWEEN;
            let new_page = self.new_page_with_offset(
                &ctx.page_properties,
                ctx.v_width,
                ctx.bg_color,
                offset,
                ctx.scale,
            );

            ctx.page_rect = new_page.primitive.get_rect();
            ctx.page_content_rect = ctx.page_rect.add_paddings(ctx.page_properties.paddings);

            document_draw.pages.push(new_page);
        }
    }

    pub fn update_document(&self, document_draw: &mut DocumentDraw) {
        document_draw.for_prims_mut(|prim| self.update_prim(prim.prop.clone(), prim));
    }

    pub fn process_document_command(
        &self,
        document_draw: &mut DocumentDraw,
        command: DocumentCommand,
    ) {
        match command {
            DocumentCommand::NewScroll(new_scroll) => {
                let scroll = new_scroll - document_draw.scroll;
                self.delta_scroll(document_draw, scroll);
            }
            DocumentCommand::NewScale(scale) => {
                let scale = scale / document_draw.scale;
                self.ratio_scroll(document_draw, scale);
            }
            DocumentCommand::DeltaScroll(delta) => self.delta_scroll(document_draw, delta),
            DocumentCommand::RatioScale(ratio) => self.ratio_scroll(document_draw, ratio),
        }
    }

    fn ratio_scroll(&self, document_draw: &mut DocumentDraw, scale: f32) {
        document_draw.for_prims_mut(|prim| {
            let new_prop = prim.prop.clone().scale(scale);
            self.update_prim(new_prop, prim);
        })
    }

    fn delta_scroll(&self, document_draw: &mut DocumentDraw, scroll: f32) {
        document_draw.for_prims_mut(|prim| {
            let new_prop = prim.prop.clone().scroll(scroll);
            self.update_prim(new_prop, prim);
        })
    }

    pub fn draw_document_draw<'a, 'b: 'a>(
        &'b self,
        rpass: &mut wgpu::RenderPass<'a>,
        document: &'a DocumentDraw,
    ) {
        document
            .prims()
            .for_each(|prim| self.draw_prim(rpass, prim));
    }

    fn new_page_with_offset(
        &self,
        page_properties: &PageProperties,
        v_width: f32,
        bg_color: docx_document::Color,
        offset: f32,
        scale: f32,
    ) -> Page {
        let size = page_properties.clone().size * scale;

        let first_page = Page {
            page_properties: page_properties.clone(),
            primitive: self.new_prim((
                math::Rectangle::new(((v_width - size.width) * 0.5, offset), size),
                bg_color,
            )),
        };
        first_page
    }
}

fn get_lines(words: Vec<Word>, ctx: &Context) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut curr_line = Line {
        height: 0.,
        min_width: 0.,
        range: 0..0,
    };
    for word in words.iter() {
        let (widht, height) = get_words_sizes(word);
        if curr_line.min_width + widht + Self::DEFAULT_VERTICAL_SPACING * ctx.scale
            > ctx.page_content_rect.width()
        {
            let end = curr_line.range.end;
            lines.push(curr_line);

            curr_line = Line {
                height,
                min_width: widht,
                range: end..(end + 1),
            };
            continue;
        }

        curr_line.min_width += widht;
        curr_line.height = curr_line.height.max(height);
        curr_line.range.end += 1;
    }
    lines
}

fn get_words_sizes(word: &Word) -> (f32, f32) {
    let (widht, height) =
        word.glyphs_views
            .iter()
            .fold((0., 0.), |(acc_width, acc_height), glyphs| {
                let math::Size { width, height } = glyphs.primitive.get_rect().size();
                (acc_width + width, height.max(acc_height))
            });
    (widht, height)
}
            struct Line {
                height: f32,
                min_width: f32,
                range: Range<usize>,
            }

enum WordState {
    Finished(Word),
    Unfinished(Word),
}
fn get_words(texts: &Vec<TextNode>) -> Vec<Word> {
    use WordState::*;
    texts
        .iter()
        .fold(Vec::new(), |mut words, text| {
            let mut curr_word = Word::default();
            match words.pop() {
                Some(Unfinished(word)) => curr_word = word,
                Some(Finished(word)) => words.push(Finished(word)),
                _ => {}
            }

            let TextNode {
                properties,
                content,
            } = text;

            for grapheme in content.graphemes(true) {
                if grapheme.trim().len() == 0 {
                    finish_curr_word(&mut words, &mut curr_word);
                } else {
                    push_grapheme_to_curr_word(properties, &mut curr_word, grapheme);
                }
            }

            words.push(Unfinished(curr_word));

            words
        })
        .iter()
        .map(|word| match word {
            Finished(word) | Unfinished(word) => word.clone_without_primitive(),
        })
        .collect()
}

fn push_grapheme_to_curr_word(properties: &TextProperties, curr_word: &mut Word, g: &str) {
    let properties = properties.clone();

    curr_word.word.push_str(g);
    if let Some(last_glyphs_view) = curr_word.glyphs_views.last_mut() {
        if last_glyphs_view.properties == properties {
            last_glyphs_view.word_range.end += 1;
        } else {
            let last = last_glyphs_view.word_range.end;
            curr_word.glyphs_views.push(GlyphsView {
                word_range: last..(last + 1),
                properties,
                ..Default::default()
            })
        }
    } else {
        curr_word.glyphs_views.push(GlyphsView {
            properties,
            word_range: 0..1,
            ..Default::default()
        })
    }
}

fn finish_curr_word(words: &mut Vec<WordState>, curr_word: &mut Word) {
    use WordState::*;
    words.push(Finished(curr_word.clone_without_primitive()));
    *curr_word = Word::default();
}

impl Word {
    fn clone_without_primitive(&self) -> Word {
        Word {
            word: self.word.clone(),
            glyphs_views: self
                .glyphs_views
                .iter()
                .map(|glyphs_view| GlyphsView {
                    word_range: glyphs_view.word_range.clone(),
                    properties: glyphs_view.properties.clone(),
                    ..Default::default()
                })
                .collect(),
        }
    }
}

impl DocumentDraw {
    pub fn get_or_load_font(
        &mut self,
        idx: impl Into<FontIdx>,
    ) -> anyhow::Result<rusttype::Font<'static>> {
        let idx = idx.into();

        if let Some(font) = self.fonts.get(&idx) {
            Ok(font.clone())
        } else {
            let font = font::find_font(idx.name.as_str(), Some(idx.mode.as_str()))?;
            self.fonts.insert(idx, font.clone());
            Ok(font)
        }
    }

    pub fn prims<'document>(&'document self) -> PrimIter<'document> {
        PrimIter {
            document: self,
            state: PrimIterState::Pages(0),
        }
    }

    pub fn for_prims_mut<'document>(
        &'document mut self,
        mut f: impl FnMut(&'document mut Primitive) -> (),
    ) {
        for page in self.pages.iter_mut() {
            f(&mut page.primitive)
        }
    }
}

pub struct PrimIter<'document> {
    document: &'document DocumentDraw,
    state: PrimIterState,
}

pub enum PrimIterState {
    Pages(usize),
}

impl<'a> Iterator for PrimIter<'a> {
    type Item = &'a Primitive;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            PrimIterState::Pages(curr) => {
                if curr >= self.document.pages.len() {
                    return None;
                }
                self.state = PrimIterState::Pages(curr + 1);

                Some(&self.document.pages[curr].primitive)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct PageProperties {
    pub size: math::Size,
    pub paddings: math::Paddings,
}

impl From<Option<docx_document::SectrOfProperties>> for PageProperties {
    fn from(page_properties: Option<docx_document::SectrOfProperties>) -> Self {
        if let Some(page_properties) = page_properties {
            Self {
                size: page_properties.get_size().into(),
                paddings: page_properties.get_margins().into(),
            }
        } else {
            PageProperties::default()
        }
    }
}

impl Default for PageProperties {
    fn default() -> Self {
        Self {
            size: (100., 100.).into(),
            paddings: (10.).into(),
        }
    }
}

impl Default for DocumentDraw {
    fn default() -> Self {
        Self {
            pages: Default::default(),
            paragraphes: Default::default(),
            fonts: Default::default(),
            scroll: 100.,
            scale: 0.5,
        }
    }
}

impl From<(String, String)> for FontIdx {
    fn from((name, mode): (String, String)) -> Self {
        Self { name, mode }
    }
}
