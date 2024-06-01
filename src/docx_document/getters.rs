
use super::{
    DocumentGrid, DocxDocument, DocxNode, FormProt, NumType, PageMargin, PageSize, PageType,
    TextDirection,
};

#[derive(Clone)]
pub struct SectrOfProperties {
    pub page_type: Option<PageType>,
    pub page_size: PageSize,
    pub page_margin: PageMargin,
    pub page_num_type: Option<NumType>,
    pub form_prot: Option<FormProt>,
    pub text_direction: TextDirection,
    pub document_grid: Option<DocumentGrid>,
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

impl Default for SectrOfProperties {
    fn default() -> Self {
        Self {
            page_type: Some(PageType::NextPage),
            page_size: PageSize {
                width: 1000.,
                height: 1000.,
            },
            page_margin: PageMargin {
                footer: 100.,
                gutter: 100.,
                header: 100.,
                bottom: 100.,
                left: 100.,
                right: 100.,
                top: 100.,
            },
            page_num_type: Some(NumType::Decimal),
            form_prot: Some(FormProt { val: true }),
            text_direction: TextDirection::LeftToRightTopToBottom,
            document_grid: Some(DocumentGrid {
                char_space: 10,
                line_pitch: 20,
                grid_type: super::GridType::Default,
            }),
        }
    }
}

impl crate::traits::Scale for SectrOfProperties {
    fn scale(self, v: f32) -> Self {
        Self {
            page_size: self.page_size.scale(v),
            page_margin: self.page_margin.scale(v),
            ..self
        }
    }
}


impl crate::traits::Scale for PageSize {
    fn scale(self, v: f32) -> Self {
        Self {
            width: self.width * v,
            height: self.height * v,
        }
    }
}

impl crate::traits::Scale for PageMargin {
    fn scale(self, v: f32) -> Self {
        Self {
            footer: self.footer * v,
            gutter: self.gutter * v,
            header: self.header * v,
            bottom: self.bottom * v,
            left: self.left * v,
            right: self.right * v,
            top: self.top * v,
        }
    }
}
