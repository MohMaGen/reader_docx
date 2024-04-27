use raylib::RaylibHandle;
use crate::Environment;


#[derive(Clone, Debug)]
pub struct Block {
    pub pos: (f32, f32),
    pub size: (f32, f32),
    // top, right, down, left
    pub paddings: (f32, f32, f32, f32),
}

pub trait Scalable {
    fn scale_by(&self, v: f32) -> Self;

    fn scale(&self, env: &Environment) -> Self where Self: std::marker::Sized {
        self.scale_by(env.get_scale())
    }
}

pub trait Scrolable {
    fn scroll_by(&self, v: f32) -> Self;

    fn scroll(&self, env: &Environment) -> Self where Self: std::marker::Sized {
        self.scroll_by(env.get_scroll())
    }
}

pub enum Alignment {
    Right,
    Left,
    Center,
}

impl Block {
    pub fn get_child_pos(
        &self,
        alg: Alignment,
        (width, _): (f32, f32),
        margin: (f32, f32, f32, f32),
    ) -> (f32, f32) {
        use Alignment::*;

        let Block {
            pos: (x, y),
            size: (block_width, _),
            paddings: (top, right, _, left),
        } = *self;

        let (m_top, m_right, _m_down, m_left) = margin;

        match alg {
            Left => (x + left + m_left, y + top + m_top),
            Right => (x + block_width - right - m_right, y + top + m_top),
            Center => (
                x + left + (block_width - left - right) / 2. - width / 2.,
                y + top + m_top,
            ),
        }
    }

    pub fn window_block(rl: &RaylibHandle) -> Self {
        Self {
            pos: (0., 0.),
            size: (rl.get_screen_width() as f32, rl.get_screen_height() as f32),
            paddings: (0., 0., 0., 0.),
        }
    }

    pub fn map_size<F>(&self, f: F) -> Self
        where F: Fn((f32, f32)) -> (f32, f32)
    {
        Self {
            size: f(self.size),
            ..self.clone()
        }
    }

    pub fn map_pos<F>(&self, f: F) -> Self
        where F: Fn((f32, f32)) -> (f32, f32)
    {
        Self {
            pos: f(self.pos),
            ..self.clone()
        }
    }

    pub fn add_top_padding(&mut self, v: f32) {
        self.paddings.0 += v;
    }
}

impl Scalable for Block {
    fn scale_by(&self, v: f32) -> Self {
        Self {
            size: self.size.scale_by(v),
            paddings: self.paddings.scale_by(v),
            ..(*self)
        }
    }
}

impl Scrolable for Block {
    fn scroll_by(&self, v: f32) -> Self {
        Self {
            pos: self.pos.scroll_by(v).clone(),
            ..self.clone()
        }
    }
}

impl Scalable for (f32, f32) {
    fn scale_by(&self, v: f32) -> Self {
        (self.0 * v, self.1 * v).clone()
    }
}

impl Scrolable for (f32, f32) {
    fn scroll_by(&self, v: f32) -> Self {
        (self.0, self.1 + v).clone()
    }
}

impl Scalable for f32 {
    fn scale_by(&self, v: f32) -> Self {
        *self * v
    }
}

impl Scalable for (f32, f32, f32, f32) {
    fn scale_by(&self, v: f32) -> Self {
        (self.0 * v, self.1 * v, self.2 * v, self.3 * v)
    }
}
