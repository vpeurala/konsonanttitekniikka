//! The app icon: the heroine casting a spell at a one-eyed monster in
//! front of a portal. Drawn with the game's own sprites and saved as PNG
//! files by `cargo run -- --render-icons`.

use macroquad::prelude::*;

use crate::sprites::{draw_cyclops, draw_girl};

/// The icon's background, also used as Android's adaptive icon background.
pub const BACKGROUND: Color = Color::new(0.14, 0.1, 0.3, 1.0);
const GLOW: Color = Color::new(0.55, 0.3, 0.9, 1.0);

/// The icon is drawn in a square of this many units.
const WORLD: f32 = 100.0;

/// Renders the icon at `size` x `size` pixels. With `background`, it fills
/// the square; without, only the characters are drawn, shrunk by
/// `content_scale` around the center, on a transparent background.
fn render(size: u32, background: bool, content_scale: f32) -> Image {
    let target = render_target(size, size);
    target.texture.set_filter(FilterMode::Linear);
    // The icon's content, zoomed out so it fits `content_scale` of the
    // square.
    let visible = WORLD / content_scale;
    let origin = (WORLD - visible) / 2.0;
    let mut camera = Camera2D::from_display_rect(Rect::new(origin, origin, visible, visible));
    camera.render_target = Some(target.clone());
    set_camera(&camera);

    if background {
        clear_background(BACKGROUND);
        // A soft glow behind the characters, built from many faint rings.
        for i in 0..40 {
            let r = 72.0 - i as f32 * 1.6;
            draw_circle(55.0, 50.0, r, Color { a: 0.018, ..GLOW });
        }
    } else {
        clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
    }

    let monster = vec2(72.0, 42.0);
    let girl = vec2(32.0, 58.0);
    draw_cyclops(monster, 17.0, 0.4, 0.0, girl);
    draw_girl(girl, 0.0, false, Some(monster));

    // The spell halfway to the monster, trailing sparkles back toward her.
    let hand = girl + (monster - girl).normalize() * 14.0;
    let orb = hand.lerp(monster, 0.42);
    for (i, t) in [0.2, 0.45, 0.7].iter().enumerate() {
        let spark = hand.lerp(orb, *t) + vec2(0.0, [1.5, -1.2, 0.8][i]);
        draw_circle(spark.x, spark.y, 1.0 + *t, [MAGENTA, PINK, WHITE][i]);
    }
    draw_circle(orb.x, orb.y, 7.5, Color::new(1.0, 0.4, 0.8, 0.35));
    draw_circle(orb.x, orb.y, 4.5, Color::new(1.0, 0.6, 0.9, 0.9));
    draw_circle(orb.x, orb.y, 2.2, WHITE);

    set_default_camera();
    // Make sure the drawing reaches the texture before reading it back.
    unsafe { get_internal_gl() }.flush();
    target.texture.get_texture_data()
}

/// Android's screen densities, with their scale from the 1x baseline.
const ANDROID_DENSITIES: [(&str, f32); 5] = [
    ("mdpi", 1.0),
    ("hdpi", 1.5),
    ("xhdpi", 2.0),
    ("xxhdpi", 3.0),
    ("xxxhdpi", 4.0),
];

/// Android's adaptive icon foreground is 108dp, of which the launcher may
/// crop everything outside the central 72dp; keep the content inside it.
const ANDROID_SAFE_ZONE: f32 = 66.0 / 108.0;

/// Writes all icon files under the project directory `root`.
pub fn render_all(root: &str) {
    let ios = format!("{root}/ios/Assets.xcassets/AppIcon.appiconset/AppIcon.png");
    render(1024, true, 1.0).export_png(&ios);
    println!("wrote {ios}");

    for (density, scale) in ANDROID_DENSITIES {
        // At 1x, legacy icons are 48px and adaptive foregrounds 108px.
        let dir = format!("{root}/android/res/mipmap-{density}");
        std::fs::create_dir_all(&dir).expect("icon folder should be creatable");
        let legacy = (48.0 * scale) as u32;
        render(legacy, true, 1.0).export_png(&format!("{dir}/ic_launcher.png"));
        let foreground = (108.0 * scale) as u32;
        render(foreground, false, ANDROID_SAFE_ZONE)
            .export_png(&format!("{dir}/ic_launcher_foreground.png"));
        println!("wrote {dir}");
    }
}
