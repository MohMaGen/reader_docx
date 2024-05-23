use super::{FontHandle, FontProperties};
use unicode_segmentation::UnicodeSegmentation;

impl super::DocxDocument {
    pub fn init_or_push_to_font(&mut self, font_name: String, text: String) -> FontHandle {
        self.fonts.init_or_push_to_font(font_name, text)
    }

    pub fn push_to_default_font(&mut self, text: String) -> FontHandle {
        self.fonts.push_to_default_font(text)
    }
}

impl super::FontTable {
    pub fn init_or_push_to_font(&mut self, font_name: String, text: String) -> FontHandle {
        let predicate = |FontProperties { name, .. }: &FontProperties| *name == font_name;

        let variants = text.graphemes(true).map(grapheme_to_i32);
        let fonts = &mut self.fonts;

        if let Some(handle) = fonts.iter().position(predicate) {
            variants.into_iter().for_each(|variant| {
                fonts[handle].variants.insert(variant);
            });
            handle
        } else {
            fonts.push(FontProperties {
                name: font_name,
                variants: variants.collect(),
            });
            fonts.len() - 1
        }
    }

    pub fn push_to_default_font(&mut self, text: String) -> FontHandle {
        let variants = text.graphemes(true).map(grapheme_to_i32);
        variants.into_iter().for_each(|variant| {
            self.fonts[0].variants.insert(variant);
        });
        0
    }
}

fn grapheme_to_i32(grapheme: &str) -> i32 {
    let mut zero = [0u8; 4];
    for (idx, byte) in grapheme.bytes().take(4).enumerate() {
        zero[3-idx] = byte;
    }

    i32::from_be_bytes(zero)
}
