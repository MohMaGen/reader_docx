
mod impls;



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
    Text(Text)
}

