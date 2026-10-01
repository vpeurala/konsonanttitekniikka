# Developing Lukuloitsu

A guide for whoever works on this code next: a new contributor, or you after
a long break. It explains the domain, how the code is organised and why, how
to change things safely, and the traps already fallen into.

Read the [README](../README.md) first for what the game is and how to build
it. [AGENTS.md](../AGENTS.md) is a denser reference written for AI coding
agents; this document is the human version, with the reasoning.

Contents

1. [The domain](#1-the-domain)
2. [Ground rules](#2-ground-rules)
3. [The architecture in one page](#3-the-architecture-in-one-page)
4. [Tour of the code](#4-tour-of-the-code)
5. [Design decisions and their reasons](#5-design-decisions-and-their-reasons)
6. [How things work](#6-how-things-work)
7. [Recipes: how to change X](#7-recipes-how-to-change-x)
8. [Testing and verifying](#8-testing-and-verifying)
9. [Platforms, releases, operations](#9-platforms-releases-operations)
10. [Pitfalls](#10-pitfalls)
11. [Style](#11-style)

---

## 1. The domain

**The consonant technique** (*konsonanttitekniikka*) is the Finnish version of
the Major system, a mnemonic for remembering numbers. Every digit stands for
one consonant:

| 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
|---|---|---|---|---|---|---|---|---|---|
| H | J | K | L | M | P | R | S | T | V |

Vowels are filler. A number becomes a word: 22 is KEKO (K and K); 44 is MUUMIO;
77 is SUSI. Longer numbers are read **two digits at a time**, an odd digit
left over at the end being read as a single-digit word: 201 is KOHO JÄÄ.

The game teaches **110 fixed pairs**: the digits 0–9 and the numbers 00–99,
each a number and a word, each with a picture.

Vocabulary you will meet in the code (the game's own text is Finnish):

| Term | Meaning |
|---|---|
| **Pair** (`Pair`, `PairId`) | One number–word combination. `PairId` is its index in `PAIRS`. |
| **Question** | What a monster shows: one pair, or a *long number* made of several pairs. |
| **Monster / enemy** | Carries a question toward the heroine. Shows a number (you type the word) or a word (you type the number). |
| **Boss** | Ends a level; takes several hits, one per number in its queue. From level 22 some of its numbers are long. |
| **Slot** | Where typed characters go: the number slot takes digits, the word slot takes letters. Each answers the monsters showing the *other* kind. |
| **Level** | Level 1 has the digits; every level adds 5 more pairs, so all 110 are met by level 21. |
| **Checkpoint** | A game can start from every fifth level reached (`levels.rs`). |
| **Memory** | Spaced-repetition state: per pair, a difficulty and when it was last seen. Decides what appears and when. |
| **Lesson** | Something that happened to a pair (answered in *n* seconds, or missed) which memory learns from. |
| **Badge** (*kunniamerkki*) | An achievement; never taken back once earned. |
| **Progress** | What is known about the player in one mode: memory, best level, stars, streak, stats, badges. A **Profile** holds one for the easy mode and one for hardcore mode (no hints), plus the sound setting. |
| **Harjoittele / Edistyminen / Kunniamerkit / Pelaa** | Practice / progress map / badges / play. |

The players are **children and teens** (the game was made for the author's
daughter). Keep the difficulty curve gentle, the content friendly, and
prefer privacy-friendly choices: no cookies, no tracking of individuals.

## 2. Ground rules

These come from the project's history; break them only on purpose.

- **Talk in English; the game stays in Finnish.** Menus, messages, banners
  and help texts are Finnish. Code, comments, docs and commit messages are
  English.
- **`core/src/pairs.rs` is authoritative.** Never "fix" a word that looks odd.
  Change a pair only when the owner asks (44 became MUUMIO that way). Every
  pair must have a picture in `src/gfx/pictures.rs`.
- **Licenses.** Code: MIT OR Apache-2.0. Fonts: SIL Open Font License. Word
  list: CC BY-SA 4.0. The name and icon are not licensed. Don't add anything
  under another license, and keep the README's License section true when
  adding assets or dependencies.
- **Commit and push only when asked.** Check `git status` for stray files
  before committing (a rendered `music.wav` once slipped in). Commit messages
  have a short imperative subject and a body that explains *why*.
- **CI must stay green:** `cargo test`, `cargo clippy --all-targets -- -D
  warnings`, `cargo fmt --check`; and for anything touching the web build,
  `cargo clippy --target wasm32-unknown-unknown -- -D warnings`.

## 3. The architecture in one page

The code is a **functional core inside an imperative shell**, and the split is
enforced by the compiler: it is two crates.

```
                       ┌──────────────────────────────────────────────┐
  window, clock,       │  SHELL  (root crate, uses macroquad)         │
  keyboard, touch ───▶ │  Inputs::read()  ──▶  Frame                  │
                       │                          │                   │
                       │  ┌───────────────────────▼────────────────┐  │
                       │  │ CORE (crates `core/`, only depends on  │  │
                       │  │ glam: no graphics, sound, clock, I/O)  │  │
                       │  │                                        │  │
                       │  │  App::update(&Frame) ──▶ Vec<Effect>   │  │
                       │  │     screens, game simulation, memory,  │  │
                       │  │     badges, save format, layout        │  │
                       │  └───────────────────────┬────────────────┘  │
                       │                          │                   │
                       │  perform(effects):  play sound / write save  │
                       │                     / count statistic / quit │
                       │  app_render::draw(&App)  ── reads state only │
                       └──────────────────────────────────────────────┘
```

- **`core/`** (`lukuloitsu-core`) holds all the rules and state machines:
  pairs, curriculum, memory, badges, the save format, the whole game
  simulation, every screen's behaviour and layout, and `App` itself. It
  depends only on `glam` (vectors). If you feel you need macroquad in `core/`,
  the code belongs in the shell instead.
- **The root crate** is the shell. It turns the window into a `Frame`,
  performs the `Effect`s the app asks for, and draws whatever state the core
  exposes. Drawing only *reads* state.

Each frame, `main.rs` does exactly this:

1. `Inputs::read()` builds a `Frame` (elapsed time, wall-clock time, keys,
   typed characters, touches, mouse wheel, "was away").
2. `App::update(&Frame)` returns a list of `Effect`s.
3. `perform(effects)` plays sounds, writes the save file, sends a statistic,
   or quits.
4. `app_render::draw(&app)` draws.

Everything between 1 and 3 is a plain function of its inputs, so it is tested
by running made-up frames through it, with no window, sound card or clock.

## 4. Tour of the code

### Workspace

```
core/            lukuloitsu-core: the pure crate (see below)
src/             the shell: reading the window, drawing, sound, disk
vendor/miniquad  patched copy of macroquad's platform layer (vendor/README.md lists the patches)
web/             index.html, JavaScript bridge, privacy page, share image
android/ ios/ macos/   packaging metadata (Info.plist, icons, ...)
assets/          fonts only; every other picture is drawn by code
scripts/         build/package helpers (web, Android, iOS sim, macOS app, Linux, booklet)
docs/            this document; screenshots for the README
.github/         CI (rust.yml), releases (release.yml), audit.yml, dependabot
```

### `core/src` (the pure crate)

| Module | What it is |
|---|---|
| `pairs` | The 110 pairs, the digit→consonant table, which characters count as answers. |
| `curriculum` | Which pairs each level introduces (5 per level, seeded, so identical in every game); `unlocked_pairs(level)`. |
| `long_numbers` | Reads long numbers two digits at a time, leading zeros allowed; when and how long they appear. |
| `memory` | Spaced repetition (see [6.4](#64-spaced-repetition)). `Memory`, `Lesson`, `Happened`. |
| `progress` | `Profile` (settings and a `Progress` per mode): the one copy of everything known about the player. |
| `save` | `Profile` ⇄ text. Only the format; where the file lives is the shell's business. |
| `badges` | The table `BADGES`, `Stats`, `Standing`, `award`. |
| `levels` | The checkpoint rule. |
| `game/` | The simulation: `Game`, `World`, `Player`, `Vitals`, `Stage`, `Display`, rules, spawning, answer matching, `Scene` (read-only view for drawing). |
| `rng` | The only source of randomness, in separate streams. |
| `effects`, `obstacles`, `portals` | Sparks/lightning; obstacle layout and steering; portal positions. All pure state. |
| `app` (+ `app/persistence`, `app/events`) | Which screen is showing, saving cadence, badge awarding, what game events mean for counters and statistics. Produces `Effect`s. |
| `screens/` | `title`, `levels`, `practice`, `progress_map`, `badge_screen`: each screen's state, `update`, layout and hit-testing. |
| `touch` | On-screen keypad and joystick logic and layout. |
| `toast` | The "new badge" notice queue. |
| `input`, `key`, `geometry`, `view` | `Frame`, `KeyCode`, `Phase`, `Pointer`; typed `Key`; `Rect`; `View` (screen ⇄ virtual coordinates). |
| `sfx`, `analytics`, `arena`, `color` | Small shared vocabulary: sound names, statistic events (`path()`/`title()` in Finnish), arena size and heroine geometry, `Color` for sparks. |

### `src` (the shell)

| Path | What it is |
|---|---|
| `main.rs` | Entry point; `play()` runs the loop, `perform()` does effects. |
| `cli.rs` | Pure command-line parsing: play, or `--render-icons/-music/-booklet`. |
| `app_render.rs` | Draws whichever screen `App` says is showing, then the badge toast. |
| `input/` | `frame.rs` (`Inputs::read`, the one place that asks window and clock), `keyboard.rs` (physical keys → Finnish characters), `touch.rs` (touch/mouse events, `enabled()`), `lifecycle.rs` (detects "was away"). |
| `screens/` | *Drawing only* for each screen; the behaviour is in `core/src/screens`. |
| `gfx/` | `fonts`, `view` (camera), `pictures` (the 110 word pictures), `sprites` (heroine, monsters, boss, obstacles), `badge_art` (medals and emblems), `game_render`, `touch_render`, `icon`. |
| `sound/` | `audio.rs` (all sounds synthesised at startup), `music.rs` (a minute-long loop written as note names). |
| `platform/` | `save.rs` (file paths per OS, atomic write), `analytics.rs` (send a count), `web.rs` (JS bridge, wasm only). |
| `booklet/` | The printable Finnish PDF booklet generator (`--render-booklet`). |

### Where do the tests live?

Almost all of them are in `core/` (`app/tests.rs`, `game/tests.rs`, and in
each module). The shell has few: the command line, the touch reader, the
booklet, and colour helpers. That is by design: whatever is worth testing was
moved to where it can be tested.

## 5. Design decisions and their reasons

**Functional core, imperative shell.** Side effects live in one thin outer
layer. This makes behaviour deterministic and testable without a screen, and
it makes refactoring safe: you can restructure a screen and know from tests
that it still behaves the same. Rules of thumb:

- Don't call `is_key_pressed`, `get_frame_time`, `date::now`, or measure text
  inside `core/`. Add a field to `Frame` or `game::Input` and let the shell
  fill it.
- Functions return what they want done (`Outputs`, `Effect`, `PracticeOutcome`)
  instead of doing it or writing into a hidden buffer.
- Drawing reads state and never changes it.
- Layout that both hit-testing and drawing need lives in pure functions in
  core (`button_rect`, `cell_rect`, `medal_center`, `legend_layout`,
  `content_rect`), so it is tested and never drifts from what is drawn.

**Two crates make purity enforceable.** `core` cannot import macroquad. The
compiler, not discipline, keeps it pure.

**The game only *reports* what she learns.** `Game::update(&Input, &Memory)`
returns `Lesson`s; `App` learns them into the live memory. Inside a frame the
game still sees its own lessons (a copy-on-write view), so what spawns next
is decided as if memory had already learned, and results are unchanged. If you
change the order in which random numbers are drawn or memory is read, you
change every game (see determinism below).

**Values that change by returning a new value.** `Player`, `Vitals` and
`Stage` are small `Copy` values whose transitions return a new one
(`vitals = vitals.hurt()`). `World` owns the monsters, spells, portals and
obstacles and reports what happened (`Hit`, `Contact`, `SpellEvent`) without
knowing about energy, sound or memory; `game/play.rs` turns those reports into
state changes, sounds and sparks. `&mut self` is still used where copying
would be silly (`World`, the screens); the point is *who decides*, not the
syntax.

**Deterministic simulation.**
- **Fixed timestep.** The game advances in steps of 1/120 s (`Timestep`),
  however long the frames are; a frame counts as at most 0.25 s. Typed keys
  wait in `Pending` for the next step, so a slow frame can't skip a hit and a
  fast one can't lose a key.
- **Seeded random streams.** `rng.rs` derives everything from one seed, with
  separate streams (`Curriculum`, `Portals`, `Gameplay`, `Effects`,
  `Obstacles`) so drawing more sparks can never change which pairs a level
  unlocks. Every game introduces the same pairs in the same order. Random
  draws are lent as `&mut Rng`; *the order of draws is part of the behaviour*.
- **Label widths come from a fixed table** of Nunito Bold letter widths
  (`game/metrics.rs`), not from measuring text, so the simulation never
  depends on the font machinery.

**One copy of the player's state.** `Progress` owns `Memory`; `save.rs` only
converts it to and from text. There used to be a second copy of the pairs that
had to be synced; don't reintroduce one.

**Everything drawn is code.** No image assets besides fonts: pictures, sprites,
medals, icons and the booklet's images are all drawn procedurally. That keeps
the repo small, makes art diff-able, and lets the booklet reuse the game's own
drawing code.

**All sounds are synthesised at startup.** No audio files.

**No accounts, no tracking.** Saves live on the device (local storage on the
web). The web version sends anonymous counts to GoatCounter; the apps send
nothing. If you change what is stored or counted, keep three things in
agreement: the privacy page `web/tietosuoja.html`, the note at the end of the
title screen (`PRIVACY_NOTE`), and the README's paragraph on statistics.

## 6. How things work

### 6.1 A frame, in detail

`Frame` (core/src/input.rs): `dt`, `now` (seconds since 1970), `away`,
`screen` size, `typed` keys, `pressed`/`down` physical keys, `wheel`, and
`touches` (screen coordinates). Menus get pointers mapped into virtual units
with `frame.pointers(content_rect)`.

`App::update`:
1. advances menu-animation time and the toast queue;
2. Tab toggles sound (still called `music_on` in the save);
3. the current screen handles the frame and returns a `Step` (the next screen,
   if leaving, plus `Effect`s);
4. newly earned badges are awarded (toast, `Sfx::Badge`, statistic, save);
5. if sound is off, all `Effect::Play` are dropped.

The screens are updated apart from a `Context` holding everything else
(progress, touch controls, toasts), so a screen can be borrowed mutably while
the rest of the app is used. `Effect`s are: `Play(Sfx)`, `Save(String)`,
`Count(analytics::Event)`, `Quit`.

### 6.2 The game

`Game::update(&Input, &Memory) -> Outputs` is the only way in. `Outputs`
carries `sfx`, `events` (`GameEvent`: started, over, level completed, answered,
monster defeated, flawless level) and `lessons`.

Typing: digits go to the number slot, letters to the word slot; each answers
monsters showing the other kind. A dead-end slot is shown red at first; typing
*past* it costs energy (leaving room to fix a typo with Backspace). Backspace
empties both slots. Matching lives in `game/answer.rs` (`resolve_input`).

Levels need at most 40 points before their boss. A boss's numbers form a
queue; each right answer wounds it and shows the next number. From level 22
(the first level after all pairs are met) bosses carry long numbers: shown
split into pairs and made of well-known pairs for the first five such levels;
three digits at first, one more every three levels, up to six.

The game pauses itself when the app was away (`Frame::away`): the window was
minimised, or a gap between frames was so long that the app clearly wasn't
running (iOS gives no event).

### 6.3 Screens

Each screen in `core/src/screens/` is a struct with `update(&mut self, &Frame,
&[Pointer])` returning an action (`TitleAction`, `LevelAction`, ...), plus pure
layout functions. Its counterpart in `src/screens/` has a `draw` function that
reads the screen through accessors. When adding UI, put state, behaviour and
layout in core, and drawing in the shell.

Touch controls (`core/src/touch.rs`) are a full Finnish QWERTY keypad split
into two panels beside the arena (unused keys greyed out), a floating
joystick, and pause/sound buttons. Touches are read as an ordered *event
stream*, because a quick tap can begin and end within one frame; the mouse
counts as a finger.

Coordinates: the arena is a fixed **800×600 virtual** area, scaled to fit the
window (`View`). In touch mode the content also spans the keypad panels at
negative x and beyond 800.

### 6.4 Spaced repetition

`memory.rs`. Per pair: a **difficulty** from 0 (learned) to 1, `last_seen`,
`times_seen`, and a `streak`.

- Answer quality is judged by speed: within 3 s is best, 12 s or more (or not
  at all) is worst; an answer read from a hint counts for at most half.
- Difficulty moves toward `1 - quality` with a learning rate of 0.4.
- A pair is *learned* below difficulty 0.25.
- Review interval runs from 5 hours (unknown) to 6 days (learned) by
  difficulty. A streak of good answers (quick, no hint; anything else resets
  it) stretches a learned pair's interval by half again per answer past five,
  up to 3 weeks.
- Spawn/flash-card weight grows with difficulty and with how overdue the pair
  is (`Memory::weight`). New pairs are favoured (`NEW_PAIR_SHARE`).

### 6.5 Saving and compatibility

`Profile` ⇄ plain text, one fact per line, in `core/src/save.rs`. The settings come first, then the easy mode's progress, then a `mode hardcore` line and the hardcore mode's (older files have no such line, so they are the easy mode's):

```
lukuloitsu-save 1
music on
hardcore off
best-level 4
stars 1 3
streak 20356 5
stat monsters 120
badge monsters-100 20357
pair 22 0.3512 1790000000 7 3
mode hardcore
best-level 2
```

Rules: a damaged line is skipped, not fatal; absurd numbers are clamped or
dropped (tests feed thousands of garbage files); **saves from before a field
existed must keep loading** (the `pair` streak column was added later);
badge ids are stored, so **never rename a badge id**. The shell decides where
the file goes (`platform/save.rs`) and writes a temporary file then renames it,
so an interrupted save never leaves half a file. On the web, local storage.

`Persistence` (in `core/src/app/`) decides *when* to save: at most every 5 seconds
while playing a game or practising, and right after a level, badge, sound toggle
or leaving a screen.

### 6.6 Badges

All in one table, `BADGES`: id, Finnish name, category, tier, `Requirement`.
`Stats` are lifetime counters kept in the save (monsters, bosses, long numbers,
quick answers, practice cards, flawless levels, best combo). Everything else a
requirement asks about is worked out into a `Standing` from the progress. The
game reports counts as `GameEvent`s; `App` tallies them and awards badges at the
end of each frame. Players from before badges existed get theirs quietly at
start-up. Medal art is in `src/gfx/badge_art.rs`: `emblem_of` maps *every*
badge id to a *different* emblem, and a test fails if two share one or one is
missing.

### 6.7 Sound

The Tab key is the "sound" switch: it silences music and effects (the setting
is still named `music_on`). `Sfx` names come from core; `sound/audio.rs`
synthesises them at start-up; `music.rs` is a score in note names, a
minute-long loop of six sections, mixed by a `Mix` with a drum `Kit`.

### 6.8 The booklet

`--render-booklet FILE.html` writes one self-contained HTML file (fonts and PNG
pictures embedded), 16 A4 pages in Finnish, drawn with the game's own code.
`scripts/booklet.sh` prints it to PDF with headless Chrome; CI attaches the PDF
to each release. Every number, word, consonant and level in the text is *looked
up* from `pairs`, `curriculum` and `long_numbers`, never typed in, so the
booklet cannot disagree with the game. Page height is 296.6 mm, not 297, or
Chrome sometimes inserts blank pages. A page whose content grows past its
height is silently cut off: look at the PDF after changing text.

## 7. Recipes: how to change X

**Add a badge.** Add a line to `BADGES` (`core/src/badges.rs`) and raise the
array length; add an `Emblem` and its draw function in `src/gfx/badge_art.rs`
and map the id in `emblem_of`. Tests check ids, order and that nothing is met
from the start. The README says how many badges exist; update it.

**Change a pair.** Only when asked. Edit `core/src/pairs.rs` and make sure the
picture in `src/gfx/pictures.rs` still matches. The booklet updates itself.

**Add a screen.**
1. Core: `core/src/screens/foo.rs` with state, `update`, an action enum and
   layout functions, plus tests; register it in `screens/mod.rs`.
2. Add a variant to `app::Screen` and handle it in `Context::update_screen`
   (and `wants_music`, if relevant).
3. Shell: `src/screens/foo.rs` with `draw`; add an arm in `app_render.rs`
   (`content_of` and `draw`).
4. If it should be counted, add an `analytics::Event`.

**Add input the app needs.** Add a field to `Frame` (or a variant to
`KeyCode`), fill it in `Inputs::read` (`src/input/frame.rs`), use it in core.
Never reach for the window from core.

**Add a sound effect.** Add a variant to `core/src/sfx.rs`, synthesise it in
`src/sound/audio.rs`, and emit it in `Outputs`/`Effect::Play`.

**Add a statistic.** Add a variant to `core/src/analytics.rs` (paths and titles
in Finnish), produce it as `Effect::Count`, and update the privacy page and the
README/title note (see section 5).

**Change gameplay numbers.** They are in `game/rules.rs`. Remember that changing
what is drawn from `Rng`, or in what order, changes every game's spawns.

**Add a field to the save.** Add it to `Progress`, write and read it in
`save.rs`, make its absence load as the default, and add a "saves from before X
still load" test.

**Change a web-facing JS function.** `src/platform/web.rs` and
`web/lukuloitsu.js` are two halves of one interface: change both and bump
`PAGE_VERSION` in `web.rs` and `version` in the JavaScript, which must match.

## 8. Testing and verifying

```sh
cargo test                                               # both crates
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo clippy --target wasm32-unknown-unknown -- -D warnings   # web changes
```

CI (`.github/workflows/rust.yml`) checks format, lints (the web build too) and
tests on Ubuntu (it installs ALSA, X11 and OpenGL libraries first); measures
coverage with `cargo llvm-cov` and sends it to Codecov (files that only draw
are left out: see the regex in the workflow, and update it when you add
draw-only files); and builds the Windows executable, the macOS app and the
Linux package the way a release does. `.github/workflows/audit.yml` runs
`cargo audit` on `Cargo.lock` changes and weekly; Dependabot opens update PRs.
Two advisories are known and only warn (ttf-parser unmaintained, macroquad
soundness notes); a real vulnerability fails the job.

**Testing style.** Drive the app or a screen with made-up `Frame`s
(`core/src/app/tests.rs` is the model), and the game with made-up `Input`s
(`core/src/game/tests.rs`). Save-file tests read thousands of garbage files.
Layout is tested through the pure layout functions.

**Checking drawing changes without a screen test.** When refactoring drawing
code, prove nothing changed:
- *Screenshots.* Temporarily make `main` draw a screen and call
  `get_screen_data().export_png(path)`, then restore `src/main.rs`. Keep
  screenshots and scratch files outside the repository. Compare before/after
  renders pixel by pixel; anti-aliased text can differ by a few pixels between
  any two runs, so compare against that noise floor.
- *Audio.* `cargo run --release -- --render-music FILE.wav` before and after;
  the output is deterministic, so `cmp` the files.
- *Booklet.* Render the HTML before and after and `cmp`, then read the PDF page
  by page (`pdftoppm` from poppler helps).

**Stop the game's audio after every test.** The music loops, and a forgotten
instance keeps playing unseen. Run `xcrun simctl shutdown all` after simulator
tests, close browser tabs showing the game and stop the preview server, and
don't leave the desktop build running.

**Browser preview gotcha.** A hidden preview pane gets frames only in bursts;
the game reads a long gap as the player being away and pauses itself,
discarding that frame's keys, so keyboard tests there are unreliable. Practice
mode doesn't auto-pause. The browser tool's `type` action sends no physical key
codes, which the game reads; use `key`, one key at a time.

## 9. Platforms, releases, operations

**Web** (lukuloitsu.fi). `scripts/web.sh` builds into `target/web` with
link-time optimisation via environment variables (about 13% smaller). Netlify
runs `scripts/netlify-build.sh` (see `netlify.toml`) on every push to `main`;
`rust-toolchain.toml` is there for Netlify's build machine. `.cargo/config.toml`
passes `--allow-undefined` to the wasm linker because the JavaScript functions
only exist once the page loads the game. `web/mq_js_bundle.js` is macroquad's
bundle with two fixes (listed at its top): the key where Finnish keyboards have
Ä reports code 39, and a variable declaration strict mode requires; keep them
if you ever replace the bundle. `web/index.html` suspends all sound while the
page is hidden (a hidden page gets no frames, so the game can't stop its own
music), shows "Ladataan…" until ready, and asks you to turn an upright phone.
Statistics: GoatCounter, site code `lukuloitsu`; it doesn't count localhost.
Events are paths like `peli-alkoi/taso-1`, `taso-lapaisty/5`, `harjoittelu`,
`merkki/<badge id>` (`merkki/ankara/<badge id>` in hardcore mode). The domain is at DNSimple and points to Netlify.

**Android.** `scripts/android.sh` builds with `cargo quad-apk`, installs and
starts it. Needs Java 8, the SDK with build-tools 30.0.3 and platform 33, and
NDK 27.2.12479018. Install `cargo-quad-apk` with
`scripts/install-cargo-quad-apk.sh` (pins a working commit, applies a patch).
App ID `fi.lukuloitsu.lukuloitsu`; it must equal the crate name (Android loads
the library by the last part of the package name). Saves:
`/data/data/fi.lukuloitsu.lukuloitsu/files/save.txt`. The test phone is reached
with `adb -d` (picks the USB device even when emulators are listed).

**iOS.** `scripts/ios-sim.sh` builds for the simulator and launches it.
`.cargo/config.toml` records iOS SDK 18.0 in the binary because iOS 27 won't
launch apps built against its own SDK without UIKit's scene lifecycle, which
miniquad lacks. It is a workaround and wouldn't pass App Store review.

**macOS.** `build.rs` embeds `macos/Info.plist` so the menu bar says
"Lukuloitsu"; after renaming the project folder run `cargo clean -p lukuloitsu`.
Saves: `~/Library/Application Support/Lukuloitsu/save.txt`.
`scripts/macos-app.sh` makes a universal `Lukuloitsu.app` and zips it. It is
signed ad hoc (the `lipo` step discards the linker's signature and Apple silicon
refuses unsigned programs); without a paid Developer ID it can't be notarised,
so downloaders right-click and choose Open.

**Linux.** `cargo run --release` (see the README for libraries).
`scripts/linux-package.sh` makes the tarball. CI builds on Ubuntu 22.04, so it
runs on glibc 2.35+; don't move the runner without changing the release notes.
Saves: `~/.lukuloitsu/save.txt`.

**Windows.** `cargo run --release` on Windows, or from a Mac
`rustup target add x86_64-pc-windows-gnu`, `brew install mingw-w64` and
`cargo build --release --target x86_64-pc-windows-gnu`. The executable is
unsigned, so Chrome and SmartScreen warn (`.github/release-notes.md` tells
players what to click); free signing needs every component under an
OSI-approved license, which the CC BY-SA word list isn't. On the one Windows
machine tried, neither the executable nor the web version made a sound; it is
unresolved and probably not the game's fault.

**Touch controls on a computer:** `LUKULOITSU_TOUCH=1 cargo run`.

**Releases.** A release is a version tag, and every part of the project carries
the same version. Bump `version` in `Cargo.toml` (updates `Cargo.lock`),
`version_code` in `Cargo.toml` for Android, and `CFBundleShortVersionString` and
`CFBundleVersion` in `ios/Info.plist`. The code is
`major * 10000 + minor * 100 + patch` (0.4.0 is 400). Commit, then
`git tag -a X.Y.Z -m "Lukuloitsu X.Y"` and push the tag (no `v` prefix).
`release.yml` builds the Windows executable, attaches a zip with the licenses,
uses `.github/release-notes.md` as the text, then adds the macOS app, the Linux
tarball and the booklet. It can also be started by hand for an existing tag.

**`vendor/miniquad`** is a patched copy of macroquad's platform layer; the
changes are listed in `vendor/README.md`. Check it before upgrading macroquad.

## 10. Pitfalls

Already found the hard way:

- **Changing the order of random draws** or the order memory is read changes
  every game's behaviour. Tests will notice; don't "fix" the tests.
- **Renaming a badge id or dropping a save field** breaks existing players'
  saves. Only add.
- **A page of the booklet that grows** is cut off silently. View the PDF.
- **Reading a render target back gives the picture upside down**;
  `booklet/images.rs` flips it, like `export_png` does.
- **Frame gaps look like "away".** Long gaps between frames pause the game and
  discard that frame's keys.
- **Chrome adds a blank page** after each booklet page at exactly 297 mm height.
- **The Ä key**: the physical key where Finnish keyboards have Ä must map
  correctly on the web (bundle fix above); keyboard input is by physical
  position (`input/keyboard.rs`), so Ä/Ö work on any layout.
- **Windows sound** is unresolved (see above).
- **Forgotten instances keep playing music.** Stop the game after testing.
- **Stray files in commits.** A rendered `music.wav` once slipped in. Don't add
  ignore rules unless asked; just check `git status`.
- **Don't add cookies, trackers or anything identifying**; keep the privacy
  page, title note and README in step.

## 11. Style

- Match the surrounding code: small doc comments that say *why*, descriptive
  names, comment density like the neighbours.
- Prefer explicit imports over globs (test modules may use `super::*`).
- Keep functions short; a function of 60+ lines is usually several steps that
  deserve names. When splitting drawing code, verify output as in section 8.
- New pure logic gets a test next to it. New layout gets a pure function.
- Workspace lints (`Cargo.toml`) go beyond clippy's defaults
  (`format_push_string`, `unused_self`, `float_cmp`); keep both crates clean.
- Commit subject: short and imperative. Body: what changed and why.
