//! Playing whole games without a screen: the simulation is fed inputs and
//! its outputs are checked.

use super::*;
use crate::long_numbers;
use crate::pairs::PAIRS;

const START: f64 = 1_800_000_000.0;
const FRAME: f32 = 1.0 / 60.0;

fn game_from(level: u32) -> Game {
    Game::new(false, Memory::default(), level)
}

fn game() -> Game {
    game_from(1)
}

fn frame() -> Input {
    Input {
        dt: FRAME,
        now: START,
        ..Input::default()
    }
}

fn typing(text: &str) -> Input {
    Input {
        keys: text.chars().map(Key::Char).collect(),
        ..frame()
    }
}

/// What a run of frames asked the outside world to do.
#[derive(Default)]
struct Log {
    sfx: Vec<Sfx>,
    events: Vec<GameEvent>,
}

impl Log {
    fn add(&mut self, outputs: Outputs) {
        self.sfx.extend(outputs.sfx);
        self.events.extend(outputs.events);
    }

    fn append(&mut self, later: Log) {
        self.sfx.extend(later.sfx);
        self.events.extend(later.events);
    }
}

/// Plays `seconds` of frames with `input` held, returning what happened.
fn play(game: &mut Game, seconds: f32, input: &Input) -> Log {
    let mut log = Log::default();
    for _ in 0..(seconds / FRAME).round() as usize {
        log.add(game.update(input));
    }
    log
}

fn idle(game: &mut Game, seconds: f32) -> Log {
    play(game, seconds, &frame())
}

/// A game with one monster showing `pair` right on top of the player, so
/// it hurts her on the next frame.
fn with_collision(game: &mut Game) {
    let mut enemy = Enemy::new(Question::single(PAIRS[0]), false, 0, 0.0);
    enemy.pos = game.player;
    game.admit(enemy);
}

/// A game with one monster far from the player.
fn with_monster(game: &mut Game) {
    let mut enemy = Enemy::new(Question::single(PAIRS[22 + 10]), false, 0, 0.0);
    enemy.pos = vec2(50.0, 50.0);
    game.admit(enemy);
}

#[test]
fn a_new_game_reports_that_it_started() {
    let mut game = game_from(4);
    let outputs = game.update(&frame());
    assert_eq!(outputs.events, vec![GameEvent::Started { level: 4 }]);
    // ...and only once.
    assert!(game.update(&frame()).events.is_empty());
}

#[test]
fn monsters_start_coming_after_a_moment() {
    let mut game = game();
    idle(&mut game, 0.5);
    assert!(game.enemies.is_empty());
    idle(&mut game, 1.0);
    assert!(!game.enemies.is_empty());
}

#[test]
fn monsters_never_appear_close_to_the_player() {
    let mut game = game();
    idle(&mut game, 1.2);
    for enemy in &game.enemies {
        assert!(enemy.pos.distance(game.player) >= 200.0);
    }
}

#[test]
fn typing_a_monsters_answer_casts_a_spell_and_scores() {
    let mut game = game();
    with_monster(&mut game);
    let answer = game.enemies[0].answer();
    let mut log = Log::default();
    log.add(game.update(&typing(&answer)));
    assert!(game.enemies.is_empty(), "the monster is out of play");
    assert_eq!(game.score, 1);
    assert_eq!(game.stage.points, 1);
    assert!(log.sfx.contains(&Sfx::Cast));
    // The spell explodes it a moment later.
    assert!(idle(&mut game, 1.0).sfx.contains(&Sfx::Explode));
    assert!(game.spells.is_empty());
}

#[test]
fn a_right_answer_is_remembered() {
    let mut game = game();
    with_monster(&mut game);
    let pair = game.enemies[0].question.first();
    let answer = game.enemies[0].answer();
    game.update(&typing(&answer));
    assert_eq!(game.memory().record(&pair).map(|r| r.times_seen), Some(1));
}

#[test]
fn a_wrong_key_costs_energy_after_the_first_dead_end() {
    let mut game = game();
    // With no monsters, any character leads nowhere.
    let outputs = game.update(&typing("1"));
    assert_eq!(
        game.energy, MAX_ENERGY,
        "a typo is free until she types past it"
    );
    assert!(!outputs.sfx.contains(&Sfx::Wrong));
    let outputs = game.update(&typing("1"));
    assert_eq!(game.energy, MAX_ENERGY - WRONG_PENALTY);
    assert!(outputs.sfx.contains(&Sfx::Wrong));
    assert!(game.typed.get(Slot::Number).is_empty());
}

