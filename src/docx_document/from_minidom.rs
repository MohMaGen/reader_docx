use std::str::FromStr;

use anyhow::Context;
use minidom::{Element, NSChoice};

use crate::docx_document::{
    DocumentGrid, FormProt, GridType, Justification, LineRule, NumType, PageMargin, PageSize,
    ParagraphProperties, SpacingProperties, TextDirection,
};

use super::{ColorValue, DocxDocument, FontTable, TextSize, TextWidth};

impl<'a> TryFrom<(&'a minidom::Element, &'a minidom::Element)> for DocxDocument {
    type Error = anyhow::Error;

    fn try_from(
        (root, fonts): (&'a minidom::Element, &'a minidom::Element),
    ) -> Result<Self, Self::Error> {
        if root.name() != "document" {
            return Err(anyhow::Error::msg(
                "Can't create doc document cause root isn't document node.",
            ));
        }

        let mut document = DocxDocument::default();
        document.fonts = FontTable::try_from(fonts)?;
        let body = root
            .get_child_ans("body")
            .context("Document must containt body.")?;

        let default_chars = String::from("ABOBA");

        use super::DocxNode::Todo;
        for root_element in body.children() {
            let root_element = Box::new(match root_element.name() {
                "p" => parse_paragraph(root_element, &mut document, &default_chars),
                "sectPr" => parse_sectr_properties(root_element)?,
                _ => Todo(root_element.clone()),
            });
            document.content.push(root_element);
        }

        Ok(document)
    }
}

#[inline]
fn parse_paragraph(
    root_element: &Element,
    document: &mut DocxDocument,
    default_chars: &String,
) -> super::DocxNode {
    use super::DocxNode::Paragrapth;
    Paragrapth {
        properties: parse_paragraph_properties(root_element, document, default_chars),
        attrs: root_element
            .attrs()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect(),
        texts: get_texts_of_element(root_element, document),
    }
}

#[inline]
fn parse_paragraph_properties(
    root_element: &Element,
    document: &mut DocxDocument,
    default_chars: &String,
) -> ParagraphProperties {
    let Some(ppr) = root_element.get_child_ans("pPr") else {
        return ParagraphProperties::default();
    };
    ParagraphProperties {
        justify: ppr.get_childs_attr::<Justification>("jc", "w:val"),
        text_properties: parse_text_properties(ppr, document, default_chars),
        spacing: parce_spacing(ppr),
    }
}

fn parce_spacing(ppr: &Element) -> SpacingProperties {
    SpacingProperties {
        line: parse_float_as_some(ppr, "spacing", "w:line"),
        line_rule: ppr.get_childs_attr::<LineRule>("spacing", "w:lineRule"),
        after: parse_float_as_some(ppr, "spacing", "w:after"),
        before: parse_float_as_some(ppr, "spacing", "w:before"),
    }
}

fn parse_float_as_some(ppr: &Element, name: &str, attr: &str) -> Option<f32> {
    ppr.get_childs_attr::<u64>(name, attr)
        .map(|v| v as f32 * 0.1)
}

fn parse_sectr_properties(
    root_element: &Element,
) -> Result<super::DocxNode, <DocxDocument as TryFrom<(&Element, &Element)>>::Error> {
    use super::DocxNode::SectrOfProperties;
    Ok(SectrOfProperties {
        page_type: root_element
            .get_childs_attr("type", "w:val")
            .context(format!("can't parse page type: `{:?}`", root_element))?,

        page_size: PageSize {
            width: get_float(root_element, "pgSz", "w:w").context("widht")?,
            height: get_float(root_element, "pgSz", "w:h").context("height")?,
        },
        page_margin: PageMargin {
            footer: get_float(root_element, "pgMar", "w:footer").context("footer")?,
            gutter: get_float(root_element, "pgMar", "w:gutter").context("gutter")?,
            header: get_float(root_element, "pgMar", "w:header").context("header")?,
            bottom: get_float(root_element, "pgMar", "w:bottom").context("bottom")?,
            left: get_float(root_element, "pgMar", "w:left").context("left")?,
            right: get_float(root_element, "pgMar", "w:right").context("right")?,
            top: get_float(root_element, "pgMar", "w:top").context("top")?,
        },
        page_num_type: root_element
            .get_childs_attr::<NumType>("pgNumType", "w:fmt")
            .context("can't get page num type")?,
        form_prot: root_element
            .get_childs_attr::<FormProt>("formProt", "w:val")
            .context("can't get page form prot")?,
        text_direction: root_element
            .get_childs_attr::<TextDirection>("textDirection", "w:val")
            .context("can't get text direction")?,
        document_grid: DocumentGrid {
            char_space: root_element
                .get_childs_attr::<u64>("docGrid", "w:charSpace")
                .context("can't get text direction")?,
            line_pitch: root_element
                .get_childs_attr::<u64>("docGrid", "w:linePitch")
                .context("can't get text direction")?,
            grid_type: root_element
                .get_childs_attr::<GridType>("docGrid", "w:type")
                .context("can't get text direction")?,
        },
    })
}

