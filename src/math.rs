#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    pub left_top: Point,
    pub right_bottom: Point,
}

#[derive(Debug, Clone, Copy)]
pub struct Paddings {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl From<(f32, f32)> for Point {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

impl From<(f32, f32)> for Size {
    fn from((width, height): (f32, f32)) -> Self {
        Self { width, height }
    }
}

impl From<(f32, f32, f32, f32)> for Rectangle {
    fn from((left, top, right, bottom): (f32, f32, f32, f32)) -> Self {
        Self {
            left_top: (left, top).into(),
            right_bottom: (right, bottom).into(),
        }
    }
}

impl<T: Into<Point>, P: Into<Point>> From<(T, P)> for Rectangle {
    fn from((left_top, right_bottom): (T, P)) -> Self {
        Self {
            left_top: left_top.into(),
            right_bottom: right_bottom.into(),
        }
    }
}

impl From<(f32, f32, f32, f32)> for Paddings {
    fn from((top, right, bottom, left): (f32, f32, f32, f32)) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}

impl From<(f32, f32)> for Paddings {
    fn from((top_bottom, right_left): (f32, f32)) -> Self {
        Self {
            top: top_bottom,
            right: right_left,
            bottom: top_bottom,
            left: right_left,
        }
    }
}

impl From<f32> for Paddings {
    fn from(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
}

impl Rectangle {
    pub fn new(top_left: impl Into<Point>, size: impl Into<Size>) -> Self {
        let top_left = top_left.into();

        let size = size.into();
        Self {
            left_top: top_left,
            right_bottom: (top_left.x + size.width, top_left.y + size.height).into(),
        }
    }

    pub fn add_paddings(self, paddings: impl Into<Paddings>) -> Self {
        let paddings = paddings.into();

        Self {
            left_top: (
                self.left_top.x + paddings.left,
                self.left_top.y + paddings.top,
            )
                .into(),
            right_bottom: (
                self.right_bottom.x - paddings.right,
                self.right_bottom.y - paddings.bottom,
            )
                .into(),
        }
    }

    pub fn get_point_and_size(self) -> (Point, Size) {
        (
            self.left_top,
            (
                self.right_bottom.x - self.left_top.x,
                self.left_top.y - self.right_bottom.y,
            )
                .into(),
        )
    }

    pub fn with_size(self, size: impl Into<Size>) -> Self {
        Self::new(self.left_top, size.into())
    }

    pub fn with_height(self, height: f32) -> Self {
        let (left_top, size) = self.get_point_and_size();
        Self::new(left_top, (size.width, height))
    }

    pub fn with_width(self, width: f32) -> Self {
        let (left_top, size) = self.get_point_and_size();
        Self::new(left_top, (width, size.height))
    }

    pub fn with_size_centered(self, size: impl Into<Size>) -> Self {
        let (left_top, prev_size) = self.get_point_and_size();
        let size = size.into();
        Self::new(
            (
                left_top.x - (size.width - prev_size.width) / 2.,
                left_top.y - (size.height - prev_size.height) / 2.,
            ),
            size,
        )
    }

    pub fn add_point(self, point: impl Into<Point>) -> Self {
        let point = point.into();
        let (left_top, size) = self.get_point_and_size();

        Self::new((left_top.x + point.x, left_top.y + point.y), size)
    }

    pub fn add_x(self, x: f32) -> Self {
        let (left_top, size) = self.get_point_and_size();

        Self::new((left_top.x + x, left_top.y), size)
    }

    pub fn add_y(self, y: f32) -> Self {
        let (left_top, size) = self.get_point_and_size();

        Self::new((left_top.x, left_top.y + y), size)
    }

    pub fn x(self) -> f32 {
        self.left_top.x
    }
    pub fn y(self) -> f32 {
        self.left_top.y
    }

    pub fn width(self) -> f32 {
        let (_left_top, size) = self.get_point_and_size();
        size.width
    }

    pub fn height(self) -> f32 {
        let (_left_top, size) = self.get_point_and_size();
        size.height
    }

    pub fn move_left_top(self, delta: impl Into<Point>) -> Self {
        let delta = delta.into();
        (
            (self.left_top.x + delta.x, self.left_top.y + delta.y),
            self.right_bottom,
        )
            .into()
    }

    pub fn move_right_bottom(self, delta: impl Into<Point>) -> Self {
        let delta = delta.into();
        (
            self.left_top,
            (self.right_bottom.x + delta.x, self.right_bottom.y + delta.y),
        )
            .into()
    }
}


impl From<(u32, u32)> for Size {
    fn from((width, height): (u32, u32)) -> Self {
        Self {
            width: width as f32,
            height: height as f32,
        }
    }
}
impl From<(i32, i32)> for Point {
    fn from((x, y): (i32, i32)) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
        }
    }
}

impl std::ops::Mul<f32> for Paddings {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            top: self.top * rhs,
            right: self.right * rhs,
            bottom: self.bottom * rhs,
            left: self.left * rhs,
        }
    }
}

impl std::ops::Mul<f32> for Size {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            width: self.width * rhs,
            height: self.height * rhs,
        }
    }
}

impl std::ops::Mul<f32> for Rectangle {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        self.with_size_centered(self.get_point_and_size().1 * rhs)
    }
}