#[test]
fn wrong_keys_alone_never_end_the_game() {
    let mut game = game();
    game.update(&typing(&"1".repeat(100)));
    assert_eq!(game.energy, LOW_ENERGY);
    assert!(!game.is_over());
}

#[test]
fn backspace_empties_both_slots() {
    let mut game = game();
    game.update(&typing("1"));
    game.update(&typing("s"));
    assert_eq!(
        (game.typed.get(Slot::Number), game.typed.get(Slot::Word)),
        ("1", "s")
    );
    game.update(&Input {
        keys: vec![Key::Backspace],
        ..frame()
    });
    assert!((game.typed.get(Slot::Number).is_empty() && game.typed.get(Slot::Word).is_empty()));
}

#[test]
fn keys_that_are_not_answers_are_ignored() {
    let mut game = game();
    game.update(&typing("wxz"));
    assert!(game.typed.get(Slot::Word).is_empty());
}

#[test]
fn a_monster_reaching_her_hurts_and_teaches() {
    let mut game = game();
    with_collision(&mut game);
    let mut log = Log::default();
    log.add(game.update(&typing("1")));
    log.add(game.update(&frame()));
    assert_eq!(game.energy, MAX_ENERGY - COLLISION_PENALTY);
    assert!(game.enemies.is_empty());
    assert!(log.sfx.contains(&Sfx::Hurt));
    assert!(
        game.typed.get(Slot::Number).is_empty(),
        "what she typed is dropped"
    );
    let feedback = game.feedback.as_ref().expect("the pair is shown");
    assert!(feedback.text.contains(PAIRS[0].number));
    assert!(game.memory().record(&PAIRS[0]).is_some());
}

#[test]
fn the_game_ends_once_when_energy_runs_out() {
    let mut game = game();
    game.energy = COLLISION_PENALTY;
    with_collision(&mut game);
    let mut log = Log::default();
    log.add(game.update(&frame()));
    log.add(game.update(&frame()));
    idle(&mut game, 1.0);
    assert!(game.is_over());
    assert_eq!(
        log.events,
        vec![
            GameEvent::Started { level: 1 },
            GameEvent::Over { level: 1 }
        ]
    );
}

#[test]
fn nothing_moves_after_the_game_is_over() {
    let mut game = game();
    game.energy = 0.0;
    let before = game.player;
    let outputs = play(
        &mut game,
        1.0,
        &Input {
            arrows: vec2(1.0, 0.0),
            ..frame()
        },
    );
    assert_eq!(game.player, before);
    assert_eq!(outputs.events, vec![GameEvent::Started { level: 1 }]);
}

#[test]
fn enter_starts_over_and_keeps_what_she_learned() {
    let mut game = game_from(3);
    with_monster(&mut game);
    let answer = game.enemies[0].answer();
    game.update(&typing(&answer));
    let learned = game.memory().records().count();
    assert_eq!(learned, 1);
    game.energy = 0.0;
    game.level = 5;

    // Nothing but Enter or a tap restarts it.
    game.update(&frame());
    assert!(game.is_over());
    let outputs = game.update(&Input {
        confirm: true,
        ..frame()
    });
    assert!(!game.is_over());
    assert_eq!((game.level, game.energy, game.score), (3, MAX_ENERGY, 0));
    assert_eq!(game.memory().records().count(), learned);
    assert_eq!(outputs.events, vec![GameEvent::Started { level: 3 }]);
}

#[test]
fn a_tap_also_starts_over() {
    let mut game = game();
    game.energy = 0.0;
    game.update(&Input { taps: 1, ..frame() });
    assert!(!game.is_over());
}

#[test]
fn pausing_stops_time_and_drops_typing() {
    let mut game = game();
    with_monster(&mut game);
    game.update(&Input {
        pause: true,
        ..frame()
    });
    assert!(game.is_paused());
    let (time, pos) = (game.stage.time, game.enemies[0].pos);
    let animation = game.play_time;
    let answer = game.enemies[0].answer();
    play(&mut game, 1.0, &typing(&answer));
    assert_eq!(game.stage.time, time);
    assert_eq!(game.play_time, animation, "animations freeze too");
    assert_eq!(game.enemies[0].pos, pos);
    assert_eq!(game.score, 0, "typing during a pause does nothing");

    game.update(&Input {
        pause: true,
        ..frame()
    });
    assert!(!game.is_paused());
    assert!((game.typed.get(Slot::Number).is_empty() && game.typed.get(Slot::Word).is_empty()));
}