#[inline]
fn get_float(root_element: &Element, name: &str, attr: &str) -> Result<f32, anyhow::Error> {
    Ok(root_element
        .get_childs_attr::<u64>(name, attr)
        .context("can't parse float")? as f32
        / 10.)
}

impl<'a> TryFrom<&'a minidom::Element> for FontTable {
    type Error = anyhow::Error;

    fn try_from(fonts: &'a minidom::Element) -> Result<Self, Self::Error> {
        let mut table = FontTable::default();

        for font in fonts.children().filter(|tag| tag.name() == "font") {
            let name = font
                .get_attr::<String>("w:name")
                .context("Font must have name")?;
            table.init_or_push_to_font(name, "".to_string());
        }

        Ok(table)
    }
}

#[inline]
fn get_texts_of_element(
    root_element: &minidom::Element,
    document: &mut DocxDocument,
) -> Vec<super::TextNode> {
    root_element
        .children()
        .filter(|tag| tag.name() == "r")
        .filter_map(|r_tag| {
            let content = r_tag.get_texts()?;

            let properties = parse_text_properties(r_tag, document, &content)?;

            Some(super::TextNode {
                properties,
                content,
            })
        })
        .collect()
}

#[inline]
fn parse_text_properties(
    parent_tag: &Element,
    document: &mut DocxDocument,
    content: &String,
) -> Option<super::TextProperties> {
    let rpr = parent_tag.get_child_ans("rPr")?;

    let size = rpr
        .get_childs_attr::<i32>("sz", "w:val")
        .map(TextSize::from);

    let size_cs = rpr
        .get_childs_attr::<i32>("szCs", "w:val")
        .map(TextSize::from);

    let font_handle = if let Some(font_name) = rpr.get_childs_attr::<String>("rFonts", "w:ascii") {
        document.init_or_push_to_font(font_name, content.clone())
    } else {
        document.push_to_default_font(content.clone())
    };

    let color = rpr.get_childs_attr::<ColorValue>("color", "w:val");

    let width = rpr
        .has_child_ans("b")
        .then_some(TextWidth::Bold)
        .unwrap_or_default();

    let italic = rpr.has_child_ans("i");

    let underline = rpr.has_child_ans("b");

    Some(super::TextProperties {
        font_handle,
        size,
        size_cs,
        width,
        color,
        italic,
        underline,
    })
}

trait HasChildAnyNs {
    fn has_child_ans(&self, name: &str) -> bool;
}

impl HasChildAnyNs for Element {
    fn has_child_ans(&self, name: &str) -> bool {
        self.has_child(name, NSChoice::Any)
    }
}

trait GetChildsAttr {
    fn get_childs_attr<T: FromStr>(&self, name: &str, attr: &str) -> Option<T>;
}

trait GetTexts {
    fn get_texts(&self) -> Option<String>;
}

impl GetChildsAttr for Element {
    #[inline]
    fn get_childs_attr<T: FromStr>(&self, name: &str, attr: &str) -> Option<T> {
        self.get_child_ans(name)?.get_attr::<T>(attr)
    }
}

impl GetTexts for Element {
    #[inline]
    fn get_texts(&self) -> Option<String> {
        Some(
            self.get_child_ans("t")?
                .texts()
                .next()
                .unwrap_or_default()
                .to_string(),
        )
    }
}

trait GetChildAnyNS {
    fn get_child_ans(&self, name: &str) -> Option<&Element>;
}

impl GetChildAnyNS for Element {
    #[inline]
    fn get_child_ans(&self, name: &str) -> Option<&Element> {
        self.get_child(name, NSChoice::Any)
    }
}

trait GetAttr {
    fn get_attr<T: FromStr>(&self, attr: &str) -> Option<T>;
}
impl GetAttr for Element {
    #[inline]
    fn get_attr<T: FromStr>(&self, attr: &str) -> Option<T> {
        self.attr(attr)?.parse::<T>().ok()
    }
}
