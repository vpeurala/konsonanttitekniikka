//! Music and sound effects, synthesized at startup.
//!
//! Every sound is rendered into samples, wrapped as an in-memory WAV file
//! and handed to macroquad, so the game needs no audio assets.

use std::f32::consts::TAU;

use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound, stop_sound};

pub const RATE: u32 = 44_100;

const MUSIC_VOLUME: f32 = 0.35;
const SFX_VOLUME: f32 = 0.6;

pub use lukuloitsu_core::sfx::Sfx;

pub struct Audio {
    music: Sound,
    music_on: bool,
    playing: bool,
    type_key: Sound,
    cast: Sound,
    explode: Sound,
    hurt: Sound,
    wrong: Sound,
    level_up: Sound,
    game_over: Sound,
    boss: Sound,
    thunder: Sound,
    badge: Sound,
}

impl Audio {
    pub async fn load() -> Self {
        Audio {
            music: load(&crate::sound::music::music()).await,
            music_on: true,
            playing: false,
            type_key: load(&type_key()).await,
            cast: load(&cast()).await,
            explode: load(&explode()).await,
            hurt: load(&hurt()).await,
            wrong: load(&wrong()).await,
            level_up: load(&level_up()).await,
            game_over: load(&game_over()).await,
            boss: load(&boss()).await,
            thunder: load(&thunder()).await,
            badge: load(&badge()).await,
        }
    }

    pub fn play(&self, sfx: Sfx) {
        let (sound, volume) = match sfx {
            Sfx::Type => (&self.type_key, 0.3),
            Sfx::Cast => (&self.cast, 0.7),
            Sfx::Explode => (&self.explode, 0.8),
            Sfx::Hurt => (&self.hurt, 0.9),
            Sfx::Wrong => (&self.wrong, 0.8),
            Sfx::LevelUp => (&self.level_up, 0.9),
            Sfx::GameOver => (&self.game_over, 0.9),
            Sfx::Boss => (&self.boss, 1.0),
            Sfx::Thunder => (&self.thunder, 1.0),
            Sfx::Badge => (&self.badge, 0.9),
        };
        play_sound(
            sound,
            PlaySoundParams {
                looped: false,
                volume: volume * SFX_VOLUME,
            },
        );
    }

    /// Starts or stops the music loop to match `wanted` and the mute
    /// setting.
    pub fn set_music(&mut self, wanted: bool) {
        let play = wanted && self.music_on;
        if play && !self.playing {
            play_sound(
                &self.music,
                PlaySoundParams {
                    looped: true,
                    volume: MUSIC_VOLUME,
                },
            );
        } else if !play && self.playing {
            stop_sound(&self.music);
        }
        self.playing = play;
    }

    pub fn set_music_on(&mut self, on: bool) {
        self.music_on = on;
    }
}

async fn load(samples: &[f32]) -> Sound {
    load_sound_from_bytes(&wav(samples))
        .await
        .expect("synthesized WAV should always decode")
}

/// Encodes mono samples in -1..1 as a 16-bit PCM WAV file.
pub fn wav(samples: &[f32]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // fmt chunk size
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * 2).to_le_bytes()); // bytes per second
    out.extend_from_slice(&2u16.to_le_bytes()); // bytes per frame
    out.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

// ---------------------------------------------------------------------
// Building blocks

pub fn sample_count(seconds: f32) -> usize {
    (seconds * RATE as f32) as usize
}

pub fn midi_to_freq(note: u8) -> f32 {
    440.0 * 2f32.powf((note as f32 - 69.0) / 12.0)
}

#[derive(Clone, Copy)]
pub enum Wave {
    Sine,
    Triangle,
    /// A pulse wave with the given duty cycle; 0.5 is a square.
    Pulse(f32),
}

