use std::{collections::HashMap, ops::Range, sync::Arc};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    colorscheme::ColorScheme,
    docx_document::{self, Color, TextNode, TextProperties},
    draw::DrawState,
    font, math,
    primitives::{PlainTextProperties, Primitive, PrimitiveProperties},
};

#[derive(Debug)]
pub struct DocumentDraw {
    pub paragraphes: Vec<Paragraph>,
    pub fonts: HashMap<FontIdx, rusttype::Font<'static>>,
    pub scroll: f32,
    pub scale: f32,
    pub pages: Vec<Page>,
    pub bg_color: Color,
    pub cursor: Cursor,
}

#[derive(Debug, Default)]
pub enum Cursor {
    #[default]
    Normal,
    Edit,
    Select,
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
    pub properties: docx_document::ParagraphProperties,
    pub words: Vec<Word>,
    pub lines: Vec<Line>,
}

#[derive(Debug)]
pub struct Line {
    height: f32,
    min_width: f32,
    widht_with_spacing: f32,
    last_scale: f32,
    range: Range<usize>,
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
    scale: f32,
}

pub enum DocumentCommand {
    NewScroll(f32),
    DeltaScroll(f32),
    NewScale(f32),
    RatioScale(f32),
}

pub enum VerticalSpacing {
    Relative(f32),
    Absolute(f32),
}

