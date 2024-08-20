fn main() -> anyhow::Result<()> {
    let data = include_str!("./document_formated.xml");
    let document: word_xml::WordXMLDocument = data.parse()?;

    println!("{:#?}", document);

    Ok(())
}
