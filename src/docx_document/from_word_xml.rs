use anyhow::{anyhow, Context};
use word_xml::Node;

use crate::docx_document::DocxNode;

use super::{
    Color, DocumentGrid, DocxDocument, FontTable, FormProt, GridType, Justification, LineRule,
    NumType, PageMargin, PageSize, ParagraphProperties, SpacingProperties, TextDirection, TextSize,
    TextWeight,
};

impl<'a> TryFrom<(&'a word_xml::WordXMLDocument, &'a word_xml::WordXMLDocument)> for DocxDocument {
    type Error = anyhow::Error;

    fn try_from(
        (document_xml, fonts): (&'a word_xml::WordXMLDocument, &'a word_xml::WordXMLDocument),
    ) -> Result<Self, Self::Error> {
        if document_xml.root.name != "w:document" {
            return Err(anyhow!(
                "Invalid document root element name: {:?}",
                document_xml.root.name
            ));
        }

        let mut document = DocxDocument {
            fonts: FontTable::try_from(fonts)?,
            ..Default::default()
        };

        let body = document_xml
            .root
            .get_child("w:body")
            .context("No body element")?;

        for root_element in body.inners.iter().filter_map(Node::get_element) {
            let curr = Box::new(match root_element.name.as_str() {
                "w:p" => parse_paragraph(root_element, &mut document),
                "w:sectPr" => {
                    parse_sectr_properties(root_element).context(format!("{:#?}", root_element))?
                }
                _ => DocxNode::TodoWordXml(root_element.clone()),
            });
            document.content.push(curr);
        }

        Ok(document)
    }
}

impl<'a> TryFrom<&'a word_xml::WordXMLDocument> for FontTable {
    type Error = anyhow::Error;

    fn try_from(value: &'a word_xml::WordXMLDocument) -> Result<Self, Self::Error> {
        let mut table = FontTable::default();

        for font in value.root.get_children("w:font") {
            let name = font
                .get_attr_parsed::<String>("w:name")
                .context("Font must have name")?;
            table.init_or_push_to_font(name, String::new());
        }

        Ok(table)
    }
}

#[inline]
fn parse_paragraph(
    root_element: &word_xml::Element,
    document: &mut DocxDocument,
) -> super::DocxNode {
    use super::DocxNode::Paragrapth;
    Paragrapth {
        properties: parse_paragraph_properties(root_element, document),
        attrs: root_element
            .attrs
            .iter()
            .map(|attr| (attr.name.clone(), attr.value.clone()))
            .collect(),
        texts: get_texts_of_element(root_element, document),
    }
}

#[inline]
fn parse_paragraph_properties(
    root_element: &word_xml::Element,
    document: &mut DocxDocument,
) -> ParagraphProperties {
    let Some(ppr) = root_element.get_child("w:pPr") else {
        return ParagraphProperties::default();
    };
    ParagraphProperties {
        justify: ppr.get_childs_attr_parsed::<Justification>("w:jc", "w:val"),
        text_properties: parse_text_properties(ppr, document, &Default::default()),
        spacing: parce_spacing(ppr),
    }
}

fn parce_spacing(ppr: &word_xml::Element) -> SpacingProperties {
    SpacingProperties {
        line: parse_float_as_some(ppr, "w:spacing", "w:line"),
        line_rule: ppr.get_childs_attr_parsed::<LineRule>("spacing", "w:lineRule"),
        after: parse_float_as_some(ppr, "w:spacing", "w:after"),
        before: parse_float_as_some(ppr, "w:spacing", "w:before"),
    }
}

fn parse_float_as_some(ppr: &word_xml::Element, name: &str, attr: &str) -> Option<f32> {
    ppr.get_childs_attr_parsed::<u64>(name, attr)
        .map(|v| v as f32 * 0.1)
}

fn parse_sectr_properties(root_element: &word_xml::Element) -> anyhow::Result<super::DocxNode> {
    use super::DocxNode::SectrOfProperties;
    Ok(SectrOfProperties {
        page_type: page_type(root_element),

        page_size: parse_page_size(root_element).context("Must have page size.")?,
        page_margin: parse_page_margin(root_element).context("Must have page margins.")?,
        page_num_type: parse_page_num_type(root_element),
        form_prot: parse_form_prot(root_element),
        text_direction: parse_text_direction(root_element),
        document_grid: parse_document_grid(root_element),
    })
}

