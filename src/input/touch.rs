//! Reading touches, and deciding whether to show touch controls. What the
//! controls do with them is `lukuloitsu_core::touch`, and drawing them is
//! `gfx::touch_render`. On a computer, the mouse stands in for a finger.

use macroquad::input::utils::{register_input_subscriber, repeat_all_miniquad_input};
use macroquad::miniquad::{self, EventHandler};
use macroquad::prelude::*;

use lukuloitsu_core::input::{Phase, Pointer};

/// Whether to show touch controls: always on phones, and on a computer
/// when the `LUKULOITSU_TOUCH` environment variable is set.
pub fn enabled() -> bool {
    #[cfg(target_arch = "wasm32")]
    if crate::platform::web::touch_screen() {
        return true;
    }
    cfg!(any(target_os = "ios", target_os = "android"))
        || std::env::var_os("LUKULOITSU_TOUCH").is_some()
}

const MOUSE_ID: u64 = u64::MAX;

/// The window's touch phase as the app knows it.
fn phase_of(phase: miniquad::TouchPhase) -> Phase {
    match phase {
        miniquad::TouchPhase::Started => Phase::Started,
        miniquad::TouchPhase::Moved => Phase::Moved,
        miniquad::TouchPhase::Ended => Phase::Ended,
        miniquad::TouchPhase::Cancelled => Phase::Cancelled,
    }
}

/// Reads touches as the ordered stream of events they arrive in, so a
/// quick tap that starts and ends within one frame still counts.
pub struct TouchReader {
    subscriber: usize,
}

struct TouchCollector(Vec<(Phase, u64, Vec2)>);

impl EventHandler for TouchCollector {
    fn update(&mut self) {}
    fn draw(&mut self) {}

    fn touch_event(&mut self, phase: miniquad::TouchPhase, id: u64, x: f32, y: f32) {
        self.0.push((phase_of(phase), id, vec2(x, y)));
    }
}

impl TouchReader {
    pub fn new() -> Self {
        TouchReader {
            subscriber: register_input_subscriber(),
        }
    }

    /// The touches since the previous call, in order, and the mouse acting
    /// as one more finger, at positions in screen points. Must be called
    /// every frame.
    pub fn read(&self) -> Vec<Pointer> {
        let mut collector = TouchCollector(Vec::new());
        repeat_all_miniquad_input(&mut collector, self.subscriber);
        // Touches arrive in physical pixels.
        let dpi = miniquad::window::dpi_scale();
        let mut pointers: Vec<Pointer> = collector
            .0
            .into_iter()
            .map(|(phase, id, pos)| Pointer {
                id,
                pos: pos / dpi,
                phase,
            })
            .collect();
        pointers.extend(mouse_pointer());
        pointers
    }
}

/// The mouse as a finger, for trying touch controls on a computer. A
/// click quick enough to press and release within one frame, as a
/// browser's synthetic clicks are, still counts as both.
fn mouse_pointer() -> Vec<Pointer> {
    let pressed = is_mouse_button_pressed(MouseButton::Left);
    let released = is_mouse_button_released(MouseButton::Left);
    let phases = match (pressed, released) {
        (true, true) => vec![Phase::Started, Phase::Ended],
        (true, false) => vec![Phase::Started],
        (false, true) => vec![Phase::Ended],
        _ if is_mouse_button_down(MouseButton::Left) => vec![Phase::Moved],
        _ => Vec::new(),
    };
    let pos = Vec2::from(mouse_position());
    phases
        .into_iter()
        .map(|phase| Pointer {
            id: MOUSE_ID,
            pos,
            phase,
        })
        .collect()
}