impl DrawState<'_> {
    const DEFAULT_VERTICAL_SPACING: f32 = 1.0;
    const DEFAULT_SPACING_BEFORE: f32 = 20.;
    const DEFAULT_SPACING_AFTER: f32 = 20.;
    const DEFAULT_LINE_SPACING: f32 = 1.0;
    const PAGE_SPACE_BETWEEN: f32 = 100.;
    const DEFAULT_FONT_SIZE: f32 = 12.;

    pub fn new_document_draw(
        &self,
        colorscheme: ColorScheme,
        document: Arc<Box<docx_document::DocxDocument>>,
    ) -> anyhow::Result<DocumentDraw> {
        let mut document_draw = DocumentDraw::default();

        let page_properties = PageProperties::from(document.get_properties());
        let (v_width, _v_height) = (self.config.width as f32, self.config.height as f32);

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
            scale,
        };

        log::info!("page rect {page_rect:?}, page content rect {page_content_rect:?}");

        document_draw.pages = vec![first_page];
        document_draw.bg_color = bg_color;

        let Some(nodes) = &Arc::clone(&document).content.nodes else {
            return Ok(document_draw);
        };

        for paragraph in nodes.iter() {
            let docx_document::DocxNode::Paragrapth {
                properties, texts, ..
            } = paragraph
            else {
                continue;
            };
            let paragraph_tp = properties.text_properties.clone().unwrap_or_default();

            let delta = properties
                .spacing
                .before
                .unwrap_or(Self::DEFAULT_SPACING_BEFORE)
                * ctx.scale;

            self.vertical_offset_and_push(&mut ctx, &mut document_draw.pages, delta);

            let mut words = get_words(texts);

            self.create_words_prims(&mut words, &mut document_draw, paragraph_tp, &ctx)?;
            log::info!(
                "words {:?}",
                words
                    .iter()
                    .map(|word| word.word.clone())
                    .collect::<Vec<_>>()
            );

            let lines = get_lines(&words, &ctx, Self::DEFAULT_VERTICAL_SPACING * ctx.scale);

            for line in &lines {
                log::info!("line {line:?}");
                let (vertical_offset, vertical_space) = get_line_vertical_metrics(
                    properties.justify.clone(),
                    &ctx,
                    line,
                    Self::DEFAULT_VERTICAL_SPACING,
                );

                update_line(&mut words, line, &ctx, vertical_offset, vertical_space);

                let delta = (line.height
                    + line.last_scale
                        * properties
                            .spacing
                            .line
                            .unwrap_or(Self::DEFAULT_LINE_SPACING))
                    * ctx.scale;

                self.vertical_offset_and_push(&mut ctx, &mut document_draw.pages, delta);
            }

            document_draw.paragraphes.push(Paragraph {
                words,
                lines,
                properties: properties.clone(),
            });

            let delta = properties
                .spacing
                .after
                .unwrap_or(Self::DEFAULT_SPACING_AFTER)
                * ctx.scale;

            self.vertical_offset_and_push(&mut ctx, &mut document_draw.pages, delta);
        }

        Ok(document_draw)
    }

    fn create_words_prims<T: GetOrLoadFont>(
        &self,
        words: &mut [Word],
        fonts_collection: &mut T,
        paragraph_tp: TextProperties,
        ctx: &Context,
    ) -> Result<(), anyhow::Error> {
        for word in words.iter_mut() {
            for glyphs_view in word.glyphs_views.iter_mut() {
                let content = word.word[glyphs_view.word_range.clone()].to_string();

                let font =
                    fonts_collection.get_or_load_font(glyphs_view.properties.get_font_idx())?;

                let color = glyphs_view
                    .properties
                    .color
                    .unwrap_or(paragraph_tp.color.unwrap_or(Color::BLACK));

                let scale = ctx.scale
                    * 2.
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

                glyphs_view.primitive = self.new_prim(PlainTextProperties::new(
                    ((0., 0.), (0., scale)),
                    color,
                    content,
                    font,
                ));
            }
        }
        Ok(())
    }

    fn vertical_offset_and_push(&self, ctx: &mut Context, pages: &mut Vec<Page>, delta: f32) {
        if delta < ctx.page_content_rect.height() {
            ctx.page_content_rect = ctx.page_content_rect.move_left_top((0., delta));
        } else {
            let offset = ctx.page_rect.right_bottom.y + Self::PAGE_SPACE_BETWEEN * ctx.scale;
            let new_page = self.new_page_with_offset(
                &ctx.page_properties,
                ctx.v_width,
                ctx.bg_color,
                offset,
                ctx.scale,
            );

            ctx.page_rect = new_page.primitive.get_rect();
            ctx.page_content_rect = ctx.page_rect.add_paddings(ctx.page_properties.paddings);

            pages.push(new_page);
        }
    }

    pub fn update_document(&self, document_draw: &mut DocumentDraw) -> anyhow::Result<()> {
        let first_page = document_draw.pages.first().unwrap();
        let page_properties = first_page.page_properties.clone();
        let (v_width, _v_height) = (self.config.width as f32, self.config.height as f32);

        let bg_color = document_draw.bg_color;

        let scroll = document_draw.scroll;
        let scale = document_draw.scale;

        let first_page =
            self.new_page_with_offset(&page_properties, v_width, bg_color, scroll, scale);

        let page_rect = first_page.primitive.get_rect();
        let page_content_rect = page_rect.add_paddings(page_properties.paddings);

        document_draw.pages = vec![first_page];

        let mut ctx = Context {
            page_content_rect,
            page_properties,
            bg_color,
            page_rect,
            v_width,
            scale,
        };

        let paragraphs_len = document_draw.paragraphes.len();
        for (idx, paragraph) in document_draw.paragraphes.iter_mut().enumerate() {
            let properties = paragraph.properties.clone();

            if idx != 0 {
                let delta = properties
                    .spacing
                    .before
                    .unwrap_or(Self::DEFAULT_SPACING_BEFORE)
                    * ctx.scale;

                self.vertical_offset_and_push(&mut ctx, &mut document_draw.pages, delta);
            }

            log::info!("{:?}", paragraph.lines);
            for (idx, line) in paragraph.lines.iter().enumerate() {
                log::info!("{:?}", ctx.page_content_rect);

                let (vertical_offset, vertical_space) = get_line_vertical_metrics(
                    properties.justify.clone(),
                    &ctx,
                    line,
                    Self::DEFAULT_VERTICAL_SPACING,
                );

                update_line(
                    &mut paragraph.words,
                    line,
                    &ctx,
                    vertical_offset,
                    vertical_space,
                );

                if idx != paragraph.lines.len() - 1 {
                    let delta = properties
                        .spacing
                        .line
                        .map(|sp| sp * ctx.scale)
                        .unwrap_or(Self::DEFAULT_LINE_SPACING * line.height);

                    println!("DELTA {:?}", delta);
                    self.vertical_offset_and_push(&mut ctx, &mut document_draw.pages, delta);
                }
            }

            if idx != paragraphs_len - 1 {
                let delta = properties
                    .spacing
                    .after
                    .unwrap_or(Self::DEFAULT_SPACING_AFTER)
                    * ctx.scale;

                self.vertical_offset_and_push(&mut ctx, &mut document_draw.pages, delta);
            }
        }

        document_draw.for_prims_mut(|prim| {
            let prop = prim.prop.clone();
            self.update_prim(prop, prim);
        });
        Ok(())
    }

    pub fn process_document_command(
        &self,
        document_draw: &mut DocumentDraw,
        command: DocumentCommand,
    ) {
        match command {
            DocumentCommand::NewScroll(new_scroll) => {
                document_draw.scroll = new_scroll;
            }
            DocumentCommand::NewScale(scale) => {
                let scale = scale.max(0.1).min(2.);
                let ratio = scale / document_draw.scale;
                self.scale_by_ratio(document_draw, ratio);
                document_draw.scale = scale.max(0.1).min(2.);
            }
            DocumentCommand::DeltaScroll(delta) => document_draw.scroll += delta,
            DocumentCommand::RatioScale(ratio) => {
                let prev = document_draw.scale;
                document_draw.scale = (document_draw.scale * ratio).max(0.1).min(2.);
                let ratio = document_draw.scale / prev;
                self.scale_by_ratio(document_draw, ratio);
            }
        }
    }

    pub fn draw_document_draw<'a, 'b: 'a>(
        &'b self,
        rpass: &mut wgpu::RenderPass<'a>,
        document: &'a DocumentDraw,
    ) {
        document.for_prims(|prim| self.draw_prim(rpass, prim));
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

        Page {
            page_properties: page_properties.clone(),
            primitive: self.new_prim((
                math::Rectangle::new(((v_width - size.width) * 0.5, offset), size),
                bg_color,
            )),
        }
    }
}