fn parse_document_grid(root_element: &word_xml::Element) -> Option<DocumentGrid> {
    Some(DocumentGrid {
        char_space: root_element.get_childs_attr_parsed::<u64>("w:docGrid", "w:charSpace")?,
        line_pitch: root_element.get_childs_attr_parsed::<u64>("w:docGrid", "w:linePitch")?,
        grid_type: root_element.get_childs_attr_parsed::<GridType>("w:docGrid", "w:type")?,
    })
}

fn parse_text_direction(root_element: &word_xml::Element) -> TextDirection {
    root_element
        .get_childs_attr_parsed::<TextDirection>("w:textDirection", "w:val")
        .context("can't get text direction")
        .ok()
        .unwrap_or_default()
}

fn parse_form_prot(root_element: &word_xml::Element) -> Option<FormProt> {
    root_element
        .get_childs_attr_parsed::<FormProt>("w:formProt", "w:val")
        .context("can't get page form prot")
        .ok()
}

fn parse_page_num_type(root_element: &word_xml::Element) -> Option<NumType> {
    root_element
        .get_childs_attr_parsed::<NumType>("w:pgNumType", "w:fmt")
        .context("can't get page num type")
        .ok()
}

fn parse_page_margin(root_element: &word_xml::Element) -> anyhow::Result<PageMargin> {
    Ok(PageMargin {
        footer: get_float(root_element, "w:pgMar", "w:footer").context("footer")?,
        gutter: get_float(root_element, "w:pgMar", "w:gutter").context("gutter")?,
        header: get_float(root_element, "w:pgMar", "w:header").context("header")?,
        bottom: get_float(root_element, "w:pgMar", "w:bottom").context("bottom")?,
        left: get_float(root_element, "w:pgMar", "w:left").context("left")?,
        right: get_float(root_element, "w:pgMar", "w:right").context("right")?,
        top: get_float(root_element, "w:pgMar", "w:top").context("top")?,
    })
}

fn page_type(root_element: &word_xml::Element) -> Option<super::PageType> {
    root_element
        .get_childs_attr_parsed("w:type", "w:val")
        .context(format!("can't parse page type: `{:?}`", root_element))
        .ok()
}

fn parse_page_size(root_element: &word_xml::Element) -> Option<PageSize> {
    Some(PageSize {
        width: get_float(root_element, "w:pgSz", "w:w").ok()?,
        height: get_float(root_element, "w:pgSz", "w:h").ok()?,
    })
}

#[inline]
fn get_float(
    root_element: &word_xml::Element,
    name: &str,
    attr: &str,
) -> Result<f32, anyhow::Error> {
    Ok(root_element
        .get_childs_attr_parsed::<u64>(name, attr)
        .context("can't parse float")? as f32
        / 10.)
}

#[inline]
fn get_texts_of_element(
    root_element: &word_xml::Element,
    document: &mut DocxDocument,
) -> Vec<super::TextNode> {
    root_element
        .inners
        .iter()
        .filter_map(word_xml::Node::get_element)
        .filter(|tag| tag.name.as_str() == "w:r")
        .filter_map(|r_tag| {
            let content = r_tag.get_child("w:t")?.get_texts();

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
    parent_tag: &word_xml::Element,
    document: &mut DocxDocument,
    content: &String,
) -> Option<super::TextProperties> {
    let rpr = parent_tag.get_child("w:rPr")?;

    let size = rpr
        .get_childs_attr_parsed::<i32>("w:sz", "w:val")
        .map(TextSize::from);

    let size_cs = rpr
        .get_childs_attr_parsed::<i32>("w:szCs", "w:val")
        .map(TextSize::from);

    let font_name = rpr.get_childs_attr_parsed::<String>("w:rFonts", "w:ascii");
    let font_handle = if let Some(font_name) = font_name.clone() {
        document.init_or_push_to_font(font_name, content.clone())
    } else {
        document.push_to_default_font(content.clone())
    };

    let color = rpr.get_childs_attr_parsed::<Color>("w:color", "w:val");

    let width = rpr
        .has_child("w:b")
        .then_some(TextWeight::Bold)
        .unwrap_or_default();

    let italic = rpr.has_child("w:i");

    let underline = rpr.has_child("w:b");

    Some(super::TextProperties {
        font_handle,
        font_name,
        size,
        size_cs,
        weight: width,
        color,
        italic,
        underline,
    })
}
