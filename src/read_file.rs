use anyhow::Context;
use minidom::{Children, Element};
use std::{
    io::Read,
    fmt::Write,
    path::Path,
};
use zip::ZipArchive;

pub fn read(archive_path: &Path, file_path: &Path) -> anyhow::Result<Element> {
    let archive = std::fs::File::open(archive_path)
        .context(format!("Can't read file <<{:?}>>.", archive_path.display()))?;

    let mut archive = ZipArchive::new(archive)?;

    let mut file = archive
        .by_name(file_path.to_str().context("invalid file path.")?)
        .context(format!(
            "There isnot file with path {:?} in archive {:?}",
            file_path.display(),
            archive_path.display()
        ))?;

    let mut content = String::new();
    file.read_to_string(&mut content)
        .context("Can't unzip file")?;

    content.parse().context("Can't parse xml file")
}

pub fn print_elem(root: &Element, level: usize) {
    print!("{}<{}>", "---".repeat(level), root.name());

    for attr in root.attrs() {
        print!(" [{} = {}] ", attr.0, attr.1);
    }
    for text in root.texts() {
        print!(" <<{}>> ", text);
    }
    println!();
}

pub fn print_tree(root: &Element) {
    print_elem(root, 0);
    for (elem, level) in ElementsIterator::new(root) {
        print_elem(elem, level);
    }
}

pub fn write_tree<'a, 'b, T: Write>(root: &'a Element, writer: &'b mut T) -> anyhow::Result<()>
where
    'b: 'a,
{
    write_elem(root, 0, writer)?;

    for (elem, level) in ElementsIterator::new(root) {
        write_elem(elem, level, writer)?;
    }

    Ok(())
}

pub fn write_elem<'a, 'b, T: Write>(
    root: &'a Element,
    level: usize,
    writer: &'b mut T,
) -> anyhow::Result<()>
where
    'b: 'a,
{
    write!(writer, "{}<{}>", "---".repeat(level), root.name())?;

    for attr in root.attrs() {
        write!(writer, " [{} = {}] ", attr.0, attr.1)?;
    }
    for text in root.texts() {
        write!(writer, " <<{}>> ", text)?;
    }
    writeln!(writer, )?;
    Ok(())
}

pub struct ElementsIterator<'a> {
    iterators: Vec<Children<'a>>,
    level: usize,
}

impl<'a> ElementsIterator<'a> {
    pub fn new(root: &'a Element) -> Self {
        Self {
            level: 0,
            iterators: vec![root.children()],
        }
    }
}

impl<'a> std::iter::Iterator for ElementsIterator<'a> {
    type Item = (&'a Element, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let ElementsIterator {
            ref mut iterators,
            ref mut level,
        } = self;

        while let Some(iterator) = iterators.last_mut() {
            if let Some(elem) = iterator.next() {
                iterators.push(elem.children());
                let tmp = *level;
                *level += 1;
                return Some((elem, tmp));
            } else {
                *level = level.checked_sub(1).unwrap_or_default();
                iterators.pop();
            }
        }
        None
    }
}
