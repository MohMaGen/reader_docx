use minidom::NSChoice;

use crate::docx_document::ParagrapthProperties;

use super::DocxDocument;

impl TryFrom<minidom::Element> for DocxDocument {
    type Error = anyhow::Error;

    fn try_from(root: minidom::Element) -> Result<Self, Self::Error> {
        if root.name() != "document" {
            return Err(anyhow::Error::msg(
                "Can't create doc document cause root isn't document node",
            ));
        }

        let mut document = DocxDocument::default();

        use super::DocxNode::*;
        for root_element in root.children() {
            let root_element = Box::new(match root_element.name() {
                "p" => {
                    let paragraph = Paragrapth {
                        properties: ParagrapthProperties::default(),
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

fn get_texts_of_element(root_element: &minidom::Element, document: &mut DocxDocument) -> Vec<super::TextNode> {
    root_element
        .children()
        .filter(|tag| tag.name() == "r")
        .filter_map(|r_tag| {
            Some(super::TextNode {
                properties: super::TextProperties { font_handle: (), size: (), size_cs: (), width: () },
                content: r_tag
                    .get_child("t", NSChoice::Any)?
                    .texts()
                    .next()
                    .unwrap_or_default()
                    .to_string(),
            })
        })
        .collect()
}
