use std::{collections::HashSet, str::FromStr};

use minidom::Element;

pub mod add_font;
pub mod content_tree;
pub mod display;
pub mod from_minidom;
pub mod from_word_xml;
pub mod getters;
pub mod parse_fonts;

pub use getters::SectrOfProperties;

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
    TodoWordXml(word_xml::Element)
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

impl std::fmt::Display for TextDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::LeftToRightTopToBottom => "lrTb",
                Self::LeftToRightBottomToTop => "lrBt",
                Self::RightToLeftTopToBottom => "rlTb",
                Self::RightToLeftBottomToTop => "rlBt",
            }
        )
    }
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

impl std::fmt::Display for FormProt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.val)
    }
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

impl std::fmt::Display for NumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumType::Decimal => write!(f, "decimal"),
        }
    }
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

impl std::fmt::Display for PageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NextPage => write!(f, "nextPage"),
        }
    }
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

#[derive(Debug, Clone, Default)]
pub enum LineRule {
    #[default]
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

impl std::fmt::Display for LineRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "auto")
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

impl std::fmt::Display for Justification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Justification::Start => write!(f, "start"),
            Justification::End => write!(f, "end"),
            Justification::Center => write!(f, "center"),
            Justification::Width => write!(f, "width"),
        }
    }
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

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TextProperties {
    pub font_handle: FontHandle,
    pub font_name: Option<String>,
    pub size: Option<TextSize>,
    pub size_cs: Option<TextSize>,
    pub weight: TextWeight,
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
    pub const BLACK: Self = Color {
        r: 0.,
        g: 0.,
        b: 0.,
        a: 1.,
    };

    #[inline]
    pub fn as_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    #[inline]
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn to_xml_val(&self) -> String {
        format!(
            "{:02X}{:02X}{:02X}",
            (self.r * u8::MAX as f32) as u8,
            (self.g * u8::MAX as f32) as u8,
            (self.b * u8::MAX as f32) as u8
        )
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

impl From<Color> for wgpu::Color {
    fn from(Color { r, g, b, a }: Color) -> Self {
        Self {
            r: r as f64,
            g: g as f64,
            b: b as f64,
            a: a as f64,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum TextWeight {
    #[default]
    Regular,
    Bold,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSize(pub f32);

impl std::fmt::Display for TextSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

impl DocxNode {
    pub fn is_paragraph(&self) -> bool {
        matches!(self, DocxNode::Paragrapth { .. })
    }
}

impl TextProperties {
    pub fn get_font_idx(&self) -> (String, String) {
        let mode = match (self.weight.clone(), self.italic) {
            (TextWeight::Bold, false) => String::from("Bold"),
            (TextWeight::Regular, false) => String::from("Regular"),
            (TextWeight::Regular, true) => String::from("Italic"),
            (TextWeight::Bold, true) => String::from("BoldItalic"),
        };

        (
            self.font_name.clone().unwrap_or("Default Font".into()),
            mode,
        )
    }
}
