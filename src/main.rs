#![feature(once_cell_try)]

use std::path::Path;

use block::{Block, Scrolable, Scalable};
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

fn main() -> anyhow::Result<()> {
    let (mut rl, thread) = raylib::init()
        .resizable()
        .size(640, 480)
        .title("Hello, World")
        .build();

    let archive = Path::new("test2.docx");
    let root = read_file::read(archive, Path::new("word/document.xml"))?;
    let fonts = read_file::read(archive, Path::new("word/fontTable.xml"))?;
    print_tree(&fonts);

    let mut env = Environment::default();

    let mut root_content = String::new();
    write_tree(&root, &mut root_content)?;

    let arial_font = &find_font(&mut rl, &thread, "Liberation", Some("Regular"))?;

    while !rl.window_should_close() {
        env.update(&mut rl);

        let page_wrapper = Block::window_block(&rl).scroll(&env);

        let page_cfg = PageConfig {
            size: (700., 700. * 1.4).scale(&env),
            margin: (20., 20., 20., 20.).scale(&env),
            roughtness: 0.01,
        };
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::LIGHTGRAY);
        let mut block = draw_page(&mut d, page_cfg, page_wrapper);

        draw_text(
            &mut d,
            TextBuilder::fast(text::TextConfig {
                content: "Aboba and boba going to run above this roofs. a Every time i ...",
                font_size: 25.,
                spacing: (1., 1.),
                color: Color::BLACK,
                align: text::TextAlign::Center,
                font: arial_font,
            },            )
            .build(500.),
            block.clone(),
            &env,
        );

        block.add_top_padding(100.0.scale(&env));

        draw_text(
            &mut d,
            TextBuilder::fast(text::TextConfig {
                content: "Aboba and boba going to run above this roofs. a Every time i ...Aboba and boba going to run above this roofs. a Every time i ..Aboba and boba going to run above this roofs. a Every time i ....",
                font_size: 12.,
                spacing: (1., 1.),
                color: Color::BLACK,
                align: text::TextAlign::Center,
                font: arial_font,
            },            )
            .build(500.),
            block,
            &env,
        )
    }

    Ok(())
}
