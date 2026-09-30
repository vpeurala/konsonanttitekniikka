//! The background music: a chiptune piece in A minor, about a minute long,
//! rendered from the score below at startup and played as a loop.
//!
//! It runs through six sections: a quiet intro, the main theme, the theme
//! again with harmony and arpeggios, a climbing development, a breakdown
//! with bells and a snare roll, and a big chorus, which leads back to the
//! intro.

use std::f32::consts::TAU;

use crate::sound::audio::{Noise, RATE, Wave, envelope, limit, midi_to_freq, sample_count};

const BPM: f32 = 150.0;
const BEAT: f32 = 60.0 / BPM;
const STEPS_PER_BAR: usize = 16;
/// A sixteenth note, in beats.
const STEP: f32 = 0.25;

// ---------------------------------------------------------------------
// The score

/// The main theme: short-short-long figures that climb and fall back.
const THEME_A: &str = "
    A4:.5 C5:.5 E5:1 D5:.5 C5:.5 B4:.5 C5:.5
    A4:1.5 G4:.5 F4:.5 A4:.5 C5:1
    G4:.5 C5:.5 E5:1 G5:.5 F5:.5 E5:.5 D5:.5
    D5:2 B4:1 G4:1
    A4:.5 C5:.5 E5:1 D5:.5 C5:.5 B4:.5 C5:.5
    F5:1 E5:.5 D5:.5 C5:1 A4:1
    B4:.5 C5:.5 D5:1 E5:.5 D5:.5 C5:.5 B4:.5
    G#4:1 B4:1 E5:2";

/// The development: runs that reach higher every other bar.
const THEME_B: &str = "
    C5:.5 D5:.5 E5:.5 F5:.5 A5:1 G5:1
    F5:.5 E5:.5 D5:1 B4:1 G4:1
    G5:.5 E5:.5 B4:1 E5:.5 G5:.5 B5:1
    A5:2 E5:1 C5:1
    D5:.5 F5:.5 A5:1 G5:.5 F5:.5 E5:.5 D5:.5
    B4:.5 D5:.5 G5:1 F5:.5 E5:.5 D5:.5 B4:.5
    C5:.5 E5:.5 G5:.5 C6:.5 B5:1 G5:1
    G#5:1 E5:.5 B4:.5 G#4:1 B4:1";

/// The breakdown's slow bell notes.
const BELLS: &str = "
    A5:2 F5:2
    G#5:2 E5:2
    A5:2 C6:2
    B5:4";

/// The chorus: the theme's shape, an octave higher and broader.
const THEME_C: &str = "
    C6:1.5 A5:.5 F5:1 A5:1
    B5:1.5 G5:.5 D5:1 G5:1
    C6:1 B5:.5 A5:.5 E5:1 A5:1
    E6:2 D6:1 C6:1
    C6:1.5 A5:.5 F5:1 C6:1
    D6:1.5 B5:.5 G5:1 D6:1
    E6:1 D6:.5 C6:.5 G5:1 C6:1
    B5:1 G#5:1 E5:1 B4:1";

// Bass patterns, one character per sixteenth: R the chord's root, O its
// octave, 5 its fifth, . nothing new. A note lasts until the next one.
const BASS_PULSE: &str = "R.......R.......";
const BASS_DRIVE: &str = "R.O.R.O.R.O.R.O.";
const BASS_SYNC: &str = "R..R..O.R..R.5O.";
const BASS_HOLD: &str = "R...............";

/// One bar of drums, one character per sixteenth. Kick and snare hit on
/// x; hats are c (closed) or o (open); toms are 1 (high) to 3 (low).
#[derive(Clone, Copy)]
struct Drums {
    kick: &'static str,
    snare: &'static str,
    hat: &'static str,
    tom: &'static str,
    /// Whether the snare grows louder through the bar, as in a roll.
    crescendo: bool,
}

const fn drums(kick: &'static str, snare: &'static str, hat: &'static str) -> Drums {
    Drums {
        kick,
        snare,
        hat,
        tom: "................",
        crescendo: false,
    }
}

