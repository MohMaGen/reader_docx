use crate::docx_document::getters::SectrOfProperties;
use crate::docx_document::TextNode;
use crate::traits::{AllSame, MakeWider, Scale};
use crate::{docx_document, UiMode};

use super::DocxDocument;
use iced::advanced::{renderer, text, Widget};
use iced::event::Status;
use iced::Length;
use iced::{Background, Color, Element};

pub struct DocxEditor<'a> {
    pub document: &'a DocxDocument,
    pub mode: UiMode,
    pub cursor: Cursor,
    pub scale: f32,
    pub width: Length,
    pub height: Length,
    pub on_edit: Option<Box<dyn Fn(DocxAction) -> super::Message + 'a>>,
}

impl<'a> DocxEditor<'a> {
    pub fn new(document: &'a DocxDocument, mode: UiMode) -> Self {
        Self {
            cursor: Cursor::new(0, 0, 0),
            document,
            width: Length::Fill,
            height: Length::Fill,
            on_edit: None,
            scale: 0.2,
            mode,
        }
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn on_action(mut self, on_edit: impl Fn(DocxAction) -> super::Message + 'a) -> Self {
        self.on_edit = Some(Box::new(on_edit));
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cursor {
    pub paragraph: usize,
    pub text: usize,
    pub grapheme: usize,
}

impl Cursor {
    pub fn new(paragraph: usize, text: usize, grapheme: usize) -> Self {
        Self {
            paragraph,
            text,
            grapheme,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocxAction {}

impl<'a, Renderer, Theme> Widget<super::Message, Theme, Renderer> for DocxEditor<'a>
where
    Renderer: iced::advanced::text::Renderer,
{
    fn size(&self) -> iced::Size<Length> {
        iced::Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        _tree: &mut iced::advanced::widget::Tree,
        _renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let limits = limits.height(self.height);

        iced::advanced::layout::Node::new(limits.max())
    }

    fn on_event(
        &mut self,
        _state: &mut iced::advanced::widget::Tree,
        _event: iced::Event,
        _layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        _shell: &mut iced::advanced::Shell<'_, super::Message>,
        _viewport: &iced::Rectangle,
    ) -> iced::advanced::graphics::core::event::Status {
        let Some(_on_edit) = self.on_edit.as_ref() else {
            return Status::Ignored;
        };

        Status::Captured
    }

    fn draw(
        &self,
        _tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let bounds = layout.bounds();

        renderer.fill_quad(
            renderer::Quad {
                bounds: bounds.make_wider(-10.),
                border: iced::Border::with_radius(10.),
                ..renderer::Quad::default()
            },
            Background::Color(Color::from_rgb(0.5, 0.5, 0.5)),
        );

        let Some(SectrOfProperties {
            page_type,
            page_size,
            page_margin,
            page_num_type,
            form_prot,
            text_direction,
            document_grid,
        }) = self
            .document
            .get_properties()
            .map(|props| props.scale(self.scale))
        else {
            return;
        };

        let mut page_inner_bounds = draw_page(
            iced::Point {
                x: bounds.center().x,
                y: bounds.y + 100.,
            },
            page_size,
            page_margin,
            self.scale,
            renderer,
        );

        self.document.content.nodes.as_ref().map(|nodes| {
            nodes.iter().for_each(|node| match node {
                docx_document::DocxNode::Paragrapth {
                    properties,
                    attrs,
                    texts,
                } => {
                    let content = texts
                        .iter()
                        .fold(String::new(), |acc, TextNode { content, .. }| {
                            format!("{}{}", acc, content)
                        });
                    
                }
                docx_document::DocxNode::SectrOfProperties { .. } => {}
                docx_document::DocxNode::Todo(_) => {}
            })
        });
    }
}

fn draw_page<Renderer: renderer::Renderer>(
    top_center: iced::Point<f32>,
    page_size: docx_document::PageSize,
    page_margin: docx_document::PageMargin,
    scale: f32,
    renderer: &mut Renderer,
) -> iced::Rectangle {
    let page_bounds = iced::Rectangle {
        x: top_center.x - page_size.width * 0.5,
        y: top_center.y,
        width: page_size.width,
        height: page_size.height,
    };
    renderer.fill_quad(
        renderer::Quad {
            bounds: page_bounds,
            border: iced::Border::with_radius(10. * scale),
            ..renderer::Quad::default()
        },
        Background::Color(Color::WHITE),
    );

    iced::Rectangle {
        x: page_bounds.x + page_margin.left,
        y: page_bounds.x + page_margin.top,
        width: page_bounds.width - page_margin.left - page_margin.right,
        height: page_bounds.height - page_margin.top - page_margin.bottom,
    }
}

impl<'a, Theme, Renderer> From<DocxEditor<'a>> for Element<'a, super::Message, Theme, Renderer>
where
    Renderer: iced::advanced::text::Renderer,
{
    fn from(docs_editor: DocxEditor<'a>) -> Self {
        Self::new(docs_editor)
    }
}
