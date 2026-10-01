# Notes for AI agents

Read [README.md](README.md) first for what the game is, and
[docs/DEVELOPING.md](docs/DEVELOPING.md) for the longer human-oriented guide. This file is what an
agent working on the code needs to know on top of that: the rules, how the
code fits together, and pitfalls already found the hard way.

## Rules

- **Talk to the user in English. Everything in the game stays in Finnish**:
  menus, messages, banners, help texts.
- **The pair list in `core/src/pairs.rs` is authoritative.** Never "fix" a word,
  even if it looks odd; change a pair only when the user asks (44 became
  MUUMIO that way). Every pair has a picture in `src/pictures.rs`.
- **Licenses.** The code is MIT OR Apache-2.0 (`LICENSE-MIT`,
  `LICENSE-APACHE`); the fonts are under the SIL Open Font License; the word
  list is CC BY-SA 4.0 (`LICENSE-CC-BY-SA`; releases up to 0.4.4 said
  CC BY-NC-SA), and the name and icon are not licensed (the README
  explains all of it). Don't put anything under a different license, and keep
  the README's License section true when adding assets or dependencies.
- **The players are children and teens.** Keep the learning curve gentle and
  the content friendly; prefer privacy-friendly choices (no cookies, no
  tracking of individuals).
- **Commit only when asked, and push only when asked**; the user usually asks
  for them separately. Before committing, check `git status` for stray files
  (a rendered `music.wav` once slipped into a commit). Don't add ignore rules
  the user hasn't asked for.
- Match the existing style: small doc comments that say why, descriptive
  names, commit messages with a short imperative subject and a body that
  explains the change.

## Checking your work

- `cargo test`, `cargo clippy --all-targets -- -D warnings` and
  `cargo fmt --check`; CI runs all three, so keep them clean.
- For web changes also run
  `cargo clippy --target wasm32-unknown-unknown -- -D warnings`.
- GitHub Actions (`.github/workflows/rust.yml`) checks formatting, lints (also
  the web build) and tests every push on Ubuntu; it installs ALSA, X11 and
  OpenGL libraries first. Two more jobs there measure test coverage with
  `cargo llvm-cov` and send it to Codecov (files that only draw are left
  out, see the comment in the workflow; `cargo llvm-cov --summary-only`
  shows it locally), and build the Windows executable. Two more build the macOS app and the Linux
  package the way a release does.
- To see a screen, temporarily make `main` draw it and call
  `get_screen_data().export_png(path)`, then restore `src/main.rs`. Put
  screenshots, rendered music and other scratch files outside the repository.
- **Stop the game's audio after every test.** The music loops, and a forgotten
  instance keeps playing unseen: run `xcrun simctl shutdown all` after
  simulator tests, close browser tabs showing the game and stop the preview
  server, and don't leave the desktop build running.
- The booklet's pages are 296.6 mm tall, not 297: at exactly 297 mm Chrome
  sometimes adds a blank page after each. Reading a render target back gives
  the picture upside down; `booklet/images.rs` flips it, like `export_png`
  does. A page whose content grows past its height is silently cut off, so
  look at the PDF after changing text.
- A hidden browser preview pane gets frames only in bursts. The game reads a
  long gap between frames as the player being away and pauses itself,
  discarding that frame's keys, so keyboard tests there are unreliable.
  Practice mode doesn't auto-pause. The browser tool's `type` action sends
  no physical key codes, which the game reads; use its `key` action, one key
  at a time.
- `.github/workflows/audit.yml` runs `cargo audit` on Cargo.lock changes and
  weekly; Dependabot (`.github/dependabot.yml`) opens update pull requests.
  Two advisories are known and only warn (ttf-parser is unmaintained,
  macroquad has soundness notes); a real vulnerability fails the job.
- The Android phone (a moto g15) is reached with `adb -d`, which picks the
  USB device even when emulators are listed.

## How the code fits together

