use raylib::{
    color::Color,
    drawing::{RaylibDraw, RaylibDrawHandle},
    math::Vector2,
};

use crate::{
    block::{Alignment, AlignmentHorizontal, AlignmentVertical, Block, Scalable},
    env::Environment,
    text::{Text, TextAlign},
};

pub struct PageConfig {
    pub size: (f32, f32),
    pub roughtness: f32,
    pub margin: (f32, f32, f32, f32),
}

pub fn draw_page(d: &mut RaylibDrawHandle, cfg: PageConfig, block: Block) -> Block {
    use AlignmentHorizontal::Center;
    use AlignmentVertical::Top;

    let PageConfig {
        size: (width, height),
        roughtness,
        margin,
    } = cfg;
    let (x, y) = block.get_child_pos(
        Alignment {
            vertical: Top,
            horizontal: Center,
        },
        Block::new((width, height)),
    );

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
        padding: margin,
    }
}

pub fn draw_text(d: &mut RaylibDrawHandle, text: &Text, mut block: Block, env: &Environment) {
    use AlignmentHorizontal::*;
    use AlignmentVertical::Top;

    let (_, first_line_height) = text.measure_first_line(env);
    let (spacing_x, spacing_y) = text.spacing.scale(env);

    for line in text.content.iter() {
        let line_block = text.get_line_block(line).scale(env);

        let (x, y) = match text.align {
            TextAlign::Left => block.get_child_pos(Alignment::new(Top, Left), line_block),
            TextAlign::Center => block.get_child_pos(Alignment::new(Top, Center), line_block),
            TextAlign::Right => block.get_child_pos(Alignment::new(Top, Right), line_block),
            TextAlign::Justify => block.get_child_pos(Alignment::new(Top, Left), line_block),
        };

        d.draw_text_ex(
            text.font,
            line,
            Vector2 { x, y },
            text.font_size.scale(env),
            spacing_x,
            text.color,
        );

        block.add_top_padding(first_line_height + spacing_y);
    }
}
