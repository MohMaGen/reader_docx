use super::{
    DocumentGrid, DocxDocument, DocxNode, FormProt, NumType, PageMargin, PageSize, PageType,
    TextDirection,
};

#[derive(Clone)]
pub struct SectrOfProperties {
    pub page_type: PageType,
    pub page_size: PageSize,
    pub page_margin: PageMargin,
    pub page_num_type: NumType,
    pub form_prot: FormProt,
    pub text_direction: TextDirection,
    pub document_grid: DocumentGrid,
}

impl SectrOfProperties {
    pub fn get_size(&self) -> (f32, f32) {
        (self.page_size.width, self.page_size.height)
    }

    pub fn get_margins(&self) -> (f32, f32, f32, f32) {
        (
            self.page_margin.header + self.page_margin.top,
            self.page_margin.right,
            self.page_margin.bottom + self.page_margin.footer,
            self.page_margin.left,
        )
    }
}

impl DocxDocument {
    pub fn get_properties(&self) -> Option<SectrOfProperties> {
        let Some(nodes) = &self.content.nodes else {
            return None;
        };
        let properties = nodes.iter().rev().find(|v| v.is_sector_of_properties())?;

        match properties {
            DocxNode::SectrOfProperties {
                page_type,
                page_margin,
                page_size,
                page_num_type,
                form_prot,
                text_direction,
                document_grid,
            } => Some(SectrOfProperties {
                page_type: page_type.clone(),
                page_size: page_size.clone(),
                page_margin: page_margin.clone(),
                page_num_type: page_num_type.clone(),
                form_prot: form_prot.clone(),
                text_direction: text_direction.clone(),
                document_grid: document_grid.clone(),
            }),
            _ => None,
        }
    }
}

impl DocxNode {
    pub fn is_sector_of_properties(&self) -> bool {
        matches!(self, Self::SectrOfProperties { .. })
    }
}
