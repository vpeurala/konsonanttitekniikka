//! Music and sound effects, synthesized at startup.
//!
//! Every sound is rendered into samples, wrapped as an in-memory WAV file
//! and handed to macroquad, so the game needs no audio assets.

use std::f32::consts::TAU;

use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound, stop_sound};

const RATE: u32 = 44_100;

const MUSIC_VOLUME: f32 = 0.35;
const SFX_VOLUME: f32 = 0.6;

/// A sound effect the game asks to be played.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sfx {
    Type,
    Cast,
    Explode,
    Hurt,
    Wrong,
    LevelUp,
    GameOver,
}

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
}

impl Audio {
    pub async fn load() -> Self {
        Audio {
            music: load(&music()).await,
            music_on: true,
            playing: false,
            type_key: load(&type_key()).await,
            cast: load(&cast()).await,
            explode: load(&explode()).await,
            hurt: load(&hurt()).await,
            wrong: load(&wrong()).await,
            level_up: load(&level_up()).await,
            game_over: load(&game_over()).await,
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

    pub fn toggle_music(&mut self) {
        self.music_on = !self.music_on;
    }
}

async fn load(samples: &[f32]) -> Sound {
    load_sound_from_bytes(&wav(samples))
        .await
        .expect("synthesized WAV should always decode")
}

/// Encodes mono samples in -1..1 as a 16-bit PCM WAV file.
fn wav(samples: &[f32]) -> Vec<u8> {
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

fn sample_count(seconds: f32) -> usize {
    (seconds * RATE as f32) as usize
}

fn midi_to_freq(note: u8) -> f32 {
    440.0 * 2f32.powf((note as f32 - 69.0) / 12.0)
}

#[derive(Clone, Copy)]
enum Wave {
    Sine,
    Triangle,
    /// A pulse wave with the given duty cycle; 0.5 is a square.
    Pulse(f32),
}

impl Wave {
    /// The wave's value at `phase`, measured in cycles.
    fn at(self, phase: f32) -> f32 {
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
struct Noise(u32);

impl Noise {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0 as f32 / u32::MAX as f32 * 2.0 - 1.0
    }
}

/// A quick linear attack followed by an exponential decay.
fn envelope(t: f32, attack: f32, decay: f32) -> f32 {
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
fn limit(samples: &mut [f32], peak: f32) {
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

/// A slow, sad descent.
fn game_over() -> Vec<f32> {
    let notes = [(67, 0.3), (66, 0.3), (65, 0.3), (64, 0.9)];
    let mut out = melody(Wave::Triangle, &notes, 0.35);
    for s in &mut out {
        *s *= 0.6;
    }
    out
}

// ---------------------------------------------------------------------
// Music

const BPM: f32 = 140.0;
const BEAT: f32 = 60.0 / BPM;
const BARS: usize = 8;

/// The chord root (MIDI) of each bar: Am F C G Am F G E.
const BASS_ROOTS: [u8; BARS] = [45, 41, 48, 43, 45, 41, 43, 40];

/// The lead line as (MIDI note, beats); 4 beats per bar.
const LEAD: [(u8, f32); 35] = [
    // Am
    (76, 1.0),
    (72, 0.5),
    (69, 0.5),
    (71, 1.0),
    (72, 1.0),
    // F
    (69, 1.5),
    (67, 0.5),
    (65, 1.0),
    (69, 1.0),
    // C
    (67, 1.0),
    (72, 1.0),
    (76, 1.0),
    (74, 0.5),
    (72, 0.5),
    // G
    (74, 2.0),
    (71, 1.0),
    (67, 1.0),
    // Am
    (76, 0.5),
    (76, 0.5),
    (74, 0.5),
    (72, 0.5),
    (71, 1.0),
    (69, 1.0),
    // F
    (72, 1.0),
    (69, 1.0),
    (65, 1.0),
    (69, 1.0),
    // G
    (71, 1.0),
    (74, 1.0),
    (71, 0.5),
    (67, 0.5),
    (71, 1.0),
    // E
    (68, 1.0),
    (71, 1.0),
    (76, 2.0),
];

/// An upbeat, slightly spooky chiptune loop.
fn music() -> Vec<f32> {
    let total = sample_count(BARS as f32 * 4.0 * BEAT);
    let mut out = vec![0.0; total];

    // Adds a sound, wrapping its tail to the start so the loop is seamless.
    let mut add = |sound: &[f32], at_beat: f32, gain: f32| {
        let start = sample_count(at_beat * BEAT);
        for (i, s) in sound.iter().enumerate() {
            out[(start + i) % total] += s * gain;
        }
    };

    // Lead: a thin pulse wave, slightly shortened so notes articulate.
    let mut beat = 0.0;
    for &(note, beats) in &LEAD {
        let freq = midi_to_freq(note);
        let tone = sweep(Wave::Pulse(0.25), freq, freq, beats * BEAT * 0.9, 0.4);
        add(&tone, beat, 0.18);
        beat += beats;
    }

    // Bass: eighth notes bouncing between the root and its octave.
    for (bar, &root) in BASS_ROOTS.iter().enumerate() {
        for eighth in 0..8 {
            let note = if eighth % 2 == 0 { root } else { root + 12 };
            let freq = midi_to_freq(note);
            let tone = sweep(Wave::Triangle, freq, freq, BEAT * 0.45, 0.2);
            add(&tone, bar as f32 * 4.0 + eighth as f32 * 0.5, 0.35);
        }
    }

    // Drums: kick on 1 and 3, snare on 2 and 4, hi-hat on the off-beats.
    let kick = sweep(Wave::Sine, 110.0, 45.0, 0.18, 0.08);
    let mut noise = Noise(0x1234_5678);
    let snare: Vec<f32> = (0..sample_count(0.15))
        .map(|i| noise.next() * envelope(i as f32 / RATE as f32, 0.001, 0.05))
        .collect();
    let hat: Vec<f32> = (0..sample_count(0.04))
        .map(|i| noise.next() * envelope(i as f32 / RATE as f32, 0.001, 0.01))
        .collect();
    for beat in 0..BARS * 4 {
        let at = beat as f32;
        if beat.is_multiple_of(2) {
            add(&kick, at, 0.5);
        } else {
            add(&snare, at, 0.15);
        }
        add(&hat, at + 0.5, 0.08);
    }

    limit(&mut out, 0.9);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lead_fills_every_bar_exactly() {
        let beats: f32 = LEAD.iter().map(|(_, beats)| beats).sum();
        assert_eq!(beats, BARS as f32 * 4.0);
    }

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
            music(),
        ] {
            let peak = sound.iter().fold(0.0f32, |m, s| m.max(s.abs()));
            assert!(peak <= 1.0, "peak {peak}");
        }
    }
}