impl DrawState<'_> {
    fn scale_by_ratio(&self, document_draw: &mut DocumentDraw, ratio: f32) {
        document_draw.paragraphes.iter_mut().for_each(|par| {
            par.words.iter_mut().for_each(|word| {
                word.glyphs_views.iter_mut().for_each(|gv| {
                    let prop = gv.primitive.prop.clone().scale(ratio);
                    self.update_prim(prop, &mut gv.primitive);
                })
            });
            par.lines.iter_mut().for_each(|line| {
                line.widht_with_spacing /= ratio;
                line.min_width /= ratio;
                line.height /= ratio;
                line.last_scale /= ratio;
            })
        });
    }
}
fn get_line_vertical_metrics(
    justification: Option<docx_document::Justification>,
    ctx: &Context,
    line: &Line,
    vertical_space: f32,
) -> (f32, VerticalSpacing) {
    use VerticalSpacing::*;
    match justification {
        Some(docx_document::Justification::Start) | None => (0f32, Relative(vertical_space)),
        Some(docx_document::Justification::Width) => (
            0f32,
            Absolute((ctx.page_content_rect.width() - line.min_width) / line.range.len() as f32),
        ),
        Some(docx_document::Justification::Center) => (
            (ctx.page_content_rect.width() - line.widht_with_spacing) / 2.,
            Relative(vertical_space),
        ),
        Some(docx_document::Justification::End) => (
            ctx.page_content_rect.width() - line.widht_with_spacing,
            Relative(vertical_space),
        ),
    }
}

fn update_line(
    words: &mut [Word],
    line: &Line,
    ctx: &Context,
    mut vertical_offset: f32,
    vertical_space: VerticalSpacing,
) {
    let mut last_scale = 1f32;
    for word in &mut words[line.range.clone()] {
        let mut prev_glyph = None;
        for glyphs_view in &mut word.glyphs_views {
            let math::Size { width, height } = glyphs_view.primitive.get_rect().size();

            let end = glyphs_view.word_range.end;
            let start = glyphs_view.word_range.start;
            let fst_char = word.word[start..=start].chars().next().unwrap_or_default();
            let last_char = word.word[(end - 1)..end].chars().next().unwrap_or_default();

            if let PrimitiveProperties::PlainText(PlainTextProperties {
                left_top,
                font,
                scale,
                ..
            }) = &mut glyphs_view.primitive.prop
            {
                *left_top = ctx.page_content_rect.left_top;

                if let Some(prev) = prev_glyph {
                    vertical_offset +=
                        font.pair_kerning(rusttype::Scale::uniform(*scale), prev, fst_char)
                            * ctx.scale;
                }
                left_top.x += vertical_offset;
                left_top.y += (line.height - height) * ctx.scale;
                vertical_offset += width;

                prev_glyph = Some(last_char);
                last_scale = *scale;
            }
        }
        match vertical_space {
            VerticalSpacing::Relative(vs) => vertical_offset += vs * last_scale,
            VerticalSpacing::Absolute(vs) => vertical_offset += vs,
        };
    }
}

