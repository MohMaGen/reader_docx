use std::str::FromStr;

impl super::Element {
    pub fn get_children<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a super::Element> {
        self.inners
            .iter()
            .filter_map(super::Node::get_element)
            .filter(move |elem| elem.name.as_str() == name)
    }

    pub fn get_child(&self, name: &str) -> Option<&Self> {
        self.inners
            .iter()
            .filter_map(super::Node::get_element)
            .find(|elem| elem.name.as_str() == name)
    }

    pub fn get_child_mut(&mut self, name: &str) -> Option<&mut Self> {
        self.inners
            .iter_mut()
            .filter_map(super::Node::get_element_mut)
            .find(|elem| elem.name.as_str() == name)
    }

    pub fn get_attr_parsed<T: FromStr>(&self, attr_name: &str) -> Option<T> {
        self.attrs.iter().find_map(|attr| {
            (attr.name.as_str() == attr_name)
                .then_some(attr.value.parse::<T>().ok())
                .flatten()
        })
    }

    pub fn get_childs_attr_parsed<T: FromStr>(
        &self,
        child_name: &str,
        attr_name: &str,
    ) -> Option<T> {
        self.get_child(child_name)?.get_attr_parsed::<T>(attr_name)
    }

    pub fn get_texts(&self) -> String {
        self.inners
            .iter()
            .filter_map(super::Node::get_text)
            .fold(String::new(), |acc, curr| acc + curr.0.as_str())
    }

    pub fn get_childs_texts(&self, child_name: &str) -> Option<String> {
        Some(self.get_child(child_name)?.get_texts())
    }

    pub fn has_child(&self, child_name: &str) -> bool {
        self.get_child(child_name).is_some()
    }
}
