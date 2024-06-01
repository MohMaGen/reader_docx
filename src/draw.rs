use std::{ops::Mul, sync::Arc};

use anyhow::Context;
use sdl2::{
    pixels::Color,
    render::{Canvas, Texture, TextureCreator, TextureQuery},
    video::{Window, WindowContext},
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    colorscheme::ColorScheme,
    math::{sdl_rect, Paddings, Point, Rectangle, Size},
    AsAnyhow, Fonts, State, ML,
};

const D_HOR_SP: f32 = 13.;
const D_LN_SP: f32 = 1.;
const D_P_B_SP: f32 = 30.;
const D_P_A_SP: f32 = 30.;
const FONT_ML: f32 = 0.41;
const P_PAD: f32 = 40. * ML;

pub fn draw<'a>(
    canvas: &mut Canvas<Window>,
    state: &State,
    fonts: &'a Fonts<'_, '_>,
) -> anyhow::Result<()> {
    canvas.set_draw_color(Color::RGB(100, 100, 100));
    canvas.clear();

    let window_size = Size::from(canvas.window().size()) * ML;
    const CONSOLE_HEIGHT: f32 = 70. * ML;

    let page_rect: Rectangle = (
        0.,
        0.,
        window_size.width,
        window_size.height - CONSOLE_HEIGHT,
    )
        .into();
    draw_page(page_rect, canvas, state, fonts)?;

    let console_rect: Rectangle = (
        0.,
        window_size.height - CONSOLE_HEIGHT,
        window_size.width,
        window_size.height,
    )
        .into();

    draw_console(console_rect, canvas, state, fonts)?;

    let file_name_rect: Rectangle = (
        window_size.width / 2. - 200. * ML,
        window_size.height - CONSOLE_HEIGHT - 20.,
        window_size.width / 2. + 200. * ML,
        window_size.height - CONSOLE_HEIGHT + 20.,
    )
        .into();
    draw_file_name(file_name_rect, canvas, state, fonts)?;

    canvas.present();

    Ok(())
}
fn draw_file_name(
    rect: impl Into<Rectangle>,
    canvas: &mut Canvas<Window>,
    state: &State,
    fonts: &Fonts<'_, '_>,
) -> anyhow::Result<()> {
    let Some(document) = state.document.clone() else {
        return Ok(());
    };

    let name = Arc::clone(&document).path.clone();

    canvas.set_draw_color(state.console_bg());
    canvas.fill_rect(sdl_rect(rect)).as_anyhow()?;

    Ok(())
}

