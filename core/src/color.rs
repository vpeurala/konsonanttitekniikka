//! A colour, for the sparks and lightning the rules describe. It has the
//! values of macroquad's named colours so the game looks the same; the
//! shell converts it when drawing.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }
}

pub const YELLOW: Color = Color::new(0.99, 0.98, 0.00, 1.00);
pub const GOLD: Color = Color::new(1.00, 0.80, 0.00, 1.00);
pub const ORANGE: Color = Color::new(1.00, 0.63, 0.00, 1.00);
pub const PINK: Color = Color::new(1.00, 0.43, 0.76, 1.00);
pub const RED: Color = Color::new(0.90, 0.16, 0.22, 1.00);
pub const MAROON: Color = Color::new(0.75, 0.13, 0.22, 1.00);
pub const SKYBLUE: Color = Color::new(0.40, 0.75, 1.00, 1.00);
pub const VIOLET: Color = Color::new(0.53, 0.24, 0.75, 1.00);
pub const SLIME: Color = Color::new(0.55, 0.7, 0.2, 1.00);
pub const MOULD_GREEN: Color = Color::new(0.35, 0.5, 0.15, 1.00);
pub const WHITE: Color = Color::new(1.00, 1.00, 1.00, 1.00);
pub const MAGENTA: Color = Color::new(1.00, 0.00, 1.00, 1.00);
