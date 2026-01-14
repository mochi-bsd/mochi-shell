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
    pub const BG_PRIMARY: Color = Color::rgb(30, 30, 35);
    pub const BG_SECONDARY: Color = Color::rgb(40, 40, 46);
    pub const BG_TERTIARY: Color = Color::rgb(20, 20, 24);
    
    pub const TEXT_PRIMARY: Color = Color::rgb(240, 240, 250);
    pub const TEXT_SECONDARY: Color = Color::rgb(200, 200, 210);
    pub const TEXT_TERTIARY: Color = Color::rgb(180, 180, 190);
    
    pub const BORDER: Color = Color::rgb(60, 60, 70);
    pub const BORDER_LIGHT: Color = Color::rgb(80, 80, 90);
    
    pub const ACCENT: Color = Color::rgb(100, 120, 255);
    pub const SUCCESS: Color = Color::rgb(80, 200, 120);
    pub const WARNING: Color = Color::rgb(255, 180, 80);
    pub const ERROR: Color = Color::rgb(255, 100, 100);
}
