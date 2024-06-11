use std::{collections::HashSet, str::FromStr};

use minidom::Element;

pub mod add_font;
pub mod content_tree;
pub mod display;
pub mod from_minidom;
pub mod getters;
pub mod parse_fonts;

#[derive(Default, Debug)]
pub struct DocxDocument {
    pub fonts: FontTable,
    pub content: ContentTree,
}

#[derive(Default, Debug)]
pub struct FontTable {
    pub fonts: Vec<FontProperties>,
    pub default_font: FontHandle,
}

pub type FontHandle = usize;

#[derive(Default, Debug)]
pub struct FontProperties {
    pub name: String,
    pub variants: HashSet<i32>,
}

#[derive(Default, Debug)]
pub struct FontVariant {
    pub font_size: usize,
    pub chars: Vec<i32>,
}

#[derive(Default, Debug, Clone)]
pub struct ContentTree {
    pub nodes: Option<Vec<DocxNode>>,
}

#[derive(Debug, Clone)]
pub enum DocxNode {
    Paragrapth {
        properties: ParagraphProperties,
        attrs: Vec<(String, String)>,
        texts: Vec<TextNode>,
    },
    SectrOfProperties {
        page_type: Option<PageType>,
        page_size: PageSize,
        page_margin: PageMargin,
        page_num_type: Option<NumType>,
        form_prot: Option<FormProt>,
        text_direction: TextDirection,
        document_grid: Option<DocumentGrid>,
    },
    Todo(Element),
}

#[derive(Debug, Clone)]
pub struct PageSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct DocumentGrid {
    pub char_space: u64,
    pub line_pitch: u64,
    pub grid_type: GridType,
}

#[derive(Debug, Default, Clone)]
pub enum GridType {
    #[default]
    Default,
}

impl FromStr for GridType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" => Ok(GridType::Default),
            _ => Err(anyhow::Error::msg("invalid grid type")),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum TextDirection {
    #[default]
    LeftToRightTopToBottom,
    LeftToRightBottomToTop,
    RightToLeftTopToBottom,
    RightToLeftBottomToTop,
}

impl FromStr for TextDirection {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lrTb" => Ok(Self::LeftToRightBottomToTop),
            "lrBt" => Ok(Self::LeftToRightTopToBottom),
            "rlTb" => Ok(Self::RightToLeftBottomToTop),
            "rlBt" => Ok(Self::RightToLeftTopToBottom),
            _ => Err(anyhow::Error::msg("invalid text direction")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FormProt {
    pub val: bool,
}
impl FromStr for FormProt {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "false" => Ok(FormProt { val: false }),
            "true" => Ok(FormProt { val: true }),
            _ => Err(anyhow::Error::msg("Ivalid from prot")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum NumType {
    Decimal,
}

impl FromStr for NumType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "decimal" => Ok(Self::Decimal),
            _ => Err(anyhow::Error::msg("Ivalid num type")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PageMargin {
    pub footer: f32,
    pub gutter: f32,
    pub header: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
    pub top: f32,
}

#[derive(Debug, Clone)]
pub enum PageType {
    NextPage,
}

impl FromStr for PageType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "nextPage" => Ok(Self::NextPage),
            _ => Err(anyhow::Error::msg(format!("Invalid page type: {:?}", s))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextNode {
    pub properties: TextProperties,
    pub content: String,
}

#[derive(Default, Debug, Clone)]
pub struct ParagraphProperties {
    pub justify: Option<Justification>,
    pub text_properties: Option<TextProperties>,
    pub spacing: SpacingProperties,
}

#[derive(Debug, Default, Clone)]
pub struct SpacingProperties {
    pub line: Option<f32>,
    pub line_rule: Option<LineRule>,
    pub after: Option<f32>,
    pub before: Option<f32>,
}

#[derive(Debug, Clone)]
pub enum LineRule {
    Auto,
}

impl FromStr for LineRule {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(LineRule::Auto),
            _ => Err(anyhow::Error::msg("Invalid line rule.")),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub enum Justification {
    #[default]
    Start,
    End,
    Center,
    Width,
}

impl FromStr for Justification {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "start" => Ok(Justification::Start),
            "end" => Ok(Justification::End),
            "width" => Ok(Justification::Width),
            "center" => Ok(Justification::Center),
            _ => Err(anyhow::Error::msg("invalid justification")),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct TextProperties {
    pub font_handle: FontHandle,
    pub size: Option<TextSize>,
    pub size_cs: Option<TextSize>,
    pub width: TextWidth,
    pub color: Option<Color>,
    pub underline: bool,
    pub italic: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl FromStr for Color {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num = u32::from_str_radix(s, 16)?.to_be_bytes();

        Ok(Self {
            r: num[1] as f32 / u8::MAX as f32,
            g: num[2] as f32 / u8::MAX as f32,
            b: num[3] as f32 / u8::MAX as f32,
            a: 1.,
        })
    }
}

impl Color {
    pub fn as_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}

impl From<(f32, f32, f32)> for Color {
    fn from((r, g, b): (f32, f32, f32)) -> Self {
        Self::rgb(r, g, b)
    }
}

impl From<u32> for Color {
    fn from(hex: u32) -> Self {
        let num = hex.to_be_bytes();
        Self {
            r: num[0] as f32 / u8::MAX as f32,
            g: num[1] as f32 / u8::MAX as f32,
            b: num[2] as f32 / u8::MAX as f32,
            a: num[3] as f32 / u8::MAX as f32,
        }
    }
}


#[derive(Default, Debug, Clone)]
pub enum TextWidth {
    #[default]
    Regular,
    Bold,
}

#[derive(Debug, Clone)]
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
