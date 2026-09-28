//! Fits the game onto any screen. The game is laid out in fixed virtual
//! units: the arena is always 800 x 600, with the touch keypad beside it
//! on touch screens. A camera scales that content to fill the screen
//! without distorting it, centering it with empty bands where the aspect
//! ratios differ.

use std::sync::Mutex;

use macroquad::prelude::*;

pub const ARENA_W: f32 = 800.0;
pub const ARENA_H: f32 = 600.0;

/// Maps between screen points and virtual units for one frame.
#[derive(Debug, Clone, Copy)]
pub struct View {
    /// The virtual point at the top left corner of the screen.
    origin: Vec2,
    /// Screen points per virtual unit.
    scale: f32,
}

impl View {
    pub fn to_virtual(self, screen: Vec2) -> Vec2 {
        screen / self.scale + self.origin
    }

    pub fn to_screen(self, virtual_pos: Vec2) -> Vec2 {
        (virtual_pos - self.origin) * self.scale
    }

    /// Everything visible on screen, in virtual units, including the bands
    /// around the content.
    pub fn visible(self) -> Rect {
        let size = vec2(screen_width(), screen_height()) / self.scale;
        Rect::new(self.origin.x, self.origin.y, size.x, size.y)
    }
}

static CURRENT: Mutex<Option<View>> = Mutex::new(None);

/// Starts drawing content of the given virtual size, scaled to fit and
/// centered on the screen. Returns the mapping for this frame.
pub fn begin(content_w: f32, content_h: f32) -> View {
    let screen = vec2(screen_width(), screen_height());
    let scale = (screen.x / content_w).min(screen.y / content_h);
    let visible = screen / scale;
    let origin = -(visible - vec2(content_w, content_h)) / 2.0;
    // Like `Camera2D::from_display_rect`, but with y pointing down, as on
    // the screen.
    set_camera(&Camera2D {
        target: origin + visible / 2.0,
        zoom: vec2(2.0 / visible.x, 2.0 / visible.y),
        ..Default::default()
    });
    let view = View { origin, scale };
    *CURRENT.lock().unwrap() = Some(view);
    view
}

/// Paints over everything outside the content, so things partly off the
/// arena, like monsters coming in from an edge, don't show in the bands
/// around it.
pub fn mask_outside(content_w: f32, content_h: f32, color: Color) {
    let screen = current().visible();
    let (left, top) = (screen.x, screen.y);
    let (right, bottom) = (screen.x + screen.w, screen.y + screen.h);
    draw_rectangle(left, top, screen.w, -top, color);
    draw_rectangle(left, content_h, screen.w, bottom - content_h, color);
    draw_rectangle(left, 0.0, -left, content_h, color);
    draw_rectangle(content_w, 0.0, right - content_w, content_h, color);
}

/// The view set up by the latest `begin`.
pub fn current() -> View {
    CURRENT
        .lock()
        .unwrap()
        .expect("view::begin should run first")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(screen: Vec2, content: Vec2) -> View {
        let scale = (screen.x / content.x).min(screen.y / content.y);
        View {
            origin: -(screen / scale - content) / 2.0,
            scale,
        }
    }

    #[test]
    fn a_matching_window_maps_one_to_one() {
        let v = view(vec2(800.0, 600.0), vec2(800.0, 600.0));
        assert_eq!(v.to_virtual(vec2(123.0, 456.0)), vec2(123.0, 456.0));
    }

    #[test]
    fn a_wide_phone_centers_the_content_between_bands() {
        let v = view(vec2(844.0, 390.0), vec2(800.0, 600.0));
        // Height fits exactly; the width has equal bands on both sides.
        assert!((v.to_virtual(vec2(0.0, 0.0)).y).abs() < 0.001);
        let left = v.to_virtual(vec2(0.0, 0.0)).x;
        let right = v.to_virtual(vec2(844.0, 0.0)).x;
        assert!((left + (right - 800.0)).abs() < 0.001);
    }

    #[test]
    fn screen_and_virtual_positions_round_trip() {
        let v = view(vec2(844.0, 390.0), vec2(1130.0, 600.0));
        let p = vec2(300.0, 200.0);
        assert!(v.to_screen(v.to_virtual(p)).distance(p) < 0.001);
    }
}
