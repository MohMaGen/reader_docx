use std::sync::LockResult;
use colored::Colorize;



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

pub trait AllSame {
    type Item;

    fn all_same(v: Self::Item) -> Self;
}


pub trait AsAnyhow {
    type Item;
    fn to_anyhow(self) -> anyhow::Result<Self::Item>;
}

impl<T> AsAnyhow for Result<T, String> {
    type Item = T;

    fn to_anyhow(self) -> anyhow::Result<Self::Item> {
        self.map_err(anyhow::Error::msg)
    }
}

impl<T> AsAnyhow for LockResult<T> {
    type Item = T;

    fn to_anyhow(self) -> anyhow::Result<Self::Item> {
        self.map_err(|err| anyhow::Error::msg(err.to_string()))
    }
}

pub fn display_error(err: &anyhow::Error) {
    eprintln!(
        "{}: `{}`\n\n{:?}\n\n",
        "[ error ]".on_red().bold(),
        err.to_string().red().bold(),
        err
    );
}

