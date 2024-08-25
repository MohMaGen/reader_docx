fn main() {
    let document = word_xml::WordXMLDocument {
        header: String::from("<?HEADER?>"),
        root: word_xml::Element::new("w:aaa").with_attr("w:value", "10")
    };

    let mut buf = Vec::new();
    let _ = document.write_to(&mut buf);

    println!("{}", String::from_utf8(buf).unwrap());
}