impl Wave {
    /// The wave's value at `phase`, measured in cycles.
    pub fn at(self, phase: f32) -> f32 {
        let p = phase.fract();
        match self {
            Wave::Sine => (p * TAU).sin(),
            Wave::Triangle => 4.0 * (p - 0.5).abs() - 1.0,
            Wave::Pulse(duty) => {
                if p < duty {
                    1.0
                } else {
                    -1.0
                }
            }
        }
    }
}

/// A deterministic white-noise source (xorshift).
pub struct Noise(pub u32);

impl Noise {
    pub fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0 as f32 / u32::MAX as f32 * 2.0 - 1.0
    }
}

/// A quick linear attack followed by an exponential decay.
pub fn envelope(t: f32, attack: f32, decay: f32) -> f32 {
    if t < attack {
        t / attack
    } else {
        (-(t - attack) / decay).exp()
    }
}

/// A tone whose frequency glides from `from` to `to` over `seconds`.
fn sweep(wave: Wave, from: f32, to: f32, seconds: f32, decay: f32) -> Vec<f32> {
    let n = sample_count(seconds);
    let mut phase = 0.0;
    (0..n)
        .map(|i| {
            let t = i as f32 / RATE as f32;
            let freq = from + (to - from) * (t / seconds);
            phase += freq / RATE as f32;
            wave.at(phase) * envelope(t, 0.005, decay)
        })
        .collect()
}

/// Adds `other` into `into`, starting at `offset` seconds.
fn mix(into: &mut Vec<f32>, other: &[f32], offset: f32, gain: f32) {
    let start = sample_count(offset);
    if into.len() < start + other.len() {
        into.resize(start + other.len(), 0.0);
    }
    for (i, s) in other.iter().enumerate() {
        into[start + i] += s * gain;
    }
}

/// Scales `samples` down so the loudest one is at most `peak`.
pub fn limit(samples: &mut [f32], peak: f32) {
    let loudest = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    if loudest > peak {
        for s in samples {
            *s *= peak / loudest;
        }
    }
}

/// Plays `notes` (MIDI note, seconds) one after another.
fn melody(wave: Wave, notes: &[(u8, f32)], decay: f32) -> Vec<f32> {
    let mut out = Vec::new();
    let mut at = 0.0;
    for &(note, seconds) in notes {
        let freq = midi_to_freq(note);
        mix(&mut out, &sweep(wave, freq, freq, seconds, decay), at, 1.0);
        at += seconds;
    }
    out
}

// ---------------------------------------------------------------------
// Sound effects

/// A soft, short blip for each typed character.
fn type_key() -> Vec<f32> {
    sweep(Wave::Sine, 1200.0, 900.0, 0.04, 0.012)
}

/// A rising, shimmering whoosh.
fn cast() -> Vec<f32> {
    let n = sample_count(0.3);
    let mut phase = 0.0;
    let mut shimmer = 0.0;
    (0..n)
        .map(|i| {
            let t = i as f32 / RATE as f32;
            let freq = 500.0 + 1500.0 * (t / 0.3).powf(0.7);
            let vibrato = 1.0 + 0.03 * (t * 40.0 * TAU).sin();
            phase += freq * vibrato / RATE as f32;
            shimmer += freq * 2.01 / RATE as f32;
            let tone = Wave::Sine.at(phase) + 0.4 * Wave::Triangle.at(shimmer);
            tone * envelope(t, 0.02, 0.15) * 0.6
        })
        .collect()
}

/// A noisy burst over a low thump, darkening as it fades.
fn explode() -> Vec<f32> {
    let n = sample_count(0.6);
    let mut noise = Noise(0x2545_f491);
    let mut filtered = 0.0;
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / RATE as f32;
            // A one-pole low-pass that closes over time.
            let cutoff = 0.5 * (-t * 6.0).exp() + 0.03;
            filtered += (noise.next() - filtered) * cutoff;
            filtered * envelope(t, 0.002, 0.18) * 1.4
        })
        .collect();
    mix(
        &mut out,
        &sweep(Wave::Sine, 120.0, 40.0, 0.35, 0.12),
        0.0,
        0.9,
    );
    limit(&mut out, 0.9);
    out
}