#[test]
fn a_tap_resumes_a_paused_game() {
    let mut game = game();
    game.update(&Input {
        pause: true,
        ..frame()
    });
    game.update(&Input { taps: 1, ..frame() });
    assert!(!game.is_paused());
}

#[test]
fn coming_back_after_being_away_finds_the_game_paused() {
    let mut game = game();
    game.update(&Input {
        away: true,
        ..frame()
    });
    assert!(game.is_paused());
}

#[test]
fn an_over_game_is_not_paused_by_being_away() {
    let mut game = game();
    game.energy = 0.0;
    game.update(&Input {
        away: true,
        ..frame()
    });
    assert!(!game.is_paused());
}

#[test]
fn scoring_enough_points_summons_the_boss() {
    let mut game = game();
    game.add_points(points_to_clear(1) - 1);
    assert!(game.enemies.is_empty());
    let mut log = Log::default();
    game.add_points(1);
    log.add(std::mem::take(&mut game.out));
    assert!(game.stage.boss_fight);
    assert!(log.sfx.contains(&Sfx::Boss));
    let boss = &game.enemies[0];
    assert!(boss.is_boss());
    assert_eq!(boss.boss.as_ref().map(|b| b.total), Some(boss_hits(1)));
}

#[test]
fn no_new_monsters_join_a_boss_fight() {
    let mut game = game();
    game.add_points(points_to_clear(1));
    idle(&mut game, 8.0);
    assert!(game.enemies.iter().all(Enemy::is_boss));
    assert_eq!(game.enemies.len(), 1);
}

#[test]
fn beating_the_boss_completes_the_level() {
    let mut game = game();
    game.add_points(points_to_clear(1));
    let mut log = Log::default();
    for _ in 0..boss_hits(1) {
        assert_eq!(game.enemies.len(), 1, "the boss stays until its last hit");
        let answer = game.enemies[0].answer();
        log.add(game.update(&typing(&answer)));
    }
    assert!(game.enemies.is_empty());
    assert_eq!(game.level, 1, "the level ends when the last spell lands");
    log.append(idle(&mut game, 1.5));

    assert_eq!(game.level, 2);
    assert!(!game.stage.boss_fight);
    assert_eq!(game.stage.points, 0);
    assert_eq!(
        log.events.last(),
        Some(&GameEvent::LevelCompleted { level: 1, stars: 3 })
    );
    assert_eq!(
        log.events
            .iter()
            .filter(|e| matches!(e, GameEvent::LevelCompleted { .. }))
            .count(),
        1
    );
    assert!(log.sfx.contains(&Sfx::LevelUp));
    assert_eq!(
        game.banner.as_ref().map(|b| b.title.as_str()),
        Some("Taso 2!")
    );
}

#[test]
fn stars_depend_on_the_energy_left() {
    let mut game = game();
    game.energy = 50.0;
    game.add_points(points_to_clear(1));
    let mut log = Log::default();
    for _ in 0..boss_hits(1) {
        let answer = game.enemies[0].answer();
        log.add(game.update(&typing(&answer)));
    }
    log.append(idle(&mut game, 1.5));
    assert!(
        log.events
            .contains(&GameEvent::LevelCompleted { level: 1, stars: 2 })
    );
}

#[test]
fn a_boss_hitting_her_bounces_back_and_hurts_once_in_a_while() {
    let mut game = game();
    game.add_points(points_to_clear(1));
    game.enemies[0].pos = game.player + vec2(20.0, 0.0);
    game.update(&frame());
    assert_eq!(game.energy, MAX_ENERGY - COLLISION_PENALTY);
    assert!(game.enemies[0].pos.distance(game.player) > 100.0);
    assert!(!game.is_over());
}

#[test]
fn a_new_level_brings_new_pairs() {
    let mut game = game();
    let before = game.curriculum.unlocked().len();
    game.complete_level(vec2(100.0, 100.0));
    assert_eq!(game.curriculum.unlocked().len(), before + 5);
}

