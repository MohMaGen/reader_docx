use std::io::Write;

impl super::WordXMLDocument {
    pub fn write_to(&self, writer: &mut impl Write) -> anyhow::Result<()> {
        writeln!(writer, "{}", self.header)?;
        self.root.write_to(writer)
    }
}

impl super::Element {
    pub fn write_to(&self, writer: &mut impl Write) -> anyhow::Result<()> {
        write!(writer, "<{}", self.name)?;
        for super::Attr { name, value } in &self.attrs {
            write!(writer, " {}={:?}", name, value)?;
        }
        write!(writer, ">")?;

        for node in &self.inners {
            match node {
                crate::Node::Element(elem) => elem.write_to(writer)?,
                crate::Node::Text(super::Text(txt)) => write!(writer, "{}", txt)?,
            }
        }

        write!(writer, "</{}>", self.name)?;

        Ok(())
    }
}
