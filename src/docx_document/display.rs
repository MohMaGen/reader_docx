use super::{ContentTree, DocxDocument, DocxNode, FontTable, ParagraphProperties, TextNode, TextProperties};

const PRE: &'static str = "  ";

impl std::fmt::Display for DocxDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, ":( document ):")?;

        writeln!(f, "*")?;
        display_property("content", &self.content, 1, f)?;

        writeln!(f, "*")?;
        display_property("fonts", &self.fonts, 1, f)?;

        writeln!(f, ":( end ):")
    }
}

impl std::fmt::Display for FontTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, ":( fonts ):")?;

        writeln!(f, "*")?;
        writeln!(f, "{}|>- fonts -<|", PRE)?;
        for font in &self.fonts {
            writeln!(f, "{}+", PRE)?;

            writeln!(f, "{}:( name ):", PRE.repeat(2))?;
            writeln!(f, "{}", font.name.with_indent(3))?;
            writeln!(f, "{}:( end ):", PRE.repeat(2))?;

            writeln!(f, "{}:( variants ):", PRE.repeat(2))?;
            writeln!(f, "{}", format!("{:?}", font.variants).with_indent(3))?;
            writeln!(f, "{}:( end ):", PRE.repeat(2))?;
        }
        writeln!(f, "{}|>- end -<|", PRE)?;

        writeln!(f, "*")?;

        writeln!(f, ":( end ):")
    }
}

impl std::fmt::Display for ContentTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Some(nodes) = &self.nodes else {
            return writeln!(f, ":[ content tree ]:");
        };
        writeln!(f, ":( content tree ):")?;

        writeln!(f, "{}|>- nodes -<|", PRE)?;

        for node in nodes {
            writeln!(f, "{}+", PRE)?;
            writeln!(f, "{}", node.to_string().with_indent(2))?;
        }

        writeln!(f, "{}|>- end -<|", PRE)?;

        writeln!(f, ":( end ):")
    }
}

impl std::fmt::Display for DocxNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocxNode::Paragrapth {
                properties,
                attrs,
                texts,
            } => {
                writeln!(f, ":( paragrapth ):")?;
                writeln!(f, "*")?;
                display_property("properties", properties, 2, f)?;

                writeln!(f, "*")?;
                display_property("attrebutes", format!("{:?}", attrs), 2, f)?;

                writeln!(f, "*")?;
                writeln!(f, "{}|>- texts -<|", PRE)?;
                for text in texts {
                    writeln!(f, "{}+", PRE)?;
                    writeln!(f, "{}", text.to_string().with_indent(2))?;
                }
                writeln!(f, "{}|>- end -<|", PRE)?;

                writeln!(f, ":( end ):")
            }
            DocxNode::Todo(element) => {
                writeln!(f, ":( todo ):")?;
                writeln!(f, "{}", format!("{element:?}").with_indent(1))?;
                writeln!(f, ":( end ):")
            }
        }
    }
}

impl std::fmt::Display for ParagraphProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, ":( paragraph properties ):")?;

        writeln!(f, "*")?;
        display_property("justify", format!("{:?}", self.justify), 2, f)?;

        writeln!(f, "*")?;
        display_property("text properties", &self.text_properties, 2, f)?;

        writeln!(f, ":( end ):")
    }
}

impl std::fmt::Display for TextProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, ":( text properties ):")?;

        writeln!(f, "*")?;
        display_property("size", format!("{:?}", self.size), 3, f)?;

        writeln!(f, "*")?;
        display_property("size cz", format!("{:?}", self.size_cs), 3, f)?;

        writeln!(f, "*")?;
        display_property("font handle", format!("{:?}", self.font_handle), 3, f)?;

        writeln!(f, "*")?;
        display_property("width", format!("{:?}", self.width), 3, f)?;

        writeln!(f, ":( end ):")
    }
}

impl std::fmt::Display for TextNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, ":( text node ):")?;
        writeln!(f, "*")?;
        display_property("properties", &self.properties, 1, f)?;

        writeln!(f, "*")?;
        display_property("content", &self.content, 1, f)?;

        writeln!(f, ":( end ):")
    }
}

fn display_property<P: ToString>(
    name: &str,
    property: P,
    indent: usize,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}>- {} -<", PRE, name.to_lowercase())?;
    writeln!(f, "{}", property.to_string().with_indent(indent))?;
    writeln!(f, "{}>- end -<", PRE)
}

trait WithIndent {
    fn with_indent(&self, level: usize) -> Self;
}

impl WithIndent for String {
    #[inline]
    fn with_indent(&self, level: usize) -> Self {
        let indent = PRE.repeat(level);
        format!(
            "{indent}{}",
            self.replace("\n", format!("\n{indent}").as_str()).trim()
        )
    }
}
