//! Fits the game onto any screen. The game is laid out in fixed virtual
//! units: the arena is always 800 x 600, with the touch keypad beside it
//! on touch screens. A camera scales that content to fill the screen
//! without distorting it, centering it with empty bands where the aspect
//! ratios differ.

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
    /// The size of the screen, in screen points.
    screen: Vec2,
}

impl View {
    /// The view that scales `content`, a rectangle in virtual units, to
    /// fit a screen of the given size and centers it.
    pub fn fit(content: Rect, screen: Vec2) -> View {
        let scale = (screen.x / content.w).min(screen.y / content.h);
        let visible = screen / scale;
        let origin = content.point() - (visible - content.size()) / 2.0;
        View {
            origin,
            scale,
            screen,
        }
    }

    pub fn to_virtual(self, screen: Vec2) -> Vec2 {
        screen / self.scale + self.origin
    }

    pub fn to_screen(self, virtual_pos: Vec2) -> Vec2 {
        (virtual_pos - self.origin) * self.scale
    }

    /// Everything visible on screen, in virtual units, including the bands
    /// around the content.
    pub fn visible(self) -> Rect {
        let size = self.screen / self.scale;
        Rect::new(self.origin.x, self.origin.y, size.x, size.y)
    }
}

/// Starts drawing `content`, a rectangle in virtual units, scaled to fit
/// and centered on the screen. Returns the mapping for this frame.
pub fn begin(content: Rect) -> View {
    let view = View::fit(content, vec2(screen_width(), screen_height()));
    let visible = view.visible();
    // Like `Camera2D::from_display_rect`, but with y pointing down, as on
    // the screen.
    set_camera(&Camera2D {
        target: visible.center(),
        zoom: vec2(2.0 / visible.w, 2.0 / visible.h),
        ..Default::default()
    });
    view
}

/// Paints over everything outside the content, so things partly off the
/// arena, like monsters coming in from an edge, don't show in the bands
/// around it.
pub fn mask_outside(view: View, content: Rect, color: Color) {
    let screen = view.visible();
    let (left, top) = (screen.x, screen.y);
    let (right, bottom) = (screen.x + screen.w, screen.y + screen.h);
    let (c_left, c_top) = (content.x, content.y);
    let (c_right, c_bottom) = (content.x + content.w, content.y + content.h);
    draw_rectangle(left, top, screen.w, c_top - top, color);
    draw_rectangle(left, c_bottom, screen.w, bottom - c_bottom, color);
    draw_rectangle(left, c_top, c_left - left, content.h, color);
    draw_rectangle(c_right, c_top, right - c_right, content.h, color);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(screen: Vec2, content: Vec2) -> View {
        View::fit(Rect::new(0.0, 0.0, content.x, content.y), screen)
    }

    #[test]
    fn content_left_of_zero_is_centered_too() {
        // Content spanning -100..700.
        let content = Rect::new(-100.0, 0.0, 800.0, 600.0);
        let v = View::fit(content, vec2(1600.0, 600.0));
        let left = v.to_virtual(vec2(0.0, 0.0)).x;
        let right = v.to_virtual(vec2(1600.0, 0.0)).x;
        assert!(((-100.0 - left) - (right - 700.0)).abs() < 0.001);
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

    #[test]
    fn the_visible_area_covers_the_content_and_the_bands() {
        let v = view(vec2(1600.0, 600.0), vec2(800.0, 600.0));
        let visible = v.visible();
        assert!((visible.w - 1600.0).abs() < 0.001);
        assert!((visible.h - 600.0).abs() < 0.001);
        assert!(visible.x < 0.0, "there are bands on both sides");
    }
}
