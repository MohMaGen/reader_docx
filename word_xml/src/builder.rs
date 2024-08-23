impl super::Element {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            attrs: Default::default(),
            inners: Default::default(),
        }
    }

    pub fn append(&mut self, node: super::Node) {
        self.inners.push(node);
    }

    pub fn append_element(&mut self, element: super::Element) {
        self.inners.push(super::Node::Element(element));
    }

    pub fn append_text(&mut self, text: &str) {
        self.inners
            .push(super::Node::Text(super::Text(text.to_string())));
    }

    pub fn append_attr(&mut self, name: impl ToString, value: impl ToString) {
        self.attrs.push(super::Attr {
            name: name.to_string(),
            value: value.to_string(),
        });
    }

    pub fn with_element(mut self, element: super::Element) -> Self {
        self.append_element(element);
        self
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.append_text(text);
        self
    }

    pub fn with_attr<'a>(mut self, name: impl ToString, value: impl ToString) -> Self {
        self.append_attr(name, value);
        self
    }
}
