use glam::u32;
use rusttype::PositionedGlyph;
use wgpu::util::DeviceExt;

use crate::{docx_document::Color, draw::DrawState, math, uniforms::Uniforms2d};

#[derive(Default)]
pub struct Primitive {
    pub prop: PrimitiveProperties,
    pub wgpu: PrimitiveWgpu,
}

#[derive(Clone, Default)]
pub enum PrimitiveProperties {
    Rect {
        rect: math::Rectangle,
        color: Color,
    },
    PlainText(PlainTextProperties),

    #[default]
    Empty,
}

#[derive(Clone)]
pub struct PlainTextProperties {
    pub left_top: math::Point,
    pub content: String,
    pub font: rusttype::Font<'static>,
    pub color: Color,
    pub scale: f32,
}

#[derive(Default)]
pub enum PrimitiveWgpu {
    Rect {
        uniform: Uniforms2d,
        buffer: wgpu::Buffer,
        bindgroup: wgpu::BindGroup,
    },
    Text {
        uniform: Uniforms2d,
        buffer: wgpu::Buffer,
        texture: wgpu::Texture,
        extent: wgpu::Extent3d,
        bindgroup: wgpu::BindGroup,
        glyphs: Vec<PositionedGlyph<'static>>,
    },
    #[default]
    Empty,
}

impl DrawState<'_> {
    pub fn new_prim(&self, prop: impl Into<PrimitiveProperties>) -> Primitive {
        match prop.into() {
            PrimitiveProperties::Rect { rect, color } => self.new_rect(rect, color),
            PrimitiveProperties::PlainText(prop) => self.new_plain_text(prop),
            _ => Default::default(),
        }
    }

    pub fn update_prim(&self, prop: impl Into<PrimitiveProperties>, primitive: &mut Primitive) {
        if primitive.is_empty() {
            *primitive = self.new_prim(prop);
            return;
        }

        match prop.into() {
            PrimitiveProperties::Rect { rect, color } => self.update_rect(rect, color, primitive),
            PrimitiveProperties::PlainText(prop) => self.update_plain_text(prop, primitive),
            _ => {}
        }
    }

    pub fn draw_prim<'a, 'b: 'a>(
        &'b self,
        rpass: &mut wgpu::RenderPass<'a>,
        primitive: &'a Primitive,
    ) {
        match &primitive.wgpu {
            PrimitiveWgpu::Rect { bindgroup, .. } => {
                log::info!(
                    "( draw rect )\n{:?}",
                    primitive.get_rect().get_point_and_size()
                );

                rpass.push_debug_group("Draw Rect Primitive");

                rpass.set_pipeline(&self.fill_pipeline.pipeline);
                rpass.set_bind_group(0, bindgroup, &[]);
                rpass.set_vertex_buffer(0, self.fill_pipeline.vertex_buffer.slice(..));
                rpass.draw(0..6, 0..1);

                rpass.pop_debug_group();
            }
            PrimitiveWgpu::Text { bindgroup, .. } => {
                log::info!(
                    "( draw text )\n{:?}",
                    primitive.get_rect().get_point_and_size()
                );

                rpass.push_debug_group("Draw Plain Text Primitive");

                rpass.set_pipeline(&self.text_pipeline.pipeline);
                rpass.set_bind_group(0, bindgroup, &[]);
                rpass.set_vertex_buffer(0, self.text_pipeline.vertex_buffer.slice(..));
                rpass.draw(0..6, 0..1);

                rpass.pop_debug_group();
            }
            _ => {}
        }
    }

    pub fn draw_and_update<'a, 'b: 'a>(
        &'b self,
        rpass: &mut wgpu::RenderPass<'a>,
        prop: impl Into<PrimitiveProperties>,
        primitive: &'a mut Primitive,
    ) {
        self.update_prim(prop, primitive);
        self.draw_prim(rpass, primitive);
    }
}