fn draw_page(
    rect: impl Into<Rectangle>,
    canvas: &mut Canvas<Window>,
    state: &State,
    fonts: &Fonts<'_, '_>,
) -> anyhow::Result<()> {
    let (page_color, page_bg_color, page_border_color) = (
        state.page_color(),
        state.page_bg_color(),
        state.page_border_color(),
    );

    let (_point, viewport_size) = rect.into().get_point_and_size();

    let page_bounds = Rectangle::new((0., 0.), viewport_size).add_paddings(15. * ML);

    canvas.draw_double_borders(
        page_bounds,
        BorderType::Outside,
        page_border_color,
        page_bg_color,
    )?;

    let Some(document) = state.document.clone() else {
        return Ok(());
    };

    let page_properties = document
        .docx_document
        .get_properties()
        .context("Document doesn't containt page properties")?;

    canvas.set_clip_rect(sdl_rect(page_bounds));

    let scale_ml = ML * state.scale;
    let mut page_rect = Rectangle::new((0., 0.), viewport_size)
        .with_size_centered(page_properties.get_size())
        .add_y(state.scroll * ML)
        * scale_ml;

    canvas.draw_double_borders(
        page_rect,
        BorderType::Outside,
        page_border_color,
        page_color,
    )?;

    let texture_creator = canvas.texture_creator();

    let page_inner_width = (page_properties.page_size.width
        - page_properties.page_margin.left
        - page_properties.page_margin.right)
        * scale_ml;

    let page_inner_height = (page_properties.page_size.height
        - page_properties.page_margin.top
        - page_properties.page_margin.bottom)
        * scale_ml;

    let mut printer: Point = (
        page_rect.left_top.x + page_properties.page_margin.left * scale_ml,
        page_rect.left_top.y + page_properties.page_margin.top * scale_ml,
    )
        .into();

    if let Some(nodes) = &Arc::clone(&document.docx_document).content.nodes {
        for (idx, node) in nodes.iter().enumerate() {
            match node {
                crate::docx_document::DocxNode::Paragrapth {
                    properties, texts, ..
                } => {
                    let Some(textures) = get_texts_textures(texts, fonts, &texture_creator) else {
                        continue;
                    };
                    let lines = get_lines(textures, page_inner_width, scale_ml);

                    if idx != 0 {
                        printer.y += properties.spacing.before.unwrap_or(D_P_B_SP) * scale_ml;
                    }

                    for (line_idx, line) in lines.iter().enumerate() {
                        let Some(line_height) = line.iter().map(|v| v.query().height).max() else {
                            continue;
                        };
                        let line_height = scale_ml * line_height as f32 * FONT_ML;
                        let line_width = line.iter().map(|v| v.query().width).sum::<u32>() as f32;
                        let line_width_sp = line
                            .iter()
                            .map(|v| v.query().width as f32 + D_HOR_SP)
                            .sum::<f32>()
                            * scale_ml
                            * FONT_ML;

                        let height_from_page_top =
                            printer.y - page_rect.y() - page_properties.page_size.height * ML;
                        if page_inner_height - height_from_page_top < line_width_sp {
                            page_rect = page_rect.add_y(page_rect.height() + P_PAD);
                            printer.y = page_rect.y() + page_properties.page_margin.top * ML;

                            canvas.draw_double_borders(
                                page_rect,
                                BorderType::Outside,
                                page_border_color,
                                page_color,
                            )?;
                        }

                        match properties.justify {
                            Some(crate::docx_document::Justification::Start) | None => {
                                canvas.draw_words_from(
                                    printer,
                                    line,
                                    line_height,
                                    D_HOR_SP,
                                    scale_ml,
                                )?;
                            }
                            Some(crate::docx_document::Justification::Width) => {
                                let hor_sp = (page_inner_width - line_width) / line.len() as f32;
                                canvas.draw_words_from(
                                    printer,
                                    line,
                                    line_height,
                                    hor_sp,
                                    scale_ml,
                                )?;
                            }
                            Some(crate::docx_document::Justification::Center) => {
                                let start = (
                                    printer.x + (page_inner_width - line_width_sp) / 2.,
                                    printer.y,
                                );
                                canvas.draw_words_from(
                                    start,
                                    line,
                                    line_height,
                                    D_HOR_SP,
                                    scale_ml,
                                )?;
                            }
                            Some(crate::docx_document::Justification::End) => {
                                let start =
                                    ((printer.x + page_inner_width) - line_width_sp, printer.y);
                                canvas.draw_words_from(
                                    start,
                                    line,
                                    line_height,
                                    D_HOR_SP,
                                    scale_ml,
                                )?;
                            }
                        }

                        if line_idx != lines.len() - 1 {
                            printer.y +=
                                line_height + properties.spacing.line.unwrap_or(D_LN_SP) * scale_ml;
                        }
                    }

                    printer.y += properties.spacing.after.unwrap_or(D_P_A_SP) * scale_ml;
                }
                _ => {}
            }
        }
    }

    canvas.set_clip_rect(None);

    Ok(())
}

fn get_lines(textures: Vec<Texture>, page_inner_width: f32, scale_ml: f32) -> Vec<Vec<Texture>> {
    let mut curr_width = 0.;
    let mut lines = Vec::<Vec<Texture>>::new();
    for word in textures {
        let TextureQuery { width, .. } = word.query();

        if curr_width + width as f32 * scale_ml * FONT_ML <= page_inner_width {
            if let Some(last) = lines.last_mut() {
                last.push(word);
            } else {
                lines.push(vec![word]);
            }
        } else {
            curr_width = 0.0;
            lines.push(vec![word]);
        }
        curr_width += (width as f32 * FONT_ML + D_HOR_SP) * scale_ml;
    }
    lines
}

fn get_texts_textures<'a>(
    texts: &Vec<crate::docx_document::TextNode>,
    fonts: &std::collections::HashMap<u16, std::rc::Rc<sdl2::ttf::Font>>,
    texture_creator: &'a TextureCreator<WindowContext>,
) -> Option<Vec<Texture<'a>>> {
    texts
        .iter()
        .filter_map(|text| {
            let properties = text.properties.clone();

            let font = fonts.get(&(properties.size.map(|size| size.0 as u16).unwrap_or(12)))?;

            let words = text
                .content
                .as_str()
                .unicode_words()
                .filter_map(|word| {
                    let surface = font
                        .render(word)
                        .blended(
                            properties
                                .color
                                .clone()
                                .map(|color| color.0)
                                .unwrap_or(Color::RGB(0, 0, 0)),
                        )
                        .ok()?;
                    let texture = texture_creator.create_texture_from_surface(&surface).ok()?;
                    Some(texture)
                })
                .collect::<Vec<_>>();

            Some(words)
        })
        .reduce(|acc, v| acc.into_iter().chain(v.into_iter()).collect::<Vec<_>>())
}

