use crate::Environment;
use raylib::RaylibHandle;

#[derive(Clone, Debug)]
pub struct Block {
    pub pos: (f32, f32),
    pub size: (f32, f32),
    // top, right, down, left
    pub padding: (f32, f32, f32, f32),
}

pub trait Scalable {
    fn scale_by(&self, v: f32) -> Self;

    fn scale(&self, env: &Environment) -> Self
    where
        Self: std::marker::Sized,
    {
        self.scale_by(env.get_scale())
    }
}

pub trait Scrolable {
    fn scroll_by(&self, v: f32) -> Self;

    fn scroll(&self, env: &Environment) -> Self
    where
        Self: std::marker::Sized,
    {
        self.scroll_by(env.get_scroll())
    }
}

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Alignment {
    pub horizontal: AlignmentHorizontal,
    pub vertical: AlignmentVertical,
}

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlignmentHorizontal {
    #[default]
    Left,
    Right,
    Center,
}

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlignmentVertical {
    #[default]
    Top,
    Bottom,
    Horizon,
}

impl Alignment {
    pub fn new(vertical: AlignmentVertical, horizontal: AlignmentHorizontal) -> Alignment {
        Self {
            horizontal,
            vertical,
        }
    }

    pub fn vertical(vertical: AlignmentVertical) -> Alignment {
        Self {
            vertical,
            horizontal: AlignmentHorizontal::Left,
        }
    }
    pub fn horizontal(horizontal: AlignmentHorizontal) -> Alignment {
        Self {
            horizontal,
            vertical: AlignmentVertical::Top,
        }
    }
}

impl Block {
    pub fn new(size: (f32, f32)) -> Self {
        Self {
            pos: (0., 0.),
            size,
            padding: (0., 0., 0., 0.),
        }
    }

    pub fn calc_pos(&mut self, alg: Alignment, parent: Block) -> (f32, f32) {
        let pos = parent.get_child_pos(alg, self.clone());
        self.pos = pos;
        pos
    }

    pub fn get_child_pos(&self, alg: Alignment, block: Block) -> (f32, f32) {
        use AlignmentHorizontal::*;
        use AlignmentVertical::*;

        let x = match alg.horizontal {
            Right => self.pos.0 + self.size.0 - self.padding.1 - block.size.0,
            Left => self.pos.0 + self.padding.3,
            Center => {
                self.pos.0
                    + self.padding.3
                    + (self.size.0 - self.padding.1 - self.padding.3 - block.size.0) / 2.
            }
        };

        let y = match alg.vertical {
            Top => self.pos.1 + self.padding.0,
            Bottom => self.pos.1 + self.size.1 - self.padding.2 - block.size.1,
            Horizon => {
                self.pos.1
                    + self.padding.0
                    + (self.size.1 - self.padding.0 - self.padding.2 - block.size.1) / 2.
            }
        };

        (x, y)
    }

    pub fn window_block(rl: &RaylibHandle) -> Self {
        Self {
            pos: (0., 0.),
            size: (rl.get_screen_width() as f32, rl.get_screen_height() as f32),
            padding: (0., 0., 0., 0.),
        }
    }

    pub fn map_size<F>(&self, f: F) -> Self
    where
        F: Fn((f32, f32)) -> (f32, f32),
    {
        Self {
            size: f(self.size),
            ..self.clone()
        }
    }

    pub fn map_pos<F>(&self, f: F) -> Self
    where
        F: Fn((f32, f32)) -> (f32, f32),
    {
        Self {
            pos: f(self.pos),
            ..self.clone()
        }
    }

    pub fn add_top_padding(&mut self, v: f32) {
        self.padding.0 += v;
    }
}

impl Scalable for Block {
    fn scale_by(&self, v: f32) -> Self {
        Self {
            size: self.size.scale_by(v),
            padding: self.padding.scale_by(v),
            ..(*self)
        }
    }
}

impl Scrolable for Block {
    fn scroll_by(&self, v: f32) -> Self {
        Self {
            pos: self.pos.scroll_by(v),
            ..self.clone()
        }
    }
}

impl Scalable for (f32, f32) {
    fn scale_by(&self, v: f32) -> Self {
        (self.0 * v, self.1 * v)
    }
}

impl Scrolable for (f32, f32) {
    fn scroll_by(&self, v: f32) -> Self {
        (self.0, self.1 + v)
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