impl DrawState<'_> {
    fn new_plain_text(&self, prop: PlainTextProperties) -> Primitive {
        if prop.content.is_empty() {
            return Primitive::default();
        }

        let v_m = prop.font.v_metrics(rusttype::Scale::uniform(prop.scale));
        let glyphs = prop
            .font
            .layout(
                prop.content.as_str(),
                rusttype::Scale::uniform(prop.scale),
                rusttype::Point {
                    x: 0f32,
                    y: v_m.ascent,
                },
            )
            .collect::<Vec<_>>();

        let size = get_glyphs_size(&glyphs, v_m);

        if size.0.ceil() as u32 == 0 || size.1.ceil() as u32 == 0 {
            return Default::default();
        }

        let uniform = self.calc_rect_uniform(math::Rectangle::new(prop.left_top, size), prop.color);

        let extent = wgpu::Extent3d {
            width: size.0.ceil() as u32,
            height: size.1.ceil() as u32,
            depth_or_array_layers: 1,
        };
        let mut texels = vec![0u8; (extent.width * extent.height) as usize];
        for glyph in &glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, v| {
                    let x = x as i32 + bounding_box.min.x;
                    let y = extent.height  as i32 - (y as i32 + bounding_box.min.y);

                    if let Some(pxl) = texels.get_mut((x + y * extent.width as i32) as usize) {
                        *pxl = (v * 255.0) as u8;
                    }
                });
            }
        }

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.queue.write_texture(
            texture.as_image_copy(),
            &texels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(extent.width),
                rows_per_image: None,
            },
            extent,
        );

        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bindgroup = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.text_pipeline.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });

        Primitive {
            prop: PrimitiveProperties::PlainText(prop),
            wgpu: PrimitiveWgpu::Text {
                uniform,
                buffer,
                texture,
                extent,
                bindgroup,
                glyphs,
            },
        }
    }

    fn update_plain_text(&self, new_prop: PlainTextProperties, primitive: &mut Primitive) {
        if let Primitive {
            prop: PrimitiveProperties::PlainText(prop),
            wgpu:
                PrimitiveWgpu::Text {
                    uniform,
                    buffer,
                    extent,
                    ..
                },
        } = primitive
        {
            if prop.content == new_prop.content && prop.scale == new_prop.scale {
                let uniform_value = self.calc_rect_uniform(
                    math::Rectangle::new(
                        new_prop.left_top,
                        (extent.width as f32, extent.height as f32),
                    ),
                    new_prop.color,
                );
                *prop = new_prop;

                *uniform = uniform_value;
                self.queue
                    .write_buffer(buffer, 0, bytemuck::cast_slice(&[uniform_value]));
            } else {
                *primitive = self.new_plain_text(new_prop);
            }
        }
    }

    fn new_rect(&self, rect: math::Rectangle, color: Color) -> Primitive {
        let uniform = self.calc_rect_uniform(rect, color);
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bindgroup = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.fill_pipeline.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
        });

        Primitive {
            prop: PrimitiveProperties::Rect { rect, color },
            wgpu: PrimitiveWgpu::Rect {
                uniform,
                buffer,
                bindgroup,
            },
        }
    }

    fn update_rect(
        &self,
        new_rect: impl Into<math::Rectangle>,
        new_color: Color,
        primitive: &mut Primitive,
    ) {
        let new_rect = new_rect.into();
        if let Primitive {
            wgpu:
                PrimitiveWgpu::Rect {
                    uniform,
                    buffer: uniform_buffer,
                    ..
                },
            ..
        } = primitive
        {
            let uniform_value = self.calc_rect_uniform(new_rect, new_color);
            primitive.prop = (new_rect, new_color).into();
            *uniform = uniform_value;
            self.queue
                .write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&[uniform_value]));
        }
    }

    fn calc_rect_uniform(&self, rect: impl Into<math::Rectangle>, color: Color) -> Uniforms2d {
        let rect: math::Rectangle = rect.into();
        let (math::Point { x, y }, math::Size { width, height }) = rect.get_point_and_size();

        let (w_width, w_height) = (self.config.width as f32, self.config.height as f32);
        let (x, y, width, height) = (
            (x + width / 2.) / w_width * 2. - 1.,
            1. - (y + height / 2.) / w_height * 2.,
            width / w_width,
            height / w_height,
        );

        let translation = glam::Mat4::from_translation(glam::Vec3 { x, y, z: 0. });
        let scale = glam::Mat4::from_scale(glam::Vec3 {
            x: width,
            y: height,
            z: 1.,
        });

        let uniform = Uniforms2d {
            color: color.as_array(),
            transform: *(translation * scale).as_ref(),
        };
        uniform
    }
}

