//! Running the whole app frame by frame, with made-up input.

use super::*;
use crate::badges::Stats;
use crate::game::GameEvent;
use crate::memory::Memory;
use macroquad::prelude::vec2;

const START: f64 = 1_800_000_000.0;
const FRAME: f32 = 1.0 / 60.0;

fn app_with(progress: SaveData, can_quit: bool) -> App {
    App::new(progress, false, can_quit, START)
}

fn app() -> App {
    app_with(SaveData::default(), true)
}

/// A player who has reached `level`, so there are levels to choose from.
fn veteran(level: u32) -> App {
    app_with(
        SaveData {
            best_level: level,
            ..SaveData::default()
        },
        true,
    )
}

fn frame() -> Frame {
    Frame {
        dt: FRAME,
        now: START,
        screen: vec2(800.0, 600.0),
        ..Frame::default()
    }
}

fn press(key: KeyCode) -> Frame {
    Frame {
        pressed: vec![key],
        ..frame()
    }
}

/// Plays `seconds` of quiet frames, starting at `from`, returning the
/// effects.
fn idle(app: &mut App, from: f64, seconds: f32) -> Vec<Effect> {
    let mut effects = Vec::new();
    for i in 0..(seconds / FRAME).round() as usize {
        effects.extend(app.update(&Frame {
            now: from + f64::from(i as f32 * FRAME),
            ..frame()
        }));
    }
    effects
}

fn counted(effects: &[Effect]) -> Vec<String> {
    effects
        .iter()
        .filter_map(|e| match e {
            Effect::Count(event) => Some(event.path()),
            _ => None,
        })
        .collect()
}

fn saves(effects: &[Effect]) -> usize {
    effects
        .iter()
        .filter(|e| matches!(e, Effect::Save(_)))
        .count()
}

fn on_title(app: &App) -> bool {
    matches!(app.screen, Screen::Title)
}

#[test]
fn the_app_starts_at_the_title_screen_with_music_wanted() {
    let app = app();
    assert!(on_title(&app));
    assert!(app.wants_music());
    assert!(app.music_on());
}

#[test]
fn a_new_player_goes_straight_into_a_game() {
    let mut app = app();
    app.update(&press(KeyCode::Enter));
    assert!(matches!(app.screen, Screen::Game(_)));
    let effects = idle(&mut app, START, 0.1);
    assert_eq!(counted(&effects), vec!["peli-alkoi/taso-1"]);
}

#[test]
fn a_returning_player_chooses_the_level_first() {
    let mut app = veteran(12);
    app.update(&press(KeyCode::Enter));
    assert!(matches!(app.screen, Screen::Levels(_)));
    app.update(&press(KeyCode::Enter));
    assert!(matches!(app.screen, Screen::Game(_)));
}

#[test]
fn escape_goes_back_from_the_level_choice() {
    let mut app = veteran(12);
    app.update(&press(KeyCode::Enter));
    app.update(&press(KeyCode::Escape));
    assert!(on_title(&app));
}

#[test]
fn escape_quits_from_the_title_screen_only_where_apps_can_quit() {
    assert!(
        app()
            .update(&press(KeyCode::Escape))
            .contains(&Effect::Quit)
    );
    let mut web = app_with(SaveData::default(), false);
    assert!(!web.update(&press(KeyCode::Escape)).contains(&Effect::Quit));
}

#[test]
fn escape_leaves_a_game_for_the_title_and_saves() {
    let mut app = app();
    app.update(&press(KeyCode::Enter));
    let effects = app.update(&press(KeyCode::Escape));
    assert!(on_title(&app));
    assert_eq!(saves(&effects), 1);
    assert!(
        !effects.contains(&Effect::Quit),
        "Esc in a game doesn't quit"
    );
}

#[test]
fn practice_is_counted_and_left_with_a_save() {
    let mut app = app();
    let effects = app.update(&press(KeyCode::H));
    assert!(matches!(app.screen, Screen::Practice(_)));
    assert_eq!(counted(&effects), vec!["harjoittelu"]);
    let effects = app.update(&press(KeyCode::Escape));
    assert!(on_title(&app));
    assert_eq!(saves(&effects), 1);
}

#[test]
fn the_progress_screen_is_counted_and_left_with_a_key() {
    let mut app = app();
    let effects = app.update(&press(KeyCode::E));
    assert!(matches!(app.screen, Screen::Progress(_)));
    assert_eq!(counted(&effects), vec!["edistyminen"]);
    app.update(&press(KeyCode::Escape));
    assert!(on_title(&app));
}

#[test]
fn tab_switches_the_music_and_saves_the_choice() {
    let mut app = app();
    let effects = app.update(&press(KeyCode::Tab));
    assert!(!app.music_on());
    let saved = last_saved(&effects);
    assert_eq!(saved.map(|d| d.music_on), Some(false));
    app.update(&press(KeyCode::Tab));
    assert!(app.music_on());
}