const SILENT: Drums = drums("................", "................", "................");
const HATS: Drums = drums("................", "................", "..c...c...c...c.");
const GROOVE_A: Drums = drums("x.....x...x.....", "....x.......x...", "..c...c...c...c.");
const GROOVE_A2: Drums = drums("x.....x...x...x.", "....x.......x...", "c.cc.cc.c.cc.cc.");
const GROOVE_B: Drums = drums("x..x..x...x..x..", "....x.......x..x", "c.c.c.c.c.c.c.o.");
const GROOVE_C: Drums = drums("x...x...x...x...", "....x.......x...", "c.o.c.o.c.o.c.o.");
const BREAK_HATS: Drums = drums("................", "................", "c.......c.......");
const TOM_FILL: Drums = Drums {
    kick: "x.....x.........",
    snare: "....x...........",
    hat: "c.c.c.c.........",
    tom: "........1.1.2.3.",
    crescendo: false,
};
const SNARE_ROLL: Drums = Drums {
    kick: "x...............",
    snare: "x.x.x.x.xxxxxxxx",
    hat: "................",
    tom: "................",
    crescendo: true,
};

struct Section {
    /// One chord per bar.
    chords: &'static str,
    /// The tune, or empty for none.
    lead: &'static str,
    /// Whether the tune is played on bells rather than the lead.
    bells: bool,
    /// Whether a second voice follows the tune a chord tone below.
    harmony: bool,
    /// Whether the tune echoes.
    echo: bool,
    /// The lowest note of the sixteenth-note arpeggio, or 0 for none.
    arp_from: u8,
    bass: &'static str,
    /// The drums of each bar, repeating if there are fewer than bars.
    drums: &'static [Drums],
    /// The drums of the last bar instead, if any.
    fill: Option<Drums>,
    /// Whether a cymbal crashes as the section starts.
    crash: bool,
    /// Whether rushing noise builds up through the last bar.
    riser: bool,
}

const SECTIONS: [Section; 6] = [
    // Intro: pad and bass, hats joining, building into the theme.
    Section {
        chords: "Am Am F G",
        lead: "",
        bells: false,
        harmony: false,
        echo: false,
        arp_from: 0,
        bass: BASS_PULSE,
        drums: &[SILENT, SILENT, HATS, HATS],
        fill: None,
        crash: true,
        riser: true,
    },
    // The theme.
    Section {
        chords: "Am F C G Am F E E",
        lead: THEME_A,
        bells: false,
        harmony: false,
        echo: false,
        arp_from: 0,
        bass: BASS_DRIVE,
        drums: &[GROOVE_A],
        fill: None,
        crash: true,
        riser: false,
    },
    // The theme again, fuller, ending in a tom fill.
    Section {
        chords: "Am F C G Am F E E",
        lead: THEME_A,
        bells: false,
        harmony: true,
        echo: false,
        arp_from: 60,
        bass: BASS_DRIVE,
        drums: &[GROOVE_A2],
        fill: Some(TOM_FILL),
        crash: false,
        riser: false,
    },
    // The development, in brighter chords.
    Section {
        chords: "F G Em Am Dm G C E",
        lead: THEME_B,
        bells: false,
        harmony: false,
        echo: false,
        arp_from: 60,
        bass: BASS_SYNC,
        drums: &[GROOVE_B],
        fill: None,
        crash: true,
        riser: false,
    },
    // The breakdown: bells over held chords, then a snare roll.
    Section {
        chords: "Dm E F E",
        lead: BELLS,
        bells: true,
        harmony: false,
        echo: true,
        arp_from: 72,
        bass: BASS_HOLD,
        drums: &[BREAK_HATS],
        fill: Some(SNARE_ROLL),
        crash: false,
        riser: true,
    },
    // The chorus, everything at once.
    Section {
        chords: "F G Am Am F G C E",
        lead: THEME_C,
        bells: false,
        harmony: true,
        echo: true,
        arp_from: 72,
        bass: BASS_DRIVE,
        drums: &[GROOVE_C],
        fill: Some(TOM_FILL),
        crash: true,
        riser: false,
    },
];

// ---------------------------------------------------------------------
// Reading the score

