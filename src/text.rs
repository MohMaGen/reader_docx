use raylib::{color::Color, math::Vector2, prelude::Font, text::measure_text_ex};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    block::{Block, Scalable},
    env::Environment,
};

#[derive(Debug)]
pub struct TextConfig<'a> {
    pub content: &'a str,
    pub font_size: f32,
    pub spacing: (f32, f32),
    pub color: Color,
    pub align: TextAlign,
    pub font: &'a Font,
}

pub struct Text<'a> {
    pub content: Vec<&'a str>,
    pub font_size: f32,
    pub spacing: (f32, f32),
    pub color: Color,
    pub size: (f32, f32),
    pub align: TextAlign,
    pub font: &'a Font,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

pub struct TextBuilder<'a> {
    text: Text<'a>,
    content: &'a str,
}

impl<'a> TextBuilder<'a> {
    pub fn fast(text: TextConfig<'a>) -> Self {
        let TextConfig {
            content,
            font_size,
            spacing,
            color,
            align,
            font,
        } = text;

        Self {
            content,
            text: Text {
                content: Vec::new(),
                font_size,
                spacing,
                color,
                font,
                align,
                size: (0., 0.),
            },
        }
    }

    pub fn build(&'a mut self, max_width: f32) -> &'a Text<'a> {
        let text = &mut self.text;
        text.size = (0., 0.);

        let (mut line_start, mut prev_idx) = (0, 0);

        for (idx, _) in self.content.grapheme_indices(true) {
            let line = &self.content[line_start..idx];
            let Vector2 {
                x: width,
                y: height,
            } = measure_text_ex(text.font, line, text.font_size, text.spacing.0);

            if width > max_width {
                text.content.push(&self.content[line_start..prev_idx]);

                let Vector2 { x: width, .. } = measure_text_ex(
                    text.font,
                    &self.content[line_start..prev_idx],
                    text.font_size,
                    text.spacing.0,
                );
                text.size.0 = text.size.0.max(width);

                line_start = prev_idx;
            }

            text.size.1 += height + text.spacing.1;
            prev_idx = idx;
        }
        text.content.push(&self.content[line_start..]);

        &self.text
    }
}

impl Text<'_> {
    pub fn measure_first_line(&self, env: &Environment) -> (f32, f32) {
        let Vector2 { x, y } = measure_text_ex(
            self.font,
            self.content[0],
            self.font_size.scale(env),
            self.spacing.0.scale(env),
        );
        (x, y)
    }

    pub fn measure_line_of_this_font(&self, line: &str, env: &Environment) -> (f32, f32) {
        let Vector2 { x, y } = measure_text_ex(
            self.font,
            line,
            self.font_size.scale(env),
            self.spacing.0.scale(env),
        );
        (x, y)
    }

    pub fn get_line_block(&self, line: &str) -> Block {
        let Vector2 { x, y } = measure_text_ex(self.font, line, self.font_size, self.spacing.0);
        Block::new((x, y))
    }
}

impl<'a> From<&'a Text<'_>> for Block {
    fn from(value: &'a Text<'_>) -> Self {
        Self {
            size: value.size,
            pos: (0., 0.),
            padding: (0., 0., 0., 0.),
        }
    }
}
