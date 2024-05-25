use raylib::drawing::RaylibDrawHandle;

use crate::{block::Block, docx_document::DocxNode, env::Environment};

pub fn draw_paragraph(
    d: &mut RaylibDrawHandle,
    paragraph: DocxNode,
    page_block: &mut Block,
    env: &Environment,
) {
    let DocxNode::Paragrapth {
        properties,
        attrs,
        texts,
    } = paragraph
    else {
        return;
    };
    



}