/// A falling, buzzy "ouch".
fn hurt() -> Vec<f32> {
    let mut out = sweep(Wave::Pulse(0.5), 400.0, 90.0, 0.35, 0.15);
    for s in &mut out {
        *s *= 0.5;
    }
    out
}

/// Two low, flat "bonk" notes.
fn wrong() -> Vec<f32> {
    let mut out = melody(Wave::Pulse(0.3), &[(50, 0.12), (46, 0.25)], 0.1);
    for s in &mut out {
        *s *= 0.4;
    }
    out
}

/// A bright rising arpeggio.
fn level_up() -> Vec<f32> {
    let notes = [(72, 0.09), (76, 0.09), (79, 0.09), (84, 0.09), (88, 0.4)];
    let mut out = melody(Wave::Pulse(0.25), &notes, 0.15);
    let echo = out.clone();
    mix(&mut out, &echo, 0.12, 0.3);
    for s in &mut out {
        *s *= 0.35;
    }
    out
}

/// A short, cheerful chime: two quick notes and a held one.
fn badge() -> Vec<f32> {
    let notes = [(79, 0.1), (84, 0.1), (91, 0.5)];
    let mut out = melody(Wave::Triangle, &notes, 0.2);
    for s in &mut out {
        *s *= 0.5;
    }
    out
}

/// A slow, sad descent.
fn game_over() -> Vec<f32> {
    let notes = [(67, 0.3), (66, 0.3), (65, 0.3), (64, 0.9)];
    let mut out = melody(Wave::Triangle, &notes, 0.35);
    for s in &mut out {
        *s *= 0.6;
    }
    out
}

/// A deep, wobbling growl that drops in pitch.
fn boss() -> Vec<f32> {
    let seconds = 1.0;
    let n = sample_count(seconds);
    let mut noise = Noise(0x0bad_f00d);
    let mut phase = 0.0;
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / RATE as f32;
            let wobble = 1.0 + 0.08 * (t * 18.0 * TAU).sin();
            let freq = (110.0 - 50.0 * t / seconds) * wobble;
            phase += freq / RATE as f32;
            // Two detuned pulses and a little breath make it rough.
            let tone = Wave::Pulse(0.3).at(phase) + 0.6 * Wave::Pulse(0.5).at(phase * 1.01);
            (tone + 0.3 * noise.next()) * envelope(t, 0.08, 0.45)
        })
        .collect();
    limit(&mut out, 0.8);
    out
}

/// A sharp crack followed by a long, low rumble.
fn thunder() -> Vec<f32> {
    let n = sample_count(2.0);
    let mut noise = Noise(0x7e57_ab1e);
    let (mut bright, mut dark) = (0.0, 0.0);
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / RATE as f32;
            let white = noise.next();
            bright += (white - bright) * 0.6;
            dark += (white - dark) * 0.02;
            let crack = bright * envelope(t, 0.001, 0.08);
            // The rumble swells a moment after the crack and rolls on.
            let roll = 1.0 + 0.5 * (t * 7.0).sin();
            let rumble = dark * envelope(t, 0.15, 0.7) * roll * 6.0;
            crack + rumble
        })
        .collect();
    limit(&mut out, 0.9);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header_matches_the_data() {
        let bytes = wav(&[0.0, 0.5, -0.5]);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(bytes.len(), 44 + 3 * 2);
        assert_eq!(u32::from_le_bytes(bytes[40..44].try_into().unwrap()), 6);
    }

    #[test]
    fn sounds_stay_within_full_scale() {
        for sound in [
            type_key(),
            cast(),
            explode(),
            hurt(),
            wrong(),
            level_up(),
            game_over(),
            boss(),
            thunder(),
            badge(),
        ] {
            let peak = sound.iter().fold(0.0f32, |m, s| m.max(s.abs()));
            assert!(peak <= 1.0, "peak {peak}");
        }
    }
}
