# Notes for AI agents

Read [README.md](README.md) first for what the game is. This file is what an
agent working on the code needs to know on top of that: the rules, how the
code fits together, and pitfalls already found the hard way.

## Rules

- **Talk to the user in English. Everything in the game stays in Finnish**:
  menus, messages, banners, help texts.
- **The pair list in `src/pairs.rs` is authoritative.** Never "fix" a word,
  even if it looks odd; change a pair only when the user asks (44 became
  MUUMIO that way). Every pair has a picture in `src/pictures.rs`.
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
  OpenGL libraries first.
- To see a screen, temporarily make `main` draw it and call
  `get_screen_data().export_png(path)`, then restore `src/main.rs`. Put
  screenshots, rendered music and other scratch files outside the repository.
- **Stop the game's audio after every test.** The music loops, and a forgotten
  instance keeps playing unseen: run `xcrun simctl shutdown all` after
  simulator tests, close browser tabs showing the game and stop the preview
  server, and don't leave the desktop build running.
- A hidden browser preview pane gets frames only in bursts. The game reads a
  long gap between frames as the player being away and pauses itself,
  discarding that frame's keys, so keyboard tests there are unreliable.
  Practice mode doesn't auto-pause. The browser tool's `type` action sends
  no physical key codes, which the game reads; use its `key` action, one key
  at a time.
- The Android phone (a moto g15) is reached with `adb -d`, which picks the
  USB device even when emulators are listed.

## How the code fits together

**Pure core, impure shell.** Side effects live only in the outermost layer.
`main.rs` is the shell: each frame it reads input into a `Frame`
(`frame.rs`, the one place that asks the window and the clock), passes it
to `App::update` (`app.rs`), performs the `Effect`s that come back (sounds,
saving, statistics, quitting), and calls `App::draw`. Everything between is
a pure state machine that takes input as arguments and reports what it
wants done as return values, so it is tested by running made-up frames
through it (see `src/app/tests.rs` and `src/game/tests.rs`). Keep it that
way: don't call `is_key_pressed`, `get_frame_time`, `date::now` or measure
text inside the core; add a field to `Frame` or `game::Input` instead.
Drawing (`draw` methods, `game/render.rs`, sprites, pictures) reads state
and never changes it.

- `main.rs`: the shell, described above, and the `--render-*` commands.
- `app.rs`: which screen is showing (title, level choice, game, practice,
  progress), progress and saving, and the `Effect`s.
- `frame.rs`: `Frame`, one frame of input, and `Inputs::read`, which makes
  it.
- `game/`: the game itself. Monsters show a number or a word; typed digits
  go to the number slot and letters to the word slot, each answering the
  monsters showing the other kind. Backspace empties both slots.
  `Game::update(&Input) -> Outputs` is the only way in; `Outputs` carry
  sound effects and `GameEvent`s (started, level completed, game over) for
  saving and analytics. A frame is advanced in steps of at most 1/30 s and
  never more than 0.25 s in all, so a slow frame can't skip past a hit.
  Text width is injected (`TextWidth`), because only the shell knows the
  loaded font. Split into `rules.rs` (numbers and pure rules), `answer.rs`
  (matching what was typed), `enemy.rs`, `spawn.rs`, `combat.rs` (spells,
  collisions, movement) and `render.rs` (the only file that draws).
- `pairs.rs`: the 110 pairs and which characters count as answers.
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
  5 hours to 6 days.
- `save.rs`: the save file, plain text with one fact per line, damaged lines
  skipped. On the web it goes to local storage.
- `practice.rs`, `progress.rs`, `title.rs`: the other screens.
- `touch.rs`: the on-screen controls. A full Finnish QWERTY keyboard split
  into two panels beside the arena, keys the game doesn't use greyed out; a
  joystick; pause and music buttons. Touches are read from the ordered event
  stream, because a quick tap can start and end within one frame; the mouse
  counts as a finger.
- `keyboard.rs`: typing by physical key position, as on a Finnish keyboard, so
  Ä and Ö work on any layout.
- `view.rs`: a fixed 800×600 virtual arena, scaled to fit the window. In touch
  mode the content also includes the keypad panels at negative x and beyond
  800. `View::fit` is pure; `view::begin` sets the camera and is for
  drawing only.
- `rng.rs`: seeded randomness in separate streams, so every game introduces
  the same pairs in the same order.
- `audio.rs`, `music.rs`: all sounds are synthesized at startup. The music is
  a score of note names in `music.rs`, a minute-long loop in six sections.
- `pictures.rs`, `sprites.rs`, `effects.rs`, `obstacles.rs`, `portals.rs`,
  `icon.rs`: everything drawn is code; there are no image assets besides the
  fonts in `assets/`.
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
- `web/index.html` has a script before the bundle that suspends all sound
  while the page is hidden; a hidden page gets no frames, so the game can't
  stop its own music. It also shows "Ladataan…" until the game is ready and
  asks to turn a phone held upright.
- Statistics: GoatCounter, site code `lukuloitsu`
  (https://lukuloitsu.goatcounter.com). It doesn't count localhost. Events
  are paths like `peli-alkoi/taso-1`, `taso-lapaisty/5`,
  `peli-paattyi/taso-4`, `harjoittelu` and `edistyminen`.
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

### Shared

- `vendor/miniquad` is a patched copy of macroquad's platform layer; the
  changes are listed in `vendor/README.md`.
- `LUKULOITSU_TOUCH=1 cargo run` shows the touch controls on a computer.
