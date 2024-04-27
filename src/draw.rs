use raylib::{
    color::Color,
    drawing::{RaylibDraw, RaylibDrawHandle},
    math::Vector2,
    text::measure_text,
};

use crate::{
    block::{Alignment, Block, Scalable}, env::Environment, text::{Text, TextAlign}
};

pub struct PageConfig {
    pub size: (f32, f32),
    pub roughtness: f32,
    pub margin: (f32, f32, f32, f32),
}

pub fn draw_page(d: &mut RaylibDrawHandle, cfg: PageConfig, block: Block) -> Block {
    let PageConfig {
        size: (width, height),
        roughtness,
        margin,
    } = cfg;
    let (x, y) = block.get_child_pos(crate::block::Alignment::Center, cfg.size, margin);

    d.draw_rectangle_rounded(
        raylib::ffi::Rectangle {
            x,
            y,
            width,
            height,
        },
        roughtness,
        0,
        Color::WHITE,
    );

    Block {
        pos: (x, y),
        size: cfg.size,
        paddings: (10., 10., 10., 10.),
    }
}

pub struct RawText<'a> {
    pub content: &'a str,
    pub font_size: f32,
    pub color: Color,
}

pub fn draw_raw_text(d: &mut RaylibDrawHandle, txt: RawText, block: Block, alg: Alignment) {
    let RawText {
        content,
        font_size,
        color,
    } = &txt;

    let width = measure_text(content, *font_size as i32);

    let (x, y) = block.get_child_pos(alg, (width as f32, 0.), (0., 0., 0., 0.));

    d.draw_text(content, x as i32, y as i32, *font_size as i32, color);
}

pub fn draw_text(d: &mut RaylibDrawHandle, text: &Text, block: Block, env: &Environment) {
    let (_, first_line_height) = text.measure_first_line(env);
    let (spacing_x, spacing_y) = text.spacing.scale(env);

    for (idx, line) in text.content.iter().enumerate() {
        let size = text.measure_line_of_this_font(line, env);

        let (block_x, block_y) = (
            block.pos.0 + block.paddings.3,
            block.pos.1 + block.paddings.0,
        );

        let line_block = Block {
            pos: (
                block_x,
                block_y + idx as f32 * (first_line_height + spacing_y),
            ),
            size: (block.size.0, first_line_height),
            paddings: (0., 0., 0., 0.),
        };
        let (x, y) = match text.align {
            TextAlign::Left => line_block.get_child_pos(Alignment::Left, size, (0., 0., 0., 0.)),
            TextAlign::Center => {
                line_block.get_child_pos(Alignment::Center, size, (0., 0., 0., 0.))
            }
            TextAlign::Right => line_block.get_child_pos(Alignment::Right, size, (0., 0., 0., 0.)),
            TextAlign::Justify => line_block.get_child_pos(Alignment::Left, size, (0., 0., 0., 0.)),
        };

        d.draw_text_ex(
            text.font,
            line,
            Vector2 { x, y },
            text.font_size.scale(env),
            spacing_x,
            text.color,
        );
    }
}