**Two crates.** `core/` (`lukuloitsu-core`) holds the rules and state
machines: `pairs`, `curriculum`, `long_numbers`, `memory`, `badges`, `levels`
(the checkpoint rule), `progress` (everything known about the player,
`Memory` included), `save` (its file format, not where it lives), `rng`,
`effects`, `obstacles`, `portals`, the whole `game` simulation and the whole
`app`: the screens as state machines (`screens/`), the touch controls' logic
(`touch`), the notices (`toast`), the input types (`input`: `Frame`,
`KeyCode`, `Pointer`), `geometry::Rect` and `view::View`. It depends
only on `glam` and has no graphics, sound, clock or input, so the compiler
enforces the purity described below: if you need macroquad in `core/`, the
code belongs in the shell instead. It hands the shell small plain types to
work with: `Sfx`, `Key`, `Color` (only for sparks), `Tone` (how a message
feels; the shell picks the colour) and `game::Scene`, a read-only picture of a
game that `src/gfx/game_render.rs` draws. The root crate is the shell: `main.rs`,
drawing, audio, reading the window, saving to disk. `main.rs`
re-exports the core modules by name, so shell code says `crate::pairs::...`.
A plain `cargo test` or `cargo clippy` covers both crates (`default-members`).

The shell is grouped by what its parts are for (bare file names in the list
below are inside these folders): `input/` (`frame`, `keyboard`, `lifecycle`,
`touch`: reading the window into a `Frame`), `screens/` (`title`, `levels`,
`practice`, `progress`, `badge_screen`, `toast`: drawing only, the screens
themselves are in core), `gfx/` (`fonts`, `view`, `pictures`, `sprites`,
`badge_art`, `icon`, `game_render`, `touch_render`), `sound/` (`audio`,
`music`) and `platform/` (`save`, `analytics`, `web`), plus `app_render`,
`booklet`, `cli` and `main`.

