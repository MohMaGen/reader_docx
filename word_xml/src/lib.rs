mod from_str;
mod getters;

#[derive(Debug)]
pub struct WordXMLDocument {
    pub header: String,
    pub root: Element,
}

#[derive(Default, Debug, Clone)]
pub struct Element {
    pub name: String,
    pub attrs: Vec<Attr>,
    pub inners: Vec<Node>,
}

#[derive(Default, Debug, Clone)]
pub struct Attr {
    pub name: String,
    pub value: String,
}

#[derive(Default, Debug, Clone)]
pub struct Text(pub String);

#[derive(Debug, Clone)]
pub enum Node {
    Element(Element),
    Text(Text),
}

impl Node {
    pub fn is_element(&self) -> bool {
        matches!(self, Self::Element(_))
    }
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }
    pub fn get_element(&self) -> Option<&Element> {
        match self {
            Node::Element(elem) => Some(elem),
            _ => None,
        }
    }
    pub fn get_text(&self) -> Option<&Text> {
        match self {
            Node::Text(text) => Some(text),
            _ => None,
        }
    }
}