fn get_glyphs_size(
    glyphs: &[rusttype::PositionedGlyph<'_>],
    v_m: rusttype::VMetrics,
) -> (f32, f32) {
    let width = {
        let min_x = glyphs
            .iter()
            .filter_map(|g| g.pixel_bounding_box())
            .map(|g| g.min.x)
            .next()
            .unwrap_or(0);

        let max_x = glyphs
            .iter()
            .rev()
            .filter_map(|g| g.pixel_bounding_box())
            .map(|g| g.max.x)
            .next()
            .unwrap_or(0);
        (max_x - min_x) as f32
    };
    let height = { (v_m.ascent - v_m.descent).ceil() };
    (width, height)
}

impl<Rect: Into<math::Rectangle>, Colour: Into<Color>> From<(Rect, Colour)>
    for PrimitiveProperties
{
    fn from((rect, color): (Rect, Colour)) -> Self {
        Self::Rect {
            rect: rect.into(),
            color: color.into(),
        }
    }
}

impl From<PlainTextProperties> for PrimitiveProperties {
    fn from(prop: PlainTextProperties) -> Self {
        Self::PlainText(prop)
    }
}

impl Primitive {
    pub fn is_empty(&self) -> bool {
        matches!(
            self,
            Primitive {
                prop: PrimitiveProperties::Empty,
                wgpu: PrimitiveWgpu::Empty
            }
        )
    }

    pub fn get_rect_mut(&mut self) -> Option<&mut math::Rectangle> {
        match self {
            Primitive {
                prop: PrimitiveProperties::Rect { rect, .. },
                ..
            } => Some(rect),
            _ => None,
        }
    }

    pub fn get_rect(&self) -> math::Rectangle {
        match self {
            Primitive {
                prop: PrimitiveProperties::Rect { rect, .. },
                ..
            } => *rect,
            Primitive {
                prop: PrimitiveProperties::PlainText(PlainTextProperties { left_top, .. }),
                wgpu: PrimitiveWgpu::Text { extent, .. },
            } => math::Rectangle::new(*left_top, (extent.width as f32, extent.height as f32)),
            _ => Default::default(),
        }
    }

    pub fn get_glyphs(&self) -> Option<&[PositionedGlyph]> {
        match &self.wgpu {
            PrimitiveWgpu::Text { glyphs, .. } => Some(glyphs),
            _ => None,
        }
    }
}

impl PlainTextProperties {
    pub fn new(
        rect: impl Into<math::Rectangle>,
        color: impl Into<Color>,
        content: String,
        font: rusttype::Font<'static>,
    ) -> Self {
        let (rect, color) = (rect.into(), color.into());
        let (left_top, size) = rect.get_point_and_size();

        let height = {
            let v_m = font.v_metrics(rusttype::Scale::uniform(1.));
            v_m.ascent - v_m.descent
        };
        let scale = size.height / height;

        Self {
            left_top,
            color,
            content,
            font,
            scale,
        }
    }
}

impl std::fmt::Debug for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.prop {
            PrimitiveProperties::Rect { rect, color } => {
                write!(f, "RECT ({:?}, {:?})", rect, color)
            }
            PrimitiveProperties::PlainText(PlainTextProperties {
                left_top,
                content,
                color,
                scale,
                ..
            }) => write!(f, "TEXT ( {left_top:?}, {content:?}, {color:?}, {scale:?})"),
            PrimitiveProperties::Empty => write!(f, "Text"),
        }
    }
}

impl PrimitiveProperties {
    pub fn scroll(self, delta: f32) -> Self {
        match self {
            PrimitiveProperties::Rect { rect, color } => Self::Rect {
                rect: rect.add_y(delta),
                color,
            },
            PrimitiveProperties::PlainText(PlainTextProperties {
                left_top,
                content,
                font,
                color,
                scale,
            }) => Self::PlainText(PlainTextProperties {
                left_top: (left_top.x, left_top.y + delta).into(),
                content,
                font,
                color,
                scale,
            }),
            PrimitiveProperties::Empty => PrimitiveProperties::Empty,
        }
    }

    pub fn scale(self, ratio: f32) -> Self {
        match self {
            PrimitiveProperties::Rect { rect, color } => Self::Rect {
                rect: rect.with_size(rect.size() * ratio),
                color,
            },
            PrimitiveProperties::PlainText(PlainTextProperties {
                left_top,
                content,
                font,
                color,
                scale,
            }) => Self::PlainText(PlainTextProperties {
                left_top: (left_top.x, left_top.y + ratio).into(),
                content,
                font,
                color,
                scale: scale * ratio,
            }),
            PrimitiveProperties::Empty => PrimitiveProperties::Empty,
        }
    }
}

impl Primitive {
    pub fn get_scale(&self) -> f32 {
        match self.prop {
            PrimitiveProperties::Rect { rect, .. } => rect.height(),
            PrimitiveProperties::PlainText(PlainTextProperties { scale, .. }) => scale,
            PrimitiveProperties::Empty => 0.,
        }
    }
}
