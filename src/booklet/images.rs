//! The impure half of the booklet: drawing the pictures with the game's own
//! drawing code, and writing the file. Everything else is in `super`.

use image::ImageEncoder;
use macroquad::prelude::*;

use super::{Assets, Characters, data_uri, html};
use crate::fonts::{FREDOKA_SEMIBOLD, NUNITO_BOLD, NUNITO_REGULAR};
use crate::pairs::PAIRS;
use crate::pictures::draw_picture;
use crate::sprites::{draw_boss, draw_cyclops, draw_girl, draw_monster, draw_portal, draw_star};

/// The pair pictures are printed about 3 cm wide, so this is plenty.
const PICTURE_PIXELS: u32 = 360;
/// The characters are printed up to 10 cm wide.
const CHARACTER_PIXELS: u32 = 720;
/// Characters are drawn in a square of this many game units.
const WORLD: f32 = 100.0;

/// Draws with `draw` on a transparent square of `size` pixels, showing
/// `world` units, and returns it as a PNG.
fn render(size: u32, world: f32, draw: impl FnOnce()) -> Vec<u8> {
    let target = render_target(size, size);
    target.texture.set_filter(FilterMode::Linear);
    let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, world, world));
    camera.render_target = Some(target.clone());
    set_camera(&camera);
    clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
    draw();
    set_default_camera();
    // Make sure the drawing reaches the texture before reading it back.
    unsafe { get_internal_gl() }.flush();
    let image = target.texture.get_texture_data();
    // A render target is read back upside down, which `export_png` corrects
    // by flipping the rows too.
    let row = usize::from(image.width) * 4;
    let flipped: Vec<u8> = image.bytes.chunks(row).rev().flatten().copied().collect();
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(
            &flipped,
            u32::from(image.width),
            u32::from(image.height),
            image::ColorType::Rgba8,
        )
        .expect("a picture should encode as PNG");
    png
}

fn png_uri(png: &[u8]) -> String {
    data_uri("image/png", png)
}

fn character(draw: impl FnOnce()) -> String {
    png_uri(&render(CHARACTER_PIXELS, WORLD, draw))
}

/// Draws everything the booklet embeds.
fn assets() -> Assets {
    let pictures = PAIRS
        .iter()
        .map(|pair| {
            let size = PICTURE_PIXELS as f32;
            png_uri(&render(PICTURE_PIXELS, size, || {
                draw_picture(*pair, vec2(size / 2.0, size / 2.0), size, 0.0);
            }))
        })
        .collect();
    let centre = vec2(WORLD / 2.0, WORLD / 2.0);
    let characters = Characters {
        girl: character(|| draw_girl(vec2(50.0, 62.0), 0.0, false, None)),
        girl_casting: character(|| {
            draw_girl(vec2(34.0, 62.0), 0.0, false, Some(vec2(92.0, 40.0)));
        }),
        monster: character(|| draw_monster(centre, 30.0, 0.4, 0.0)),
        cyclops: character(|| draw_cyclops(centre, 30.0, 0.4, 0.0, vec2(50.0, 90.0))),
        boss: character(|| draw_boss(centre, 26.0, 0.4, 0.0)),
        portal: character(|| draw_portal(vec2(50.0, 55.0), 0.5)),
        star: character(|| draw_star(centre, 44.0, true)),
    };
    Assets {
        nunito_regular: data_uri("font/ttf", NUNITO_REGULAR),
        nunito_bold: data_uri("font/ttf", NUNITO_BOLD),
        fredoka: data_uri("font/ttf", FREDOKA_SEMIBOLD),
        pictures,
        characters,
    }
}

/// Writes the booklet to `path`.
pub fn write(path: &str) {
    std::fs::write(path, html(&assets())).expect("the booklet file should be writable");
}
