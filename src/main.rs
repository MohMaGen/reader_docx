#![feature(once_cell_try)]

use std::path::Path;

use argp::{parse_args_or_exit, DEFAULT};
use args::ReaderDoc;
use block::{Block, Scalable, Scrolable};
use draw::{draw_page, draw_text, PageConfig};
use env::Environment;
use font::find_font;
use raylib::{color::Color, drawing::RaylibDraw};
use read_file::{print_tree, write_tree};
use text::TextBuilder;

pub mod block;
pub mod draw;
pub mod env;
pub mod font;
pub mod read_file;
pub mod text;
pub mod args;
pub mod docx_document;

fn main() -> anyhow::Result<()> {
    let args = parse_args_or_exit::<ReaderDoc>(DEFAULT);

    let root = read_file::read(args.input.unwrap().as_path(), Path::new("word/document.xml"))?;

    //let fonts = read_file::read(archive, Path::new("word/fontTable.xml"))?;
    print_tree(&root);

    let (mut rl, thread) = raylib::init()
        .resizable()
        .size(640, 480)
        .title("Hello, World")
        .build();


    let mut env = Environment::default();

    let mut root_content = String::new();
    write_tree(&root, &mut root_content)?;

    let arial_font = &find_font(&mut rl, &thread, "Liberation Sans", Some("Regular"))?;

    let _ = TextBuilder::fast(text::TextConfig {
        content: "Aboba and boba going to run above this roofs. a Every time i ...",
        font_size: 25.,
        spacing: (1., 1.),
        color: Color::BLACK,
        align: text::TextAlign::Center,
        font: arial_font,
    })
    .build(500.);

    while !rl.window_should_close() {
        env.update(&mut rl);

        let page_wrapper = Block::window_block(&rl).scroll(&env);

        let page_cfg = PageConfig {
            size: (1224.0, 1584.).scale(&env),
            margin: (113.4, 113.2, 113.4, 113.4).scale(&env),
            roughtness: 0.01,
        };
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::LIGHTGRAY);
        let mut block = draw_page(&mut d, page_cfg, page_wrapper);

        draw_text(
            &mut d,
            TextBuilder::fast(text::TextConfig {
                content: ";ςερτυθιοπ[]ασδφγηξκλζχψωβνμ,.йцукенгшщзхъфывапролджэячсмитьбю.ЁёЙЦУКЕНГШЩЗХЪФЫВАПРОЛДЖЭ/ЯЧСМИТЬБЮ,, a Every time i ...",
                font_size: 25.,
                spacing: (1., 1.),
                color: Color::BLACK,
                align: text::TextAlign::Center,
                font: arial_font,
            })
            .build(500.),
            block.clone(),
            &env,
        );

        block.add_top_padding(100.0.scale(&env));

        draw_text(
            &mut d,
            TextBuilder::fast(text::TextConfig {
                content: "\tAboba and boba going to run above this roofs. a Every time i ...Aboba and boba going to run above this roofs. a Every time i ..Aboba and boba going to run above this roofs. a Every time i ....",
                font_size: 12.,
                spacing: (0.5, 1.),
                color: Color::BLACK,
                align: text::TextAlign::Left,
                font: arial_font,
            },            )
            .build(600.0),
            block,
            &env,
        );
    }

    Ok(())
}
