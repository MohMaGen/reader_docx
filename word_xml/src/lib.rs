mod from_str;
mod getters;

#[derive(Debug)]
pub struct WordXMLDocument {
    pub header: String,
    pub document_element: Element,
}

#[derive(Default, Debug)]
pub struct Element {
    pub name: String,
    pub attrs: Vec<Attr>,
    pub inners: Vec<Node>,
}

#[derive(Default, Debug)]
pub struct Attr {
    pub name: String,
    pub value: String,
}

#[derive(Default, Debug)]
pub struct Text(pub String);

#[derive(Debug)]
pub enum Node {
    Element(Element),
    Text(Text),
}

impl Node {
    fn is_element(&self) -> bool {
        matches!(self, Self::Element(_))
    }
    fn is_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }
    fn get_element(&self) -> Option<&Element> {
        match self {
            Node::Element(elem) => Some(elem),
            _ => None,
        }
    }
    fn get_text(&self) -> Option<&Text> {
        match self {
            Node::Text(text) => Some(text),
            _ => None,
        }
    }
}
