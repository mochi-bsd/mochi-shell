#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn as_tuple(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    // Dark mode color palette
    pub const BG_PRIMARY: Color = Color::rgb(40, 40, 40);
    pub const BG_SECONDARY: Color = Color::rgb(50, 50, 50);
    pub const BG_TERTIARY: Color = Color::rgb(30, 30, 30);

    pub const TEXT_PRIMARY: Color = Color::rgb(255, 255, 255);
    pub const TEXT_SECONDARY: Color = Color::rgb(200, 200, 200);
    pub const TEXT_TERTIARY: Color = Color::rgb(150, 150, 150);

    pub const BORDER: Color = Color::rgb(70, 70, 70);
    pub const BORDER_LIGHT: Color = Color::rgb(90, 90, 90);

    pub const ACCENT: Color = Color::rgb(0, 122, 255);
    pub const SUCCESS: Color = Color::rgb(52, 199, 89);
    pub const WARNING: Color = Color::rgb(255, 149, 0);
    pub const ERROR: Color = Color::rgb(255, 59, 48);
}