#[test]
fn the_same_inputs_play_the_same_game() {
    let run = || {
        let mut game = game();
        let mut log = Log::default();
        for i in 0..600 {
            let keys = if i % 37 == 0 {
                vec![Key::Char('k')]
            } else {
                vec![]
            };
            log.add(game.update(&Input {
                keys,
                arrows: vec2(1.0, 0.0),
                ..frame()
            }));
        }
        let enemies: Vec<_> = game
            .enemies
            .iter()
            .map(|e| (e.pos, e.label.clone()))
            .collect();
        format!(
            "{:?} {:?} {:?} {}",
            enemies, log.sfx, log.events, game.energy
        )
    };
    assert_eq!(run(), run());
}

#[test]
fn the_arrow_keys_move_her_at_a_steady_speed() {
    let mut game = game();
    let start = game.player;
    play(
        &mut game,
        0.5,
        &Input {
            arrows: vec2(1.0, 0.0),
            ..frame()
        },
    );
    assert!((game.player.x - start.x - PLAYER_SPEED * 0.5).abs() < 1.0);
    assert_eq!(game.player.y, start.y);
}

#[test]
fn diagonal_movement_is_no_faster() {
    let mut game = game();
    let start = game.player;
    play(
        &mut game,
        0.5,
        &Input {
            arrows: vec2(1.0, 1.0),
            ..frame()
        },
    );
    let travelled = game.player.distance(start);
    assert!((travelled - PLAYER_SPEED * 0.5).abs() < 1.0);
}

#[test]
fn she_stays_inside_the_arena() {
    let mut game = game();
    play(
        &mut game,
        10.0,
        &Input {
            arrows: vec2(1.0, -1.0),
            ..frame()
        },
    );
    assert!(game.player.x <= ARENA_W && game.player.y >= 0.0);
}

#[test]
fn the_touch_stick_moves_her_by_how_far_it_is_pushed() {
    let mut game = game();
    let start = game.player;
    play(
        &mut game,
        0.5,
        &Input {
            stick: vec2(0.0, -0.5),
            ..frame()
        },
    );
    assert!((start.y - game.player.y - PLAYER_SPEED * 0.25).abs() < 1.0);
}

#[test]
fn a_long_frame_plays_like_the_same_time_in_short_ones() {
    let arrows = vec2(1.0, 0.0);
    let mut short = game();
    play(&mut short, 0.25, &Input { arrows, ..frame() });
    let mut long = game();
    long.update(&Input {
        dt: 0.25,
        arrows,
        ..frame()
    });
    assert!((short.player.x - long.player.x).abs() < 0.5);
    assert!((short.stage.time - long.stage.time).abs() < 0.01);
}

#[test]
fn a_stall_does_not_make_the_game_jump_ahead() {
    let mut game = game();
    game.update(&Input {
        dt: 30.0,
        ..frame()
    });
    assert!(game.stage.time <= MAX_FRAME_SECONDS + 1e-3);
}

#[test]
fn a_fast_spell_still_lands_in_a_long_frame() {
    let mut game = game();
    with_monster(&mut game);
    let answer = game.enemies[0].answer();
    game.update(&typing(&answer));
    let log = play(
        &mut game,
        1.0,
        &Input {
            dt: 0.25,
            ..frame()
        },
    );
    assert!(game.spells.is_empty());
    assert!(log.sfx.contains(&Sfx::Explode));
}

#[test]
fn enemies_keep_their_ids_when_others_leave() {
    let mut game = game();
    for x in [50.0, 150.0, 250.0] {
        let mut enemy = Enemy::new(Question::single(PAIRS[x as usize / 100]), false, 0, 0.0);
        enemy.pos = vec2(x, 50.0);
        game.admit(enemy);
    }
    let ids: Vec<EnemyId> = game.enemies.iter().map(|e| e.id).collect();
    assert!(ids[0] != ids[1] && ids[1] != ids[2] && ids[0] != ids[2]);

    game.hit_enemy(ids[0]);
    let left: Vec<EnemyId> = game.enemies.iter().map(|e| e.id).collect();
    assert_eq!(left, [ids[1], ids[2]]);
    assert!(game.enemy(ids[0]).is_none());
    assert_eq!(game.enemy(ids[2]).map(|e| e.pos.x), Some(250.0));

    // Hitting one that is already gone does nothing.
    game.hit_enemy(ids[0]);
    assert_eq!(game.enemies.len(), 2);
}