#[test]
fn a_game_wants_music_only_while_it_is_going() {
    let mut app = app();
    app.update(&press(KeyCode::Enter));
    idle(&mut app, START, 0.1);
    assert!(app.wants_music());
    app.update(&press(KeyCode::Space));
    assert!(!app.wants_music(), "paused");
    app.update(&press(KeyCode::Space));
    assert!(app.wants_music());
    app.update(&Frame {
        away: true,
        ..frame()
    });
    assert!(!app.wants_music(), "coming back from away pauses");
}

#[test]
fn a_game_is_saved_now_and_then_but_not_every_frame() {
    let mut app = app();
    app.update(&press(KeyCode::Enter));
    let quiet = idle(&mut app, START + 0.1, 4.0);
    assert_eq!(saves(&quiet), 0);
    let later = idle(&mut app, START + 5.5, 1.0);
    assert_eq!(saves(&later), 1);
    let soon_after = idle(&mut app, START + 6.6, 1.0);
    assert_eq!(saves(&soon_after), 0);
}

#[test]
fn a_finished_level_is_counted_remembered_and_saved() {
    let mut data = Persistence::new(SaveData::default(), START);
    let event = GameEvent::LevelCompleted { level: 1, stars: 2 };
    assert!(data.record(&event));
    assert_eq!(
        analytics_of(&event).map(|e| e.path()),
        Some("taso-lapaisty/1".to_owned())
    );
    assert_eq!(data.progress.stars.get(&1), Some(&2));
    assert!(data.progress.best_level >= 2);
}

#[test]
fn game_events_are_added_to_the_lifetime_stats() {
    let answered = |quick, long, combo| GameEvent::Answered { quick, long, combo };
    let stats = [
        GameEvent::MonsterDefeated,
        GameEvent::MonsterDefeated,
        answered(true, false, 2),
        answered(false, true, 5),
        answered(false, false, 3),
        GameEvent::FlawlessLevel,
        GameEvent::LevelCompleted { level: 1, stars: 3 },
    ]
    .iter()
    .fold(Stats::default(), events::tally);
    assert_eq!(
        stats,
        Stats {
            monsters: 2,
            bosses: 1,
            long_answers: 1,
            quick_answers: 1,
            practice_correct: 0,
            flawless_levels: 1,
            best_combo: 5,
        }
    );
}

#[test]
fn answers_are_not_worth_a_save_by_themselves() {
    for event in [
        GameEvent::MonsterDefeated,
        GameEvent::Answered {
            quick: true,
            long: false,
            combo: 1,
        },
    ] {
        assert!(!events::worth_saving(&event));
        assert_eq!(analytics_of(&event), None);
    }
}

#[test]
fn earning_a_badge_gives_a_notice_a_sound_a_count_and_a_save_once() {
    let mut app = app();
    app.data.progress.stats.monsters = 1;
    let effects = app.update(&frame());
    assert!(effects.contains(&Effect::Play(Sfx::Badge)));
    assert_eq!(counted(&effects), vec!["merkki/monsters-1"]);
    assert_eq!(saves(&effects), 1);
    assert!(
        last_saved(&effects)
            .unwrap()
            .badges
            .contains_key("monsters-1")
    );
    assert_eq!(app.toasts.current().map(|(b, _)| b.id), Some("monsters-1"));
    // Nothing more happens on the next frame.
    let again = app.update(&frame());
    assert!(counted(&again).is_empty());
    assert_eq!(saves(&again), 0);
}

#[test]
fn several_badges_at_once_each_get_their_notice_in_turn() {
    let mut app = app();
    app.data.progress.stats.monsters = 10;
    let effects = app.update(&frame());
    assert_eq!(
        counted(&effects),
        vec!["merkki/monsters-1", "merkki/monsters-10"]
    );
    assert_eq!(saves(&effects), 1, "one save for the lot");
    assert_eq!(app.toasts.current().map(|(b, _)| b.id), Some("monsters-1"));
}

#[test]
fn the_sound_switch_silences_the_badge_sound_but_not_the_badge() {
    let mut app = app();
    app.update(&press(KeyCode::Tab));
    app.data.progress.stats.monsters = 1;
    let effects = app.update(&frame());
    assert!(!effects.iter().any(|e| matches!(e, Effect::Play(_))));
    assert!(app.data.progress.badges.contains_key("monsters-1"));
    assert!(app.toasts.current().is_some());
}

#[test]
fn badges_earned_before_they_existed_are_given_quietly() {
    let mut app = veteran(6);
    assert!(app.data.progress.badges.contains_key("levels-5"));
    let effects = app.update(&frame());
    assert!(!effects.contains(&Effect::Play(Sfx::Badge)));
    assert!(app.toasts.current().is_none());
    assert!(counted(&effects).is_empty());
}

