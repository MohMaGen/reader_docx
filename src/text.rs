use raylib::{color::Color, math::Vector2, prelude::Font, text::measure_text_ex};

use crate::{block::Scalable, env::Environment};

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

    pub fn build(&'a mut self, width: f32) -> &'a Text<'a> {
        let mut word = &self.content[0..1];
        let mut first_idx = 0;
        let mut line_first = 0;

        let is_not_word = |c: char| c.is_whitespace();
        let the_end = |word: &str| word.chars().last().map(is_not_word).unwrap_or(false);

        // word ["aboba"], " "

        for (idx, c) in self.content[1..].char_indices() {
            if is_not_word(c) == the_end(word) {
                word = &self.content[first_idx..=idx];
            } else {
                let line = &self.content[line_first..first_idx];
                let Vector2 { x, .. } = measure_text_ex(
                    self.text.font,
                    line,
                    self.text.font_size,
                    self.text.spacing.0,
                );

                if x > width {
                    self.text.content.push(line);
                    line_first = first_idx;
                }
                first_idx = idx;
            }
        }
        self.text.content.push(&self.content[line_first..]);
        &self.text
    }
}

impl Text<'_> {
    pub fn measure_first_line(&self, env: &Environment) -> (f32, f32) {
        let Vector2 { x, y } =
            measure_text_ex(self.font, self.content[0], self.font_size.scale(env), self.spacing.0.scale(env));
        (x, y)
    }

    pub fn measure_line_of_this_font(&self, line: &str, env: &Environment) -> (f32, f32) {
        let Vector2 { x, y } =
            measure_text_ex(self.font, line, self.font_size.scale(env), self.spacing.0.scale(env));
        (x, y)
    }
}

