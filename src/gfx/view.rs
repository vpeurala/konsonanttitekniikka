//! Drawing the game scaled to fit any screen, on top of the core crate's
//! `View`, which does the mapping. A camera scales the content to fill the
//! screen without distorting it, centering it with empty bands where the
//! aspect ratios differ.

use macroquad::prelude::*;

pub use lukuloitsu_core::arena::{ARENA_H, ARENA_W};
pub use lukuloitsu_core::geometry::Rect;
pub use lukuloitsu_core::view::View;

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
