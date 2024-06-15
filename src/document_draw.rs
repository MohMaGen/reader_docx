use std::{ops::Range, sync::Arc};

use crate::{
    colorscheme::ColorScheme,
    docx_document,
    draw::DrawState,
    math,
    primitives::Primitive,
};

#[derive(Debug)]
pub struct DocumentDraw {
    pub paragraphes: Vec<Paragraph>,
    pub scroll: f32,
    pub scale: f32,
    pub pages: Vec<Page>,
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

#[derive(Debug)]
pub struct Word {
    pub word: String,
    pub glyph_views: Vec<GlyphView>,
}

#[derive(Debug)]
pub struct GlyphView {
    pub word_range: Range<usize>,
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

impl DrawState<'_> {
    const DEFAULT_SPACING_BEFORE: f32 = 20.;
    const DEFAULT_SPACING_AFTER: f32 = 20.;
    const PAGE_SPACE_BETWEEN: f32 = 100.;

    pub fn new_document_draw(
        &self,
        colorscheme: ColorScheme,
        document: Arc<Box<docx_document::DocxDocument>>,
    ) -> DocumentDraw {
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
            return document_draw;
        };

        for paragraph in nodes.iter() {
            let docx_document::DocxNode::Paragrapth {
                properties,
                attrs,
                texts,
            } = paragraph
            else {
                continue;
            };

            let delta = properties
                .spacing
                .before
                .unwrap_or(Self::DEFAULT_SPACING_BEFORE);

            self.vertical_offset(&mut ctx, &mut document_draw, delta);

            let delta = properties
                .spacing
                .after
                .unwrap_or(Self::DEFAULT_SPACING_AFTER);

            self.vertical_offset(&mut ctx, &mut document_draw, delta);
        }

        document_draw
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

    pub fn update_document_draw(
        &self,
        document: Arc<Box<docx_document::DocxDocument>>,
        document_draw: &mut DocumentDraw,
        colorscheme: ColorScheme,
    ) {
        *document_draw = self.new_document_draw(colorscheme, document)
    }

    pub fn draw_document_draw<'a, 'b: 'a>(
        &'b self,
        rpass: &mut wgpu::RenderPass<'a>,
        document: &'a DocumentDraw,
    ) {
        document
            .pages
            .iter()
            .for_each(|page| self.draw_prim(rpass, &page.primitive))
    }


    fn new_page_with_offset(
        &self,
        page_properties: &PageProperties,
        v_width: f32,
        bg_color: docx_document::Color,
        offset: f32,
        scale: f32,
    ) -> Page {
        println!("AAAAA");
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
            scroll: 100.,
            scale: 0.5,
        }
    }
}