/// A MIDI note from its name, like "A4" (69), "G#4" or "Bb3".
fn note(name: &str) -> u8 {
    let mut chars = name.chars();
    let letter = chars.next().expect("a note has a letter");
    let mut pitch: i32 = match letter {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => panic!("unknown note {name}"),
    };
    let rest = chars.as_str();
    let octave = if let Some(octave) = rest.strip_prefix('#') {
        pitch += 1;
        octave
    } else if let Some(octave) = rest.strip_prefix('b') {
        pitch -= 1;
        octave
    } else {
        rest
    };
    let octave: i32 = octave.parse().expect("a note has an octave");
    (12 * (octave + 1) + pitch) as u8
}

/// A line of the score as (note, beats), where "r" is a rest.
fn line(score: &str) -> Vec<(Option<u8>, f32)> {
    score
        .split_whitespace()
        .map(|item| {
            let (name, beats) = item.split_once(':').expect("a note has a length");
            let beats = if beats.starts_with('.') {
                format!("0{beats}")
            } else {
                beats.to_owned()
            };
            let beats: f32 = beats.parse().expect("a length is a number");
            let pitch = (name != "r").then(|| note(name));
            (pitch, beats)
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Chord {
    /// The root's pitch class, 0 being C.
    root: u8,
    minor: bool,
}

impl Chord {
    fn named(name: &str) -> Self {
        let minor = name.ends_with('m');
        let root = note(&format!("{}0", name.trim_end_matches('m'))) % 12;
        Chord { root, minor }
    }

    fn has(self, midi: u8) -> bool {
        let third = if self.minor { 3 } else { 4 };
        let interval = (midi + 12 - self.root) % 12;
        interval == 0 || interval == third || interval == 7
    }

    /// The first `count` notes of the chord from `low` upwards.
    fn notes_from(self, low: u8, count: usize) -> Vec<u8> {
        (low..).filter(|&n| self.has(n)).take(count).collect()
    }

    /// The root in the bass register, from E2 up.
    fn bass(self) -> u8 {
        self.notes_from(40, 3)
            .into_iter()
            .find(|&n| n % 12 == self.root)
            .unwrap_or(40)
    }

    /// The chord tone a third or more below `melody`, for a harmony line.
    fn below(self, melody: u8) -> u8 {
        (melody.saturating_sub(9)..=melody.saturating_sub(3))
            .rev()
            .find(|&n| self.has(n))
            .unwrap_or(melody.saturating_sub(12))
    }
}

// ---------------------------------------------------------------------
// Instruments

/// How an instrument sounds: its wave and loudness over a note.
#[derive(Clone, Copy)]
struct Voice {
    wave: Wave,
    attack: f32,
    decay: f32,
    /// The level held after the decay, as a fraction of the peak.
    sustain: f32,
    release: f32,
    /// How far the pitch wavers on held notes, in semitones.
    vibrato: f32,
}

const LEAD: Voice = Voice {
    wave: Wave::Pulse(0.25),
    attack: 0.005,
    decay: 0.25,
    sustain: 0.6,
    release: 0.06,
    vibrato: 0.25,
};
const HARMONY: Voice = Voice {
    wave: Wave::Pulse(0.5),
    vibrato: 0.15,
    ..LEAD
};
const BELL: Voice = Voice {
    wave: Wave::Sine,
    attack: 0.002,
    decay: 0.5,
    sustain: 0.0,
    release: 0.4,
    vibrato: 0.0,
};
const ARP: Voice = Voice {
    wave: Wave::Pulse(0.125),
    attack: 0.002,
    decay: 0.07,
    sustain: 0.0,
    release: 0.02,
    vibrato: 0.0,
};
const BASS: Voice = Voice {
    wave: Wave::Triangle,
    attack: 0.003,
    decay: 0.3,
    sustain: 0.7,
    release: 0.03,
    vibrato: 0.0,
};
/// A buzz doubling the bass, so it is heard on small speakers.
const BASS_BUZZ: Voice = Voice {
    wave: Wave::Pulse(0.5),
    ..BASS
};
const PAD: Voice = Voice {
    wave: Wave::Triangle,
    attack: 0.4,
    decay: 1.0,
    sustain: 0.8,
    release: 0.5,
    vibrato: 0.0,
};

/// One note of `voice` at `freq`, held for `seconds` and then released.
fn play(voice: Voice, freq: f32, seconds: f32) -> Vec<f32> {
    let n = sample_count(seconds + voice.release);
    let mut phase = 0.0;
    (0..n)
        .map(|i| {
            let t = i as f32 / RATE as f32;
            let level = if t < voice.attack {
                t / voice.attack
            } else {
                voice.sustain + (1.0 - voice.sustain) * (-(t - voice.attack) / voice.decay).exp()
            };
            let released = if t < seconds {
                1.0
            } else {
                (1.0 - (t - seconds) / voice.release).max(0.0)
            };
            // Vibrato fades in on notes held for a moment.
            let depth = voice.vibrato * ((t - 0.15) / 0.2).clamp(0.0, 1.0);
            let bend = 2f32.powf(depth * (t * 5.5 * TAU).sin() / 12.0);
            phase += freq * bend / RATE as f32;
            voice.wave.at(phase) * level * released
        })
        .collect()
}

fn kick() -> Vec<f32> {
    let n = sample_count(0.25);
    let mut phase = 0.0;
    (0..n)
        .map(|i| {
            let t = i as f32 / RATE as f32;
            let freq = 45.0 + 110.0 * (-t * 30.0).exp();
            phase += freq / RATE as f32;
            Wave::Sine.at(phase) * envelope(t, 0.001, 0.1)
        })
        .collect()
}

/// White noise with the lows taken out, decaying at `decay`.
fn hiss(seed: u32, seconds: f32, decay: f32, brightness: f32) -> Vec<f32> {
    let mut noise = Noise(seed);
    let mut low = 0.0;
    (0..sample_count(seconds))
        .map(|i| {
            let t = i as f32 / RATE as f32;
            let white = noise.next();
            low += (white - low) * brightness;
            (white - low) * envelope(t, 0.001, decay)
        })
        .collect()
}

fn snare() -> Vec<f32> {
    let mut out = hiss(0x5eed_0001, 0.2, 0.06, 0.2);
    let n = out.len();
    let mut phase = 0.0;
    for (i, s) in out.iter_mut().enumerate().take(n) {
        let t = i as f32 / RATE as f32;
        phase += (180.0 - 40.0 * t / 0.2) / RATE as f32;
        *s += 0.6 * Wave::Triangle.at(phase) * envelope(t, 0.001, 0.04);
    }
    out
}

fn tom(freq: f32) -> Vec<f32> {
    let n = sample_count(0.35);
    let mut phase = 0.0;
    (0..n)
        .map(|i| {
            let t = i as f32 / RATE as f32;
            phase += freq * (1.0 - 0.4 * t / 0.35) / RATE as f32;
            Wave::Sine.at(phase) * envelope(t, 0.002, 0.13)
        })
        .collect()
}

/// Noise that grows louder and brighter over `seconds`.
fn riser(seconds: f32) -> Vec<f32> {
    let mut noise = Noise(0x5eed_0005);
    let mut low = 0.0;
    let n = sample_count(seconds);
    (0..n)
        .map(|i| {
            let progress = i as f32 / n as f32;
            low += (noise.next() - low) * (0.02 + 0.5 * progress * progress);
            low * progress * progress
        })
        .collect()
}

// ---------------------------------------------------------------------
// Putting it together

/// How many bars a section lasts.
fn bars(section: &Section) -> usize {
    section.chords.split_whitespace().count()
}

/// The piece being mixed: sounds are added at beats, and anything that
/// runs past the end wraps around to the start, so the loop is seamless.
struct Mix {
    out: Vec<f32>,
}

impl Mix {
    fn new(total_bars: usize) -> Self {
        Mix {
            out: vec![0.0; sample_count(total_bars as f32 * 4.0 * BEAT)],
        }
    }

    /// Adds a sound at `beat`, at volume `gain`.
    fn add(&mut self, sound: &[f32], beat: f32, gain: f32) {
        let total = self.out.len();
        let start = sample_count(beat * BEAT);
        for (i, s) in sound.iter().enumerate() {
            self.out[(start + i) % total] += s * gain;
        }
    }
}

/// The drum sounds, made once and played wherever a pattern says.
struct Kit {
    kick: Vec<f32>,
    snare: Vec<f32>,
    closed_hat: Vec<f32>,
    open_hat: Vec<f32>,
    crash: Vec<f32>,
    toms: [Vec<f32>; 3],
}

impl Kit {
    fn new() -> Self {
        Kit {
            kick: kick(),
            snare: snare(),
            closed_hat: hiss(0x5eed_0002, 0.05, 0.012, 0.6),
            open_hat: hiss(0x5eed_0003, 0.35, 0.12, 0.6),
            crash: hiss(0x5eed_0004, 1.8, 0.7, 0.4),
            toms: [tom(220.0), tom(165.0), tom(120.0)],
        }
    }
}

/// The whole piece, one loop long.
pub fn music() -> Vec<f32> {
    let total_bars: usize = SECTIONS.iter().map(bars).sum();
    let mut mix = Mix::new(total_bars);
    let kit = Kit::new();

    let mut section_start = 0.0;
    for section in &SECTIONS {
        let chords: Vec<Chord> = section
            .chords
            .split_whitespace()
            .map(Chord::named)
            .collect();
        let bar_count = chords.len();

        play_tune(&mut mix, section, &chords, section_start);
        for (bar, chord) in chords.iter().enumerate() {
            let bar_start = section_start + bar as f32 * 4.0;
            play_pad(&mut mix, chord, bar_start);
            play_arpeggio(&mut mix, section, chord, bar_start);
            play_bass(&mut mix, section, chord, bar_start);
            let drums = match section.fill {
                Some(fill) if bar + 1 == bar_count => fill,
                _ => section.drums[bar % section.drums.len()],
            };
            play_drums(&mut mix, &kit, &drums, bar_start);
        }

        if section.crash {
            mix.add(&kit.crash, section_start, 0.12);
        }
        if section.riser {
            let last_bar = section_start + (bar_count - 1) as f32 * 4.0;
            mix.add(&riser(4.0 * BEAT), last_bar, 0.12);
        }
        section_start += bar_count as f32 * 4.0;
    }

    limit(&mut mix.out, 0.9);
    mix.out
}

/// The tune, and its harmony and echoes.
fn play_tune(mix: &mut Mix, section: &Section, chords: &[Chord], section_start: f32) {
    let chord_at = |beat: f32| chords[((beat / 4.0) as usize).min(chords.len() - 1)];
    let mut beat = 0.0;
    for (pitch, beats) in line(section.lead) {
        if let Some(pitch) = pitch {
            let at = section_start + beat;
            let seconds = beats * BEAT * 0.92;
            let (voice, gain) = if section.bells {
                (BELL, 0.3)
            } else {
                (LEAD, 0.2)
            };
            let tone = play(voice, midi_to_freq(pitch), seconds);
            mix.add(&tone, at, gain);
            if section.echo {
                mix.add(&tone, at + 0.75, gain * 0.3);
                mix.add(&tone, at + 1.5, gain * 0.12);
            }
            if section.harmony {
                let below = chord_at(beat).below(pitch);
                mix.add(&play(HARMONY, midi_to_freq(below), seconds), at, 0.09);
            }
        }
        beat += beats;
    }
}

/// A soft pad of the chord, each note doubled slightly out of tune for a
/// fuller sound.
fn play_pad(mix: &mut Mix, chord: &Chord, bar_start: f32) {
    for pitch in chord.notes_from(57, 3) {
        for detune in [0.997, 1.003] {
            let tone = play(PAD, midi_to_freq(pitch) * detune, 4.0 * BEAT);
            mix.add(&tone, bar_start, 0.035);
        }
    }
}

/// The arpeggio, up and down the chord in sixteenths.
fn play_arpeggio(mix: &mut Mix, section: &Section, chord: &Chord, bar_start: f32) {
    if section.arp_from == 0 {
        return;
    }
    let notes = chord.notes_from(section.arp_from, 4);
    for step in 0..STEPS_PER_BAR {
        let pitch = notes[[0, 1, 2, 3, 2, 1][step % 6]];
        let tone = play(ARP, midi_to_freq(pitch), STEP * BEAT);
        mix.add(&tone, bar_start + step as f32 * STEP, 0.06);
    }
}

/// The bass, each note lasting until the next.
fn play_bass(mix: &mut Mix, section: &Section, chord: &Chord, bar_start: f32) {
    let pattern: Vec<char> = section.bass.chars().collect();
    let hits: Vec<usize> = (0..STEPS_PER_BAR).filter(|&s| pattern[s] != '.').collect();
    for (i, &step) in hits.iter().enumerate() {
        let next = hits.get(i + 1).copied().unwrap_or(STEPS_PER_BAR);
        let root = chord.bass();
        let pitch = match pattern[step] {
            'O' => root + 12,
            '5' => root + 7,
            _ => root,
        };
        let seconds = (next - step) as f32 * STEP * BEAT * 0.9;
        let at = bar_start + step as f32 * STEP;
        mix.add(&play(BASS, midi_to_freq(pitch), seconds), at, 0.3);
        mix.add(&play(BASS_BUZZ, midi_to_freq(pitch), seconds), at, 0.04);
    }
}

/// One bar of drums, from the patterns of each kit piece.
fn play_drums(mix: &mut Mix, kit: &Kit, drums: &Drums, bar_start: f32) {
    for step in 0..STEPS_PER_BAR {
        let at = bar_start + step as f32 * STEP;
        let hit = |pattern: &str| pattern.as_bytes()[step];
        if hit(drums.kick) == b'x' {
            mix.add(&kit.kick, at, 0.55);
        }
        if hit(drums.snare) == b'x' {
            let swell = if drums.crescendo {
                0.3 + 0.7 * step as f32 / STEPS_PER_BAR as f32
            } else {
                1.0
            };
            mix.add(&kit.snare, at, 0.3 * swell);
        }
        match hit(drums.hat) {
            b'c' => mix.add(&kit.closed_hat, at, 0.07),
            b'o' => mix.add(&kit.open_hat, at, 0.06),
            _ => {}
        }
        if let Some(tom) = (hit(drums.tom) as char)
            .to_digit(10)
            .and_then(|d| kit.toms.get(d as usize - 1))
        {
            mix.add(tom, at, 0.35);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_names_become_midi_notes() {
        assert_eq!(note("A4"), 69);
        assert_eq!(note("C4"), 60);
        assert_eq!(note("G#4"), 68);
        assert_eq!(note("Bb3"), 58);
        assert_eq!(note("E6"), 88);
    }

    #[test]
    fn a_line_reads_notes_rests_and_lengths() {
        assert_eq!(
            line("A4:.5 r:1 E5:1.5"),
            vec![(Some(69), 0.5), (None, 1.0), (Some(76), 1.5)]
        );
    }

    #[test]
    fn chords_know_their_notes() {
        let am = Chord::named("Am");
        assert_eq!(am.notes_from(60, 4), vec![60, 64, 69, 72]);
        let e = Chord::named("E");
        assert!(e.has(68), "E major has G#");
        assert_eq!(e.bass(), 40);
        assert_eq!(Chord::named("Dm").bass(), 50);
        // The harmony takes a chord tone at least a third below.
        assert_eq!(am.below(76), 72);
        assert_eq!(am.below(69), 64);
    }

    #[test]
    fn every_tune_fills_its_section_exactly() {
        for section in &SECTIONS {
            if section.lead.is_empty() {
                continue;
            }
            let beats: f32 = line(section.lead).iter().map(|(_, beats)| beats).sum();
            assert_eq!(beats, bars(section) as f32 * 4.0, "{}", section.chords);
        }
    }

    #[test]
    fn every_pattern_is_one_bar_long() {
        for section in &SECTIONS {
            assert_eq!(section.bass.len(), STEPS_PER_BAR);
            for drums in section.drums.iter().chain(&section.fill) {
                for pattern in [drums.kick, drums.snare, drums.hat, drums.tom] {
                    assert_eq!(pattern.len(), STEPS_PER_BAR, "{pattern}");
                }
            }
        }
    }

    #[test]
    fn the_piece_lasts_about_a_minute_and_stays_in_range() {
        let music = music();
        let seconds = music.len() as f32 / RATE as f32;
        assert!((50.0..80.0).contains(&seconds), "{seconds} s");
        let peak = music.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        assert!(peak <= 1.0, "peak {peak}");
    }
}