fn draw_console(
    rect: impl Into<Rectangle>,
    canvas: &mut Canvas<Window>,
    state: &State,
    fonts: &Fonts<'_, '_>,
) -> anyhow::Result<()> {
    use BorderType::Outside;

    let ColorScheme {
        console_bg_color,
        console_fg_color,
        console_border_color,
        ..
    } = state.colorscheme.clone();

    let console_rect = rect.into().add_paddings(15. * ML);

    let console_font = fonts.get(&24).context("Font not found")?;

    canvas.draw_double_borders(
        console_rect,
        Outside,
        console_border_color,
        console_bg_color,
    )?;

    let mode_rect = draw_mode(
        console_rect,
        canvas,
        state,
        &console_font,
        console_fg_color,
        console_border_color,
    )?;

    draw_command_line(
        console_rect.move_left_top((mode_rect.width(), 0.)),
        canvas,
        state,
        &console_font,
        console_fg_color,
    )?;

    Ok(())
}

fn draw_command_line(
    console_rect: Rectangle,
    canvas: &mut Canvas<Window>,
    state: &State,
    console_font: &sdl2::ttf::Font,
    console_fg_color: Color,
) -> Result<(), anyhow::Error> {
    let input = state.console.input.as_str();
    if input == "" {
        return Ok(());
    }

    let mut texture_creator = canvas.texture_creator();
    let input_texture =
        texture_creator.create_text_texture(console_font, input, console_fg_color)?;
    let TextureQuery { width, height, .. } = input_texture.query();
    let ratio = width as f32 / height as f32;

    let mut input_rect = console_rect
        .add_paddings(5. * ML)
        .move_left_top((5. * ML, 0.));
    input_rect.right_bottom.x = input_rect
        .right_bottom
        .x
        .min(input_rect.left_top.x + (ratio * input_rect.height()));

    canvas.copy_rect(&input_texture, None, Some(input_rect))?;

    Ok(())
}

fn draw_mode(
    console_rect: Rectangle,
    canvas: &mut Canvas<Window>,
    state: &State,
    console_font: &sdl2::ttf::Font,
    console_fg_color: Color,
    console_border_color: Color,
) -> Result<Rectangle, anyhow::Error> {
    let mut texture_creator = canvas.texture_creator();

    let mode_text = state.mode.to_string();

    let mode_texture =
        texture_creator.create_text_texture(console_font, mode_text.as_str(), console_fg_color)?;
    let TextureQuery { width, height, .. } = mode_texture.query();

    let mode_rect = console_rect.add_paddings(5. * ML);
    let ratio = width as f32 / height as f32;
    let mode_rect = mode_rect.with_width(ratio * mode_rect.height() as f32);

    //   rect_width          text_width
    // -------------   =   -------------
    //  rect_height         text_height
    //
    //   text_width
    //  -------------  * rect_height = rect_width
    //   text_height

    canvas.set_draw_color(state.mode.get_bg_color(state.colorscheme.clone()));
    canvas.draw_with_borders(
        mode_rect,
        BorderType::Inside,
        2. * ML,
        console_border_color,
        state.mode.get_bg_color(state.colorscheme.clone()),
    )?;

    let text_paddings = Paddings::from((3., 3. * ratio)) * ML;
    canvas.copy_rect(
        &mode_texture,
        None,
        Some(mode_rect.add_paddings(text_paddings)),
    )?;

    Ok(mode_rect)
}

pub trait TextHelper {
    fn draw_words_from(
        &mut self,
        printer: impl Into<Point>,
        line: &Vec<Texture>,
        line_height: f32,
        horizontal_spacing: f32,
        scale_ml: f32,
    ) -> Result<(), anyhow::Error>;
}

impl TextHelper for Canvas<Window> {
    fn draw_words_from(
        &mut self,
        printer: impl Into<Point>,
        line: &Vec<Texture>,
        line_height: f32,
        horizontal_spacing: f32,
        scale_ml: f32,
    ) -> Result<(), anyhow::Error> {
        let mut from = printer.into();
        Ok(for (word_idx, word) in line.iter().enumerate() {
            let TextureQuery { width, height, .. } = word.query();
            let top_offset = line_height - height as f32 * scale_ml;
            let word_rect = Rectangle::new(
                (from.x, from.y + top_offset),
                (
                    scale_ml * FONT_ML * width as f32,
                    scale_ml * FONT_ML * height as f32,
                ),
            );

            self.copy_rect(word, None, word_rect)?;
            if word_idx != line.len() - 1 {
                from.x += (horizontal_spacing + width as f32 * FONT_ML) * scale_ml;
            }
        })
    }
}

