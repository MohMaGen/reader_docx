use std::collections::HashSet;

use minidom::Element;
use raylib::text::Font;

pub mod from_minidom;
pub mod parse_fonts;
pub mod content_tree;
pub mod add_font;

#[derive(Default)] pub struct DocxDocument { pub fonts: FontTable,
    pub content: ContentTree,
}

#[derive(Default)]
pub struct FontTable {
    pub fonts: Vec<FontProperties>,
}

pub type FontHandle = usize;

#[derive(Default)]
pub struct FontProperties {
    pub name: String,
    pub variants: HashSet<i32>,
}

#[derive(Default)]
pub struct FontVariant {
    pub font_size: usize,
    pub chars: Vec<i32>,
    pub state: FontState,
}

#[derive(Default)]
pub enum FontState {
    #[default]
    NotLoaded,
    Loaded(Font),
}

#[derive(Default)]
pub struct ContentTree {
    pub nodes: Option<Vec<Box<DocxNode>>>
}

pub enum DocxNode {
    Paragrapth {
        properties: ParagrapthProperties,
        attrs: Vec<(String, String)>,
        texts: Vec<TextNode>,
    },
    Todo(Element),
}

pub struct TextNode {
    properties: TextProperties,
    content: String,
}

#[derive(Default)]
pub struct ParagrapthProperties {
    pub justify: Justification,
    pub text_properties: TextProperties,
}

#[derive(Default)]
pub enum Justification {
    #[default]
    Left,
    Right,
    Center,
    Width,
}

#[derive(Default)]
pub struct TextProperties {
    pub font_handle: FontHandle,
    pub size: TextSize,
    pub size_cs: TextSize,
    pub width: TextWidth,
}

#[derive(Default)]
pub enum TextWidth {
    #[default]
    Regular,
    Bold,
}

pub struct TextSize(pub f32);

impl Default for TextSize {
    fn default() -> Self {
        Self(12.0)
    }
}

impl From<i32> for TextSize {
    fn from(value: i32) -> Self {
        Self(value as f32 / 2.0)
    }
}