**Pure core, impure shell.** Side effects live only in the outermost layer.
`main.rs` is the shell: each frame it reads input into a `Frame`
(`input/frame.rs`, the one place that asks the window and the clock), passes
it to `App::update` (`core/src/app.rs`), performs the `Effect`s that come back
(sounds, saving, statistics, quitting), and draws the app (`app_render.rs`). Everything between is
a pure state machine that takes input as arguments and reports what it
wants done as return values, so it is tested by running made-up frames
through it (see `core/src/app/tests.rs` and `core/src/game/tests.rs`). Keep it that
way: don't call `is_key_pressed`, `get_frame_time`, `date::now` or measure
text inside the core; add a field to `Frame` or `game::Input` instead.
Drawing (the shell's `draw` functions, `gfx/game_render.rs`, sprites, pictures) reads state
and never changes it. Layout maths that drawing needs is kept in pure
functions (`slot_rects`, `cell_rect`, `legend_layout`, `Enemy::keep_on_screen`)
so it can be tested without a screen; do the same for new layouts.

- `main.rs`: the shell, described above. `cli.rs` parses the command line (play,
  or one of the `--render-*` commands) into a `Command`. `App::start` is how
  the app begins: it counts the day played and returns the launch's save.
- `core/src/app.rs`: which screen is showing (title, level choice, game, practice,
  progress) and the `Effect`s; each screen has its own method returning a
  `Step` (where to go next, what to do outside); the screens are worked on
  apart from the rest of the app (`Context`), so no placeholder is needed while
  one runs. `app/persistence.rs` holds
  the live `Progress` (its `Memory` included). The game and practice only
  read the memory and report what she learned as `Lesson`s (`Outputs::lessons`),
  which `App` learns into it; `app/events.rs` says
  what game events mean for the counters and the statistics. `Effect::Save`
  carries the save file's text; `analytics::Event` is what gets counted, and
  the Finnish paths and titles are made there.
- `core/src/input.rs`: `Frame`, one frame of input, with `KeyCode` and touch
  `Phase` of its own; `input/frame.rs` in the shell has `Inputs::read`, which
  makes it from the window.
- `core/src/game/`: the game itself. Monsters show a number or a word; typed digits
  go to the number slot and letters to the word slot, each answering the
  monsters showing the other kind. Backspace empties both slots.
  `Game::update(&Input, &Memory) -> Outputs` is the only way in; `Outputs`
  carry sound effects and `GameEvent`s (started, level completed, game over)
  for saving and analytics, and every step inside returns the `Outputs` it
  caused instead of writing to a hidden buffer. The game advances in fixed
  steps of 1/120 s (`Timestep`), however long the frames are, so it plays out
  the same at any frame rate; a frame counts as at most 0.25 s, and typing
  waits in `Pending` for the next step, so a slow frame can't skip past a hit
  and a fast one can't lose a key.
  Its state is a few small parts behind their own boundaries: `Player`,
  `Vitals` (energy, combo, score) and `Stage` are `Copy` values whose every
  change returns a new value (`vitals = vitals.hurt()`), and `World`
  (`world.rs`, with `spawn.rs`) owns the monsters, spells, portals and
  obstacles. A `World` knows nothing of energy, sounds, sparks or memory: it
  moves things and answers with reports (`Hit`, `Contact`, `SpellEvent`,
  `Summoned`), and `play.rs` turns those into changes to her state, sounds and
  things to show. Randomness is the one thing lent out: parts that need luck
  take `&mut Rng`, and the order of the draws is part of the behaviour, so
  keep it when editing.
  Label widths come from a fixed table of Nunito Bold's letter widths
  (`metrics.rs`), not from measuring text, so the simulation never depends
  on the font machinery. Also `rules.rs` (numbers and pure rules),
  `answer.rs` (matching what was typed), `display.rs` (sparks, banners and
  messages, which the rules tell to show but never read), `enemy.rs` and
  `scene.rs` (the read-only `Scene` for drawing; `src/gfx/game_render.rs` in the
  shell is the only code that draws it).
- `booklet/`: the user instruction booklet, `--render-booklet FILE.html`:
  one self-contained HTML file (fonts and PNG pictures embedded as base64),
  16 A4 pages in Finnish, colourful, decorated with the game's characters.
  `scripts/booklet.sh` prints it to PDF with headless Chrome, CI attaches
  the PDF to each GitHub release. `mod.rs` and `pages.rs` are pure (assets in,
  HTML out) and `images.rs` is the shell that draws the pictures with the
  game's own drawing code into render targets. Every number, word,
  consonant and level in the text is looked up from `pairs`, `curriculum`
  and `long_numbers`, never typed in, so the booklet can't disagree with
  the game; keep it that way when editing pages, and test it in
  `booklet/tests.rs`. The text is Finnish. To look at pages, render the PDF
  and read it page by page (`pdftoppm` from poppler is needed for that).
- `pairs.rs`: the 110 pairs and which characters count as answers. Each
  `Pair` carries a `PairId`, its position in `PAIRS`, which memory, the
  game's appearance counts and the pictures are keyed by; `pairs::find`
  looks a pair up by number. The save file still uses numbers.
- `curriculum.rs`: which pairs each level introduces. Level 1 has 0–9, every
  later level adds 5, so all 110 are met by level 21.
- `long_numbers.rs`: long numbers, read greedily two digits at a time with an
  odd digit left over as a single-digit word; leading zeros are allowed. They
  appear as boss hits from the first level after all pairs are met (22), one
  more per level; shown split into pairs and made of well-known pairs for the
  first five such levels; three digits at first, one more every three levels,
  up to six.
- `levels.rs`: checkpoints every five levels up to the best level reached.
  Levels need at most 40 points before their boss.
- `memory.rs`: spaced repetition. A difficulty from 0 (learned) to 1 per pair,
  judged by answer speed, and when it was last seen; review intervals run from
  5 hours to 6 days by difficulty. A streak of good answers in a row (quick,
  no hint; anything else resets it) stretches a learned pair's interval by
  half again per answer past five, up to 3 weeks.
- `progress.rs`: `Profile`, the one copy of what is known about the player:
  the sound setting, the mode being played and a `Progress` for each of the
  two modes, `easy` and `hard` (the memory of each pair, best level, stars,
  streak, stats and badges, kept apart so results compare honestly, and
  each mode has its own badge inventory). Hardcore mode (`Profile::hardcore`:
  monsters never show hints; the title screen's switch, key A; `Game::new`
  takes it and `World::set_hints` passes it on to each monster) plays
  against the `hard` progress; `Profile::current` is the one in use. `App`
  holds the live copy; nothing else keeps a second one.
- `save.rs`: turns a `Profile` into the save file and back (the settings,
  the easy mode's lines, then `mode hardcore` and the hardcore mode's), plain text with one fact per line, damaged lines
  skipped and absurd numbers clamped or dropped (tests read thousands of
  garbage files). Saves from before a field existed must keep loading. On
  the web it goes to local storage.
- `badges.rs`: the badges (kunniamerkit), all in one table, `BADGES`: id,
  Finnish name, category, tier and a `Requirement`. `Stats` are the lifetime
  counters the save keeps (monsters, bosses, long numbers, quick answers,
  practice cards, flawless levels, best combo); everything else a
  requirement asks about (learned pairs, levels, three-star levels, the
  day streak) is worked out from the save and the live `Memory` into a
  `Standing`. `award` records newly earned badges, once, with the day, and
  never takes one back. The game reports counts as `GameEvent`s
  (`Answered`, `MonsterDefeated`, `FlawlessLevel`, and `LevelCompleted` for
  a boss); `App` adds them to the stats and, at the end of each frame,
  awards badges: a toast, `Sfx::Badge`, a `merkki/<id>` count (`merkki/ankara/<id>` in hardcore mode) and a save.
  Players from before badges get theirs quietly in `App::new`. To add a
  badge, add a line to the table (and raise its length); the tests check
  ids, order and that nothing is met from the start.
  `screens/badge_screen.rs` is the screen (rows by category, layout in pure
  functions), `gfx/badge_art.rs` draws the medals (a ribbon, a disc in the tier's
  metal and an `Emblem`, a small picture of the badge's own; `emblem_of` maps
  every badge id to a different one, and a test fails if two share or one is
  missing) and `toast.rs` is the notice (its drawing is `screens/toast.rs` in
  the shell).
  The "sound" switch (Tab) is still `music_on` in the save.
- `screens/`: the other screens, in core (`title`, `levels`, `practice`,
  `progress_map`), each with its layout as pure functions; the shell's
  `screens/*.rs` only draw them. The progress
  map shows how well a pair is known by colour and by 1–3 dots, so colour
  blindness doesn't hide it.
- `touch.rs` (core): the on-screen controls; `gfx/touch_render.rs` draws them. A full Finnish QWERTY keyboard split
  into two panels beside the arena, keys the game doesn't use greyed out; a
  joystick; pause and music buttons. Touches are read from the ordered event
  stream (`input/touch.rs`, shell), because a quick tap can start and end
  within one frame; the mouse counts as a finger.
- `keyboard.rs`: typing by physical key position, as on a Finnish keyboard, so
  Ä and Ö work on any layout.
- `view.rs` (core): a fixed 800×600 virtual arena, scaled to fit the window. In
  touch mode the content also includes the keypad panels at negative x and
  beyond 800. `View::fit` is pure; the shell's `gfx/view.rs` `begin` sets the
  camera and is for drawing only.
- `rng.rs`: seeded randomness in separate streams, so every game introduces
  the same pairs in the same order.
- `audio.rs`, `music.rs`: all sounds are synthesized at startup. The music is
  a score of note names in `music.rs`, a minute-long loop in six sections.
- `pictures.rs`, `sprites.rs`, `obstacles.rs`, `portals.rs`, `icon.rs`:
  everything drawn is code; there are no image assets besides the fonts in
  `assets/`.
- `effects.rs`: sparks, rings and lightning as pure state that moves on with
  time; `src/gfx/game_render.rs` draws it.
- `lifecycle.rs`: notices when the app was away, so the game pauses itself
  (`Frame::away`).
- `web.rs`, `analytics.rs`: the browser version's link to the page.

## Platforms

### Web (lukuloitsu.fi)

- Built by `scripts/web.sh` into `target/web`, with link-time optimization
  turned on through environment variables (about 13% smaller). Netlify runs
  `scripts/netlify-build.sh` (see `netlify.toml`) on every push to `main`;
  `rust-toolchain.toml` is there for Netlify's build machine.
- `.cargo/config.toml` passes `--allow-undefined` to the wasm linker, because
  the JavaScript functions only exist once the page loads the game.
- `src/web.rs` and `web/lukuloitsu.js` are two halves of one interface:
  saving, touch-screen detection, the loading message and analytics events.
  Change them together and bump `PAGE_VERSION` in `web.rs` and `version` in
  the JavaScript, which must match.
- `web/mq_js_bundle.js` is macroquad's JavaScript bundle with two fixes,
  listed at the top of the file: the key where Finnish keyboards have Ä
  reports code 39 (not 222, which miniquad doesn't know), and its quad_net
  plugin declares a variable strict mode requires. Keep those fixes if the
  bundle is ever replaced.
- `web/tietosuoja.html` is the privacy page (Finnish) and `web/og.png` the
  preview image for shared links (1200×630, a crop of the title screen; the
  `og:` tags in `index.html` point at lukuloitsu.fi). `docs/screenshots` has
  the README's pictures. Keep the privacy page, the note at the end of the
  title screen (`PRIVACY_NOTE` in `title.rs`) and the README's statistics
  paragraph in agreement when analytics or saving change.
- `web/index.html` has a script before the bundle that suspends all sound
  while the page is hidden; a hidden page gets no frames, so the game can't
  stop its own music. It also shows "Ladataan…" until the game is ready and
  asks to turn a phone held upright.
- Statistics: GoatCounter, site code `lukuloitsu`
  (https://lukuloitsu.goatcounter.com). It doesn't count localhost. Events
  are paths like `peli-alkoi/taso-1`, `taso-lapaisty/5`,
  `peli-paattyi/taso-4`, `harjoittelu`, `edistyminen`, `kunniamerkit` and
  `merkki/<badge id>` when a badge is earned (`merkki/ankara/<badge id>` in
  hardcore mode).
- The domain is at DNSimple and points to Netlify.

### Android

- `scripts/android.sh` builds with `cargo quad-apk`, installs and starts the
  game. It needs Java 8 (SDKMAN's zulu 8.0.504+1), the SDK in
  `~/Library/Android/sdk` with build-tools 30.0.3 and platform 33, and NDK
  27.2.12479018. Install `cargo-quad-apk` with
  `scripts/install-cargo-quad-apk.sh`, which pins a working GitHub commit and
  applies `scripts/cargo-quad-apk.patch`.
- The app ID is `fi.lukuloitsu.lukuloitsu`. Android loads the game's library
  by the last part of the package name, so it must equal the crate name.
- Saves go to `/data/data/fi.lukuloitsu.lukuloitsu/files/save.txt`.

### iOS

- `scripts/ios-sim.sh` builds for the simulator and launches it.
- `.cargo/config.toml` records iOS SDK 18.0 in the binary, because iOS 27
  won't launch apps built against its own SDK without UIKit's scene
  lifecycle, which miniquad lacks. This is a workaround and wouldn't pass App
  Store review.

### macOS

- `build.rs` embeds `macos/Info.plist` so the menu bar says "Lukuloitsu".
  After renaming the project folder, run `cargo clean -p lukuloitsu`, or the
  build script keeps the old path.
- Saves go to `~/Library/Application Support/Lukuloitsu/save.txt`.
- `scripts/macos-app.sh` makes `Lukuloitsu.app`, a universal (Apple silicon
  and Intel) build with the iOS icon turned into an .icns, and zips it. It
  signs the app ad hoc: the `lipo` step throws away the linker's signature,
  and Apple silicon refuses to run an unsigned program. Without a paid
  Apple Developer ID it can't be notarized, so downloaders right-click and
  choose Open (the release notes say so).

### Linux

- `cargo run --release` builds it (see the README for the libraries).
  `scripts/linux-package.sh` makes the release tarball. CI builds it on
  Ubuntu 22.04, so it runs on distributions with glibc 2.35 or newer; don't
  move it to a newer runner without changing the release notes.
- Saves go to `~/.lukuloitsu/save.txt`.

### Windows

- `cargo run --release` builds it on Windows. From a Mac, install the target
  and MinGW (`rustup target add x86_64-pc-windows-gnu`,
  `brew install mingw-w64`) and run
  `cargo build --release --target x86_64-pc-windows-gnu`.
- The executable is unsigned, so Chrome and SmartScreen warn about it
  (`.github/release-notes.md` tells players what to click). Free signing
  through SignPath Foundation needs every component under an OSI-approved
  license, which the CC BY-SA word list isn't.
- On the one Windows machine tried, neither the executable nor the web
  version made any sound. It is unresolved and probably not the game's fault
  (a muted device or a virtual machine without audio, say).

### Releases

- A release is a version tag. Every part of the project carries the same
  version: bump `version` in `Cargo.toml` (which updates `Cargo.lock`),
  `version_code` in `Cargo.toml` for Android and `CFBundleShortVersionString`
  and `CFBundleVersion` in `ios/Info.plist`. The codes are
  major * 10000 + minor * 100 + patch (0.4.0 is 400). Commit, then
  `git tag -a X.Y.Z -m "Lukuloitsu X.Y"` and push the tag. Tags have no `v`.
  `.github/workflows/release.yml` builds the Windows executable and attaches
  a zip with it and the licenses to a GitHub Release, using
  `.github/release-notes.md` as the text, then adds the macOS app, the Linux
  tarball and the booklet to it. It can also be started by hand
  for an existing tag.

### Shared

- `vendor/miniquad` is a patched copy of macroquad's platform layer; the
  changes are listed in `vendor/README.md`.
- `LUKULOITSU_TOUCH=1 cargo run` shows the touch controls on a computer.
