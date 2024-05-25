#![feature(once_cell_try)]

use std::path::Path;

use anyhow::Context;
use argp::{parse_args_or_exit, DEFAULT};
use args::ReaderDoc;
use block::{Block, Scalable, Scrolable};
use docx_document::{getters::SectrOfProperties, DocxDocument};
use draw::{draw_page, PageConfig};
use env::Environment;
use raylib::{color::Color, drawing::RaylibDraw};
use read_file::print_tree;

pub mod args;
pub mod block;
pub mod docx_document;
pub mod draw;
pub mod env;
pub mod font;
pub mod read_file;
pub mod text;
pub mod draw_paragraph;

fn main() -> anyhow::Result<()> {
    let args = parse_args_or_exit::<ReaderDoc>(DEFAULT);

    let root = read_file::read(
        args.input.clone().unwrap().as_path(),
        Path::new("word/document.xml"),
    )?;

    let fonts = read_file::read(
        args.input.unwrap().as_path(),
        Path::new("word/fontTable.xml"),
    )?;

    print_tree(&fonts);
    print_tree(&root);

    let document = DocxDocument::try_from((&root, &fonts))?;
    println!("{}", document);
    let properties = document.get_properties().context("document need page properties!!!!!")?;

    let (mut rl, thread) = raylib::init()
        .resizable()
        .size(1000, 100)
        .title("Hello, World")
        .build();

    let mut env = Environment::default();

    while !rl.window_should_close() {
        env.update(&mut rl);

        let page_wrapper = Block::window_block(&rl).scroll(&env);
        let page_cfg = PageConfig::from_properties(&properties, &env);

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::LIGHTGRAY);
        let _block = draw_page(&mut d, page_cfg, page_wrapper);

    }

    Ok(())
}

impl PageConfig {
    fn from_properties(properties: &SectrOfProperties, env: &Environment) -> Self {
        PageConfig {
            size: properties.get_size().scale(&env),
            margin: properties.get_margins().scale(&env),
            roughtness: 0.01,
        }
    }
}