/// Plays like a perfect typist who never moves: each frame, types the
/// answer to the first monster on screen. Stops after `max_seconds` or
/// once `done` says so; returns everything that happened.
fn autoplay(game: &mut Game, max_seconds: f32, done: impl Fn(&Game) -> bool) -> Log {
    let mut log = Log::default();
    for _ in 0..(max_seconds / FRAME) as usize {
        let input = match game.enemies.first() {
            Some(enemy) => typing(&enemy.answer()),
            None => frame(),
        };
        log.add(game.update(&input));
        if done(game) {
            break;
        }
    }
    log
}

fn completed_levels(log: &Log) -> Vec<u32> {
    log.events
        .iter()
        .filter_map(|event| match event {
            GameEvent::LevelCompleted { level, .. } => Some(*level),
            _ => None,
        })
        .collect()
}

#[test]
fn a_perfect_typist_plays_through_the_first_levels() {
    let mut game = game();
    let mut log = autoplay(&mut game, 600.0, |g| g.level == 5);
    log.add(std::mem::take(&mut game.out));

    assert_eq!(game.level, 5, "four levels are cleared within ten minutes");
    assert_eq!(completed_levels(&log), [1, 2, 3, 4]);
    assert_eq!(log.events.first(), Some(&GameEvent::Started { level: 1 }));
    assert!(
        !log.events
            .iter()
            .any(|e| matches!(e, GameEvent::Over { .. })),
        "a perfect typist never loses"
    );
    assert_eq!(game.energy, MAX_ENERGY);
    assert!(log.sfx.contains(&Sfx::Boss), "every level ends in a boss");
    // Each level introduces pairs, and the typist met the level's pairs.
    assert!(game.appearances.len() > 5);
}

#[test]
fn the_first_long_number_level_can_be_cleared() {
    let start = long_numbers::first_long_level();
    let mut game = game_from(start);
    let log = autoplay(&mut game, 600.0, |g| g.level > start);

    assert_eq!(completed_levels(&log), [start]);
    assert!(
        game.appearances.contains_key(&Appearance::Long),
        "the boss showed a long number"
    );
}

#[test]
fn a_typist_who_never_types_loses_on_the_first_level() {
    let mut game = game();
    let log = idle(&mut game, 300.0);

    assert!(game.is_over());
    let overs = log
        .events
        .iter()
        .filter(|e| matches!(e, GameEvent::Over { level: 1 }))
        .count();
    assert_eq!(overs, 1, "game over is reported exactly once");
    assert!(completed_levels(&log).is_empty());
}

/// Plays `seconds` at `fps` frames per second, moving right all the time.
fn play_at(fps: f32, seconds: f32) -> Game {
    let mut game = game();
    for _ in 0..(seconds * fps).round() as usize {
        game.update(&Input {
            dt: 1.0 / fps,
            arrows: vec2(1.0, 0.0),
            ..frame()
        });
    }
    game
}

#[test]
fn a_game_plays_out_the_same_at_any_frame_rate() {
    let reference = play_at(60.0, 20.0);
    assert!(!reference.enemies.is_empty(), "something happened");
    for fps in [30.0, 120.0, 240.0] {
        let other = play_at(fps, 20.0);
        assert_eq!(other.player, reference.player, "{fps} fps");
        assert_eq!(other.play_time, reference.play_time, "{fps} fps");
        assert_eq!(other.enemies.len(), reference.enemies.len(), "{fps} fps");
        for (a, b) in other.enemies.iter().zip(&reference.enemies) {
            assert_eq!(a.question.number(false), b.question.number(false));
            assert_eq!(a.pos, b.pos, "{fps} fps");
        }
    }
}

#[test]
fn typing_on_a_frame_too_short_for_a_step_is_not_lost() {
    let mut game = game();
    with_monster(&mut game);
    let answer = game.enemies[0].answer();
    // A frame at 1000 fps is a fraction of a step.
    let short = |input: Input| Input { dt: 0.001, ..input };
    for c in answer.chars() {
        game.update(&short(typing(&c.to_string())));
    }
    idle(&mut game, 0.1);
    assert_eq!(game.score, 1);
}
