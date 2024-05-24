use embedded_graphics::mono_font::{iso_8859_1, MonoFont};

pub enum FontSize {
    H4,
    H5,
    H6,
    H7,
    H8,
    H9,
    H10,
}

impl FontSize {
    pub fn to_font(&self) -> &'static MonoFont<'static> {
        match self {
            FontSize::H4 => &iso_8859_1::FONT_4X6,
            FontSize::H5 => &iso_8859_1::FONT_5X7,
            FontSize::H6 => &iso_8859_1::FONT_6X9,
            FontSize::H7 => &iso_8859_1::FONT_7X13,
            FontSize::H8 => &iso_8859_1::FONT_8X13,
            FontSize::H9 => &iso_8859_1::FONT_9X15,
            FontSize::H10 => &iso_8859_1::FONT_10X20,
        }
    }
}