pub trait TextureHelper {
    fn copy_rect(
        &mut self,
        texture: &Texture,
        src: impl Into<Option<Rectangle>>,
        des: impl Into<Option<Rectangle>>,
    ) -> anyhow::Result<()>;
}

impl TextureHelper for Canvas<Window> {
    fn copy_rect(
        &mut self,
        texture: &Texture,
        src: impl Into<Option<Rectangle>>,
        dest: impl Into<Option<Rectangle>>,
    ) -> anyhow::Result<()> {
        self.copy(texture, src.into().map(sdl_rect), dest.into().map(sdl_rect))
            .as_anyhow()
    }
}

pub trait TextSurfaceHelper {
    fn create_text_texture(
        &mut self,
        font: &sdl2::ttf::Font,
        text: &str,
        console_fg_color: Color,
    ) -> Result<Texture, anyhow::Error>;
}

impl TextSurfaceHelper for TextureCreator<WindowContext> {
    fn create_text_texture(
        &mut self,
        font: &sdl2::ttf::Font,
        text: &str,
        console_fg_color: Color,
    ) -> Result<Texture, anyhow::Error> {
        let text_surface = font
            .render(text)
            .blended(console_fg_color)
            .context("Failed to blend texture surface")?;

        self.create_texture_from_surface(&text_surface)
            .context("failed to create texture from text surface.")
    }
}

pub trait DrawWithBorders {
    fn draw_with_borders(
        &mut self,
        rect: impl Into<Rectangle>,
        border_type: BorderType,
        borders: impl Into<Paddings>,
        border_color: Color,
        bg_color: Color,
    ) -> anyhow::Result<()>;

    fn draw_double_borders(
        &mut self,
        rect: impl Into<Rectangle>,
        border_type: BorderType,
        border_color: Color,
        bg_color: Color,
    ) -> anyhow::Result<()>;
}

impl DrawWithBorders for Canvas<Window> {
    fn draw_with_borders(
        &mut self,
        rect: impl Into<Rectangle>,
        border_type: BorderType,
        borders: impl Into<Paddings>,
        border_color: Color,
        bg_color: Color,
    ) -> Result<(), anyhow::Error> {
        let rect = rect.into();
        let borders = borders.into();

        match border_type {
            BorderType::Outside => {
                self.set_draw_color(border_color);
                self.fill_rect(sdl_rect(rect.add_paddings(borders * (-1.0))))
                    .as_anyhow()?;

                self.set_draw_color(bg_color);
                self.fill_rect(sdl_rect(rect)).as_anyhow()
            }
            BorderType::Inside => {
                self.set_draw_color(border_color);
                self.fill_rect(sdl_rect(rect)).as_anyhow()?;

                self.set_draw_color(bg_color);
                self.fill_rect(sdl_rect(rect.add_paddings(borders)))
                    .as_anyhow()
            }
        }
    }

    fn draw_double_borders(
        &mut self,
        rect: impl Into<Rectangle>,
        border_type: BorderType,
        border_color: Color,
        bg_color: Color,
    ) -> anyhow::Result<()> {
        let rect = rect.into();
        let borders_outer = Paddings::from(10.0 * ML);
        let borders_between = Paddings::from(6.0 * ML);
        let borders_inner = Paddings::from(4.0 * ML);

        let multiplier = match border_type {
            BorderType::Outside => -1.,
            BorderType::Inside => 1.,
        };
        self.set_draw_color(border_color);
        self.fill_rect(sdl_rect(rect.add_paddings(borders_outer * multiplier)))
            .as_anyhow()?;

        self.set_draw_color(bg_color);
        self.fill_rect(sdl_rect(rect.add_paddings(borders_between * multiplier)))
            .as_anyhow()?;

        self.set_draw_color(border_color);
        self.fill_rect(sdl_rect(rect.add_paddings(borders_inner * multiplier)))
            .as_anyhow()?;

        self.set_draw_color(bg_color);
        self.fill_rect(sdl_rect(rect)).as_anyhow()?;
        Ok(())
    }
}

pub enum BorderType {
    Inside,
    Outside,
}