#[test]
fn learning_pairs_earns_their_badge_at_once() {
    let mut app = app();
    app.update(&press(KeyCode::Enter));
    let mut learned = Memory::default();
    for pair in &crate::pairs::PAIRS[..10] {
        for _ in 0..8 {
            learned.record_answer(*pair, 1.0, false, START);
        }
    }
    app.data.memory = learned;
    let effects = app.update(&frame());
    assert!(counted(&effects).contains(&"merkki/pairs-digits".to_owned()));
}

#[test]
fn k_opens_the_badges_and_escape_comes_back() {
    let mut app = app();
    let effects = app.update(&press(KeyCode::K));
    assert!(matches!(app.screen, Screen::Badges(_)));
    assert_eq!(counted(&effects), vec!["kunniamerkit"]);
    app.update(&press(KeyCode::Escape));
    assert!(on_title(&app));
}

#[test]
fn starting_and_ending_a_game_are_counted_but_not_worth_a_save() {
    let mut data = Persistence::new(SaveData::default(), START);
    let started = GameEvent::Started { level: 5 };
    let over = GameEvent::Over { level: 7 };
    assert!(!data.record(&started));
    assert!(!data.record(&over));
    assert_eq!(
        [started, over]
            .iter()
            .filter_map(analytics_of)
            .map(|e| e.path())
            .collect::<Vec<_>>(),
        vec!["peli-alkoi/taso-5", "peli-paattyi/taso-7"]
    );
    assert_eq!(data.progress, SaveData::default());
}

#[test]
fn sound_effects_are_passed_on() {
    let mut app = app();
    app.update(&press(KeyCode::Enter));
    // Typing with nothing on screen makes a click.
    let effects = app.update(&Frame {
        typed: vec![crate::keyboard::Key::Char('1')],
        ..frame()
    });
    assert!(effects.contains(&Effect::Play(Sfx::Type)));
}

#[test]
fn switching_the_sound_off_silences_the_sound_effects_too() {
    let typing = || Frame {
        typed: vec![crate::keyboard::Key::Char('1')],
        ..frame()
    };
    let mut app = app();
    app.update(&press(KeyCode::Enter));
    app.update(&press(KeyCode::Tab));
    assert!(!app.music_on());
    let mut effects = app.update(&typing());
    effects.extend(idle(&mut app, START, 5.0));
    assert!(
        !effects.iter().any(|e| matches!(e, Effect::Play(_))),
        "{effects:?}"
    );
    // Everything but the sounds carries on, and switching back restores them.
    app.update(&press(KeyCode::Tab));
    assert!(app.update(&typing()).contains(&Effect::Play(Sfx::Type)));
}

#[test]
fn the_same_frames_give_the_same_effects() {
    let run = || {
        let mut app = app();
        let mut all = app.update(&press(KeyCode::Enter));
        all.extend(idle(&mut app, START, 3.0));
        format!("{all:?}")
    };
    assert_eq!(run(), run());
}

fn last_saved(effects: &[Effect]) -> Option<SaveData> {
    effects.iter().rev().find_map(|e| match e {
        Effect::Save(text) => Some(SaveData::from_text(text)),
        _ => None,
    })
}

#[test]
fn what_practice_teaches_is_saved_when_leaving() {
    let mut app = app();
    app.update(&press(KeyCode::H));
    // Giving up on a card is a miss, which is remembered.
    app.update(&press(KeyCode::Space));
    let effects = app.update(&press(KeyCode::Escape));
    let saved = last_saved(&effects).expect("leaving practice saves");
    assert_eq!(saved.pairs.len(), 1);
}

#[test]
fn what_a_game_teaches_is_saved_when_leaving() {
    let mut app = app();
    app.update(&press(KeyCode::Enter));
    // Monsters that nobody answers reach her, and each is a miss.
    idle(&mut app, START, 30.0);
    let effects = app.update(&press(KeyCode::Escape));
    let saved = last_saved(&effects).expect("leaving a game saves");
    assert!(!saved.pairs.is_empty(), "the misses are in the save");
    assert!(on_title(&app));
}

#[test]
fn the_progress_screen_shows_what_was_learned_without_saving_first() {
    let mut app = app();
    app.update(&press(KeyCode::H));
    app.update(&press(KeyCode::Space));
    assert_eq!(app.data.memory.records().count(), 1, "kept live in the app");
    assert!(
        app.data.progress.pairs.is_empty(),
        "not converted every frame"
    );
}

#[test]
fn menu_animations_follow_the_frames_shown() {
    let mut app = app();
    idle(&mut app, START, 2.0);
    assert!((app.time - 2.0).abs() < 0.01, "{}", app.time);
}
