pub trait Scale {
    fn scale(self, v: f32) -> Self;
}

pub trait Scroll {
    fn scroll(self, v: f32) -> Self;
}

pub trait MakeWider {
    type Value;

    fn make_wider(self, v: impl Into<Self::Value>) -> Self;
}

impl<T> MakeWider for iced::Rectangle<T>
where
    T: std::ops::Add<T, Output = T> + std::ops::Sub<T, Output = T> + Copy,
{
    type Value = T;

    fn make_wider(self, v: impl Into<Self::Value>) -> Self {
        let v = v.into();
        Self {
            x: self.x + v,
            y: self.y + v,
            width: self.width - v - v,
            height: self.height - v - v,
        }
    }
}

pub trait AllSame {
    type Item;

    fn all_same(v: Self::Item) -> Self;
}

impl<T: Copy> AllSame for iced::Rectangle<T> {
    type Item = T;

    fn all_same(v: Self::Item) -> Self {
        Self {
            x: v,
            y: v,
            width: v,
            height: v,
        }
    }
}
