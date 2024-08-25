fn main() -> anyhow::Result<()> {
    let data = include_str!("./document_formated.xml");
    let document: word_xml::WordXMLDocument = data.parse()?;


    let mut buf = Vec::new();
    let _ = document.write_to(&mut buf);

    println!("{}", String::from_utf8(buf).unwrap());

    Ok(())
}
