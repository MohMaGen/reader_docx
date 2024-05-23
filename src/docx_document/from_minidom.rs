use std::str::FromStr;

use anyhow::Context;
use minidom::{Element, NSChoice};

use crate::docx_document::ParagraphProperties;

use super::{DocxDocument, FontTable, TextWidth};

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

        use super::DocxNode::*;
        for root_element in body.children() {
            let root_element = Box::new(match root_element.name() {
                "p" => {
                    let paragraph = Paragrapth {
                        properties: ParagraphProperties::default(),
                        attrs: root_element
                            .attrs()
                            .map(|(name, value)| (name.to_string(), value.to_string()))
                            .collect(),
                        texts: get_texts_of_element(root_element, &mut document),
                    };
                    paragraph
                }
                _ => Todo(root_element.clone()),
            });
            document.content.push(root_element);
        }

        Ok(document)
    }
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

fn get_texts_of_element(
    root_element: &minidom::Element,
    document: &mut DocxDocument,
) -> Vec<super::TextNode> {
    root_element
        .children()
        .filter(|tag| tag.name() == "r")
        .filter_map(|r_tag| {
            let rpr = r_tag.get_child_ans("rPr")?;
            let content = r_tag.get_texts()?;

            let size = rpr.get_childs_attr::<i32>("sz", "w:val")?.into();
            let size_cs = rpr.get_childs_attr::<i32>("szCs", "w:val")?.into();
            let font_handle =
                if let Some(font_name) = rpr.get_childs_attr::<String>("rFonts", "w:ascii") {
                    document.init_or_push_to_font(font_name, content.clone())
                } else {
                    document.push_to_default_font(content.clone())
                };
            let width = r_tag
                .has_child_ans("b")
                .then_some(TextWidth::Bold)
                .unwrap_or_default();
            let properties = super::TextProperties {
                font_handle,
                size,
                size_cs,
                width,
            };

            Some(super::TextNode {
                properties,
                content,
            })
        })
        .collect()
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