fn get_lines(words: &[Word], ctx: &Context, vertical_space: f32) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut curr_line = Line {
        height: 0.,
        min_width: 0.,
        widht_with_spacing: 0.,
        last_scale: 0.,
        range: 0..0,
    };
    for word in words.iter() {
        let (widht, height, last_scale) = get_words_sizes(word);
        if curr_line.widht_with_spacing + widht > ctx.page_content_rect.width() {
            let end = curr_line.range.end;
            lines.push(curr_line);

            curr_line = Line {
                height,
                min_width: widht,
                widht_with_spacing: widht + vertical_space * last_scale,
                last_scale,
                range: end..(end + 1),
            };
            continue;
        }

        curr_line.min_width += widht;
        curr_line.height = curr_line.height.max(height);
        curr_line.widht_with_spacing += widht + vertical_space * last_scale;
        curr_line.last_scale = last_scale;
        curr_line.range.end += 1;
    }
    lines.push(curr_line);
    lines
}

fn get_words_sizes(word: &Word) -> (f32, f32, f32) {
    let (widht, height, last_scale) = word.glyphs_views.iter().fold(
        (0., 0., 0.),
        |(acc_width, acc_height, _last_scale), glyphs| {
            let math::Size { width, height } = glyphs.primitive.get_rect().size();
            (
                acc_width + width,
                height.max(acc_height),
                glyphs.primitive.get_scale(),
            )
        },
    );
    (widht, height, last_scale)
}

enum WordState {
    Finished(Word),
    Unfinished(Word),
}
fn get_words(texts: &[TextNode]) -> Vec<Word> {
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
                if grapheme.trim().is_empty() {
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

pub trait GetOrLoadFont {
    fn get_or_load_font(
        &mut self,
        idx: impl Into<FontIdx>,
    ) -> anyhow::Result<rusttype::Font<'static>>;
}

impl GetOrLoadFont for HashMap<FontIdx, rusttype::Font<'static>> {
    fn get_or_load_font(
        &mut self,
        idx: impl Into<FontIdx>,
    ) -> anyhow::Result<rusttype::Font<'static>> {
        let idx = idx.into();

        if let Some(font) = self.get(&idx) {
            Ok(font.clone())
        } else {
            let font = font::find_font(idx.name.as_str(), Some(idx.mode.as_str()))?;
            self.insert(idx, font.clone());
            Ok(font)
        }
    }
}

impl GetOrLoadFont for DocumentDraw {
    fn get_or_load_font(
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
}

impl DocumentDraw {
    pub fn prims(&self) -> PrimIter<'_> {
        PrimIter {
            document: self,
            state: PrimIterState::Pages(0),
        }
    }

    pub fn for_prims<'document>(&'document self, mut f: impl FnMut(&'document Primitive)) {
        for page in &self.pages {
            f(&page.primitive)
        }

        for par in &self.paragraphes {
            for word in &par.words {
                for glyphs_view in &word.glyphs_views {
                    f(&glyphs_view.primitive)
                }
            }
        }
    }

    pub fn for_prims_mut<'document>(
        &'document mut self,
        mut f: impl FnMut(&'document mut Primitive),
    ) {
        for page in &mut self.pages {
            f(&mut page.primitive)
        }

        for par in &mut self.paragraphes {
            for word in &mut par.words {
                for glyphs_view in &mut word.glyphs_views {
                    f(&mut glyphs_view.primitive)
                }
            }
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
            scroll: 100.,
            bg_color: Color::BLACK,
            scale: 0.5,
            cursor: Default::default(),
            pages: Default::default(),
            paragraphes: Default::default(),
            fonts: Default::default(),
        }
    }
}

impl From<(String, String)> for FontIdx {
    fn from((name, mode): (String, String)) -> Self {
        Self { name, mode }
    }
}
