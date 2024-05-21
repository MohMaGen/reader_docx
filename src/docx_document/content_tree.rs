use super::{ContentTree, DocxNode};

impl ContentTree {
    pub fn push(&mut self, node: Box<DocxNode>) {
        match &mut self.nodes {
            Some(root_nodes) => root_nodes.push(node),
            None => self.nodes = Some(vec![node]),
        }
    }
}
