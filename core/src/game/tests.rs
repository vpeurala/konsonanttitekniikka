//! Playing whole games without a screen: the simulation is fed inputs and
//! its outputs are checked.

use super::rules::*;
use super::*;
use crate::arena::ARENA_W;
use crate::long_numbers::{self, Question};
use crate::memory::Happened;
use crate::pairs::PAIRS;
use enemy::EnemyId;
use glam::vec2;

const START: f64 = 1_800_000_000.0;
const FRAME: f32 = 1.0 / 60.0;

/// A game together with the memory the caller keeps for it, as `App`
/// does. It reads through to the game, so tests look at its fields.
struct Rig {
    game: Game,
    memory: Memory,
}

impl Rig {
    fn update(&mut self, input: &Input) -> Outputs {
        let out = self.game.update(input, &self.memory);
        self.learn(&out);
        out
    }

    /// Learns what the caller is told to, as `App` does.
    fn learn(&mut self, out: &Outputs) {
        for lesson in &out.lessons {
            self.memory.learn(lesson);
        }
    }

    fn memory(&self) -> &Memory {
        &self.memory
    }

    fn hit_enemy(&mut self, id: EnemyId) -> Outputs {
        let player = self.game.player.pos;
        let Some(hit) = self.game.world.hit(id, player, &self.memory) else {
            return Outputs::default();
        };
        let out = self
            .game
            .learn_from(&hit, START, &mut Known::Borrowed(&self.memory));
        self.learn(&out);
        out
    }

    fn add_points(&mut self, points: u32) -> Outputs {
        self.game.add_points(points, START, &self.memory)
    }
}

impl std::ops::Deref for Rig {
    type Target = Game;
    fn deref(&self) -> &Game {
        &self.game
    }
}

impl std::ops::DerefMut for Rig {
    fn deref_mut(&mut self) -> &mut Game {
        &mut self.game
    }
}

fn game_from(level: u32) -> Rig {
    Rig {
        game: Game::new(false, level),
        memory: Memory::default(),
    }
}

fn game() -> Rig {
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
fn play(game: &mut Rig, seconds: f32, input: &Input) -> Log {
    let mut log = Log::default();
    for _ in 0..(seconds / FRAME).round() as usize {
        log.add(game.update(input));
    }
    log
}

fn idle(game: &mut Rig, seconds: f32) -> Log {
    play(game, seconds, &frame())
}

/// A game with one monster showing `pair` right on top of the player, so
/// it hurts her on the next frame.
fn with_collision(game: &mut Rig) {
    let mut enemy = Enemy::new(Question::single(PAIRS[0]), false, 0.5, 0.0);
    enemy.pos = game.player.pos;
    game.world.admit(enemy);
}

/// A game with one monster far from the player.
fn with_monster(game: &mut Rig) {
    let mut enemy = Enemy::new(Question::single(PAIRS[22 + 10]), false, 0.5, 0.0);
    enemy.pos = vec2(50.0, 50.0);
    game.world.admit(enemy);
}

/// Answers the only monster in play, which shows a word if `cyclops`, and
/// returns the points and the energy it gave, starting from 50% energy.
fn worth_of_answering(cyclops: bool) -> (u32, f32) {
    let mut game = game();
    game.vitals = game.vitals.with_energy(50.0);
    let mut enemy = Enemy::new(Question::single(PAIRS[22 + 10]), cyclops, 0.5, 0.0);
    enemy.pos = vec2(50.0, 50.0);
    game.world.admit(enemy);
    let answer = game.world.enemies()[0].answer().to_owned();
    game.update(&typing(&answer));
    (game.vitals.score(), game.vitals.energy() - 50.0)
}

#[test]
fn a_one_eyed_monster_is_worth_a_point_and_10_percent_energy() {
    assert_eq!(worth_of_answering(true), (1, 10.0));
}

#[test]
fn a_star_monster_is_worth_two_points_and_20_percent_energy() {
    assert_eq!(worth_of_answering(false), (2, 20.0));
}

#[test]
fn a_boss_gives_a_point_and_10_percent_for_each_of_its_numbers() {
    let mut game = game();
    game.vitals = game.vitals.with_energy(50.0);
    game.add_points(points_to_clear(1));
    let numbers = boss_hits(1);
    for _ in 0..numbers {
        let answer = game.world.enemies()[0].answer().to_owned();
        game.update(&typing(&answer));
    }
    assert_eq!(game.vitals.score(), points_to_clear(1) + numbers as u32);
    assert_eq!(game.vitals.energy(), 50.0 + 10.0 * numbers as f32);
}

/// How far a walker, starting at the left with a boss in its way, strays
/// from the straight line to the player before it gets by.
fn detour_around_a_boss(cyclops: bool) -> f32 {
    let mut game = game();
    game.player.pos = vec2(700.0, 300.0);
    let mut walker = Enemy::new(Question::single(PAIRS[22 + 10]), cyclops, 0.5, 0.0);
    walker.pos = vec2(100.0, 300.0);
    walker.start_speed = 150.0;
    let id = walker.id;
    game.world.admit(walker);
    let numbers = [Question::single(PAIRS[1]), Question::single(PAIRS[2])];
    let mut boss = Enemy::boss(&numbers, false, 0.5, 0.0);
    boss.pos = vec2(250.0, 300.0);
    boss.start_speed = 0.0;
    boss.speed_growth = 0.0;
    game.world.admit(boss);
    let mut detour = 0.0f32;
    for _ in 0..90 {
        game.update(&frame());
        if let Some(e) = game.world.enemy(id) {
            detour = detour.max((e.pos.y - 300.0).abs());
        }
    }
    detour
}

#[test]
fn a_star_steers_around_monsters_in_its_way_more_than_a_one_eyed_monster_does() {
    let (star, cyclops) = (detour_around_a_boss(false), detour_around_a_boss(true));
    assert!(star > cyclops + 5.0, "star {star}, cyclops {cyclops}");
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
    assert!(game.world.enemies().is_empty());
    idle(&mut game, 1.0);
    assert!(!game.world.enemies().is_empty());
}

#[test]
fn monsters_never_appear_close_to_the_player() {
    let mut game = game();
    idle(&mut game, 1.2);
    for enemy in game.world.enemies() {
        assert!(enemy.pos.distance(game.player.pos) >= 200.0);
    }
}

#[test]
fn typing_a_monsters_answer_casts_a_spell_and_scores() {
    let mut game = game();
    with_monster(&mut game);
    let answer = game.world.enemies()[0].answer().to_owned();
    let mut log = Log::default();
    log.add(game.update(&typing(&answer)));
    assert!(
        game.world.enemies().is_empty(),
        "the monster is out of play"
    );
    assert_eq!(game.vitals.score(), STAR_POINTS);
    assert_eq!(game.stage.points, STAR_POINTS);
    assert!(log.sfx.contains(&Sfx::Cast));
    // The spell explodes it a moment later.
    assert!(idle(&mut game, 1.0).sfx.contains(&Sfx::Explode));
    assert!(game.world.spells().is_empty());
}

#[test]
fn the_game_reports_what_she_learns_and_leaves_the_memory_to_the_caller() {
    let mut game = game();
    with_monster(&mut game);
    let pair = game.world.enemies()[0].question.first();
    let answer = game.world.enemies()[0].answer().to_owned();
    let out = game.game.update(&typing(&answer), &game.memory);
    assert!(game.memory.record(&pair).is_none(), "not written to");
    assert_eq!(out.lessons.len(), 1);
    assert_eq!(out.lessons[0].pair, pair);
    assert!(matches!(out.lessons[0].what, Happened::Answered { .. }));
}

#[test]
fn a_right_answer_is_remembered() {
    let mut game = game();
    with_monster(&mut game);
    let pair = game.world.enemies()[0].question.first();
    let answer = game.world.enemies()[0].answer().to_owned();
    game.update(&typing(&answer));
    assert_eq!(game.memory().record(&pair).map(|r| r.times_seen), Some(1));
}

#[test]
fn a_wrong_key_costs_energy_after_the_first_dead_end() {
    let mut game = game();
    // With no monsters, any character leads nowhere.
    let outputs = game.update(&typing("1"));
    assert_eq!(
        game.vitals.energy(),
        FULL_ENERGY,
        "a typo is free until she types past it"
    );
    assert!(!outputs.sfx.contains(&Sfx::Wrong));
    let outputs = game.update(&typing("1"));
    assert_eq!(game.vitals.energy(), FULL_ENERGY - WRONG_PENALTY);
    assert!(outputs.sfx.contains(&Sfx::Wrong));
    // The key starts afresh in the emptied slot.
    assert_eq!(game.typed.get(Slot::Number), "1");
}

#[test]
fn defeating_monsters_raises_the_energy_above_100_up_to_the_levels_cap() {
    let mut game = game();
    for _ in 0..40 {
        with_monster(&mut game);
        let answer = game.world.enemies()[0].answer().to_owned();
        game.update(&typing(&answer));
        game.world.enemies_mut().clear();
        assert!(game.vitals.energy() <= energy_cap(1));
    }
    assert_eq!(game.vitals.energy(), energy_cap(1));
    assert!(game.vitals.energy() > FULL_ENERGY);
}

#[test]
fn a_wrong_key_empties_both_slots_so_the_next_word_starts_clean() {
    let mut game = game();
    // With no monsters, both slots are dead ends at once.
    game.update(&typing("1"));
    game.update(&typing("s"));
    game.update(&typing("1"));
    assert_eq!(game.vitals.energy(), FULL_ENERGY - WRONG_PENALTY);
    assert!(game.typed.get(Slot::Word).is_empty());
}

#[test]
fn wrong_keys_alone_never_end_the_game() {
    let mut game = game();
    game.update(&typing(&"1".repeat(100)));
    assert_eq!(game.vitals.energy(), LOW_ENERGY);
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
    assert_eq!(game.vitals.energy(), FULL_ENERGY - COLLISION_PENALTY);
    assert!(game.world.enemies().is_empty());
    assert!(log.sfx.contains(&Sfx::Hurt));
    assert!(
        game.typed.get(Slot::Number).is_empty(),
        "what she typed is dropped"
    );
    let feedback = game.display.feedback.as_ref().expect("the pair is shown");
    assert!(feedback.text.contains(PAIRS[0].number));
    assert!(game.memory().record(&PAIRS[0]).is_some());
}

#[test]
fn the_game_ends_once_when_energy_runs_out() {
    let mut game = game();
    game.vitals = game.vitals.with_energy(COLLISION_PENALTY);
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
    game.vitals = game.vitals.with_energy(0.0);
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
    let answer = game.world.enemies()[0].answer().to_owned();
    game.update(&typing(&answer));
    let learned = game.memory().records().count();
    assert_eq!(learned, 1);
    game.vitals = game.vitals.with_energy(0.0);
    game.level = 5;

    // Nothing but Enter or a tap restarts it.
    game.update(&frame());
    assert!(game.is_over());
    let outputs = game.update(&Input {
        confirm: true,
        ..frame()
    });
    assert!(!game.is_over());
    assert_eq!(
        (game.level, game.vitals.energy(), game.vitals.score()),
        (3, FULL_ENERGY, 0)
    );
    assert_eq!(game.memory().records().count(), learned);
    assert_eq!(outputs.events, vec![GameEvent::Started { level: 3 }]);
}

#[test]
fn a_tap_also_starts_over() {
    let mut game = game();
    game.vitals = game.vitals.with_energy(0.0);
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
    let (time, pos) = (game.stage.time, game.world.enemies()[0].pos);
    let animation = game.play_time;
    let answer = game.world.enemies()[0].answer().to_owned();
    play(&mut game, 1.0, &typing(&answer));
    assert_eq!(game.stage.time, time);
    assert_eq!(game.play_time, animation, "animations freeze too");
    assert_eq!(game.world.enemies()[0].pos, pos);
    assert_eq!(game.vitals.score(), 0, "typing during a pause does nothing");

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
    game.vitals = game.vitals.with_energy(0.0);
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
    assert!(game.world.enemies().is_empty());
    let mut log = Log::default();
    log.add(game.add_points(1));
    assert!(game.stage.boss_fight);
    assert!(log.sfx.contains(&Sfx::Boss));
    let boss = &game.world.enemies()[0];
    assert!(boss.is_boss());
    assert_eq!(boss.boss.as_ref().map(|b| b.total), Some(boss_hits(1)));
}

#[test]
fn no_new_monsters_join_a_boss_fight() {
    let mut game = game();
    game.add_points(points_to_clear(1));
    idle(&mut game, 8.0);
    assert!(game.world.enemies().iter().all(Enemy::is_boss));
    assert_eq!(game.world.enemies().len(), 1);
}

#[test]
fn beating_the_boss_completes_the_level() {
    let mut game = game();
    game.add_points(points_to_clear(1));
    let mut log = Log::default();
    for _ in 0..boss_hits(1) {
        assert_eq!(
            game.world.enemies().len(),
            1,
            "the boss stays until its last hit"
        );
        let answer = game.world.enemies()[0].answer().to_owned();
        log.add(game.update(&typing(&answer)));
    }
    assert!(game.world.enemies().is_empty());
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
        game.display.banner.as_ref().map(|b| b.title.as_str()),
        Some("Taso 2!")
    );
}

/// Every answer of a boss fight, typed one after another.
fn beat_the_boss(game: &mut Rig, level: u32) -> Log {
    game.add_points(points_to_clear(level));
    let mut log = Log::default();
    for _ in 0..boss_hits(level) {
        let answer = game.world.enemies()[0].answer().to_owned();
        log.add(game.update(&typing(&answer)));
    }
    log.append(idle(game, 1.5));
    log
}

fn answered(log: &Log) -> Vec<(bool, bool, u32)> {
    log.events
        .iter()
        .filter_map(|e| match e {
            GameEvent::Answered { quick, long, combo } => Some((*quick, *long, *combo)),
            _ => None,
        })
        .collect()
}

#[test]
fn a_defeated_monster_is_reported_once_with_its_answer() {
    let mut game = game();
    with_monster(&mut game);
    let answer = game.world.enemies()[0].answer().to_owned();
    let log = play(&mut game, 1.0, &typing(&answer));
    let defeated = log
        .events
        .iter()
        .filter(|e| **e == GameEvent::MonsterDefeated)
        .count();
    assert_eq!(defeated, 1);
    assert_eq!(answered(&log).len(), 1);
}

#[test]
fn answers_in_a_row_build_a_combo_and_a_wrong_key_ends_it() {
    let mut game = game();
    let mut log = Log::default();
    for _ in 0..3 {
        with_monster(&mut game);
        let answer = game.world.enemies()[0].answer().to_owned();
        log.add(game.update(&typing(&answer)));
    }
    let combos: Vec<u32> = answered(&log).iter().map(|a| a.2).collect();
    assert_eq!(combos, [1, 2, 3]);

    // No monster's answer starts with "h", and the second key is the
    // mistake: the first only shows red.
    with_monster(&mut game);
    game.update(&typing("hh"));
    assert_eq!(game.vitals.combo(), 0);
    with_monster(&mut game);
    let answer = game.world.enemies().last().unwrap().answer().to_owned();
    let after = game.update(&typing(&answer));
    assert_eq!(
        answered(&Log {
            events: after.events,
            sfx: vec![]
        })[0]
            .2,
        1
    );
}

#[test]
fn a_hit_by_a_monster_ends_the_combo_too() {
    let mut game = game();
    with_monster(&mut game);
    let answer = game.world.enemies()[0].answer().to_owned();
    game.update(&typing(&answer));
    assert_eq!(game.vitals.combo(), 1);
    with_collision(&mut game);
    game.update(&frame());
    assert_eq!(game.vitals.combo(), 0);
}

#[test]
fn an_answer_at_once_without_a_hint_is_quick_and_a_late_one_is_not() {
    let mut game = game();
    with_monster(&mut game);
    let answer = game.world.enemies()[0].answer().to_owned();
    let quick = game.update(&typing(&answer));
    assert!(
        answered(&Log {
            events: quick.events,
            sfx: vec![]
        })[0]
            .0
    );

    with_monster(&mut game);
    game.world.enemies_mut()[0].shown_for = 10.0;
    let answer = game.world.enemies()[0].answer().to_owned();
    let slow = game.update(&typing(&answer));
    assert!(
        !answered(&Log {
            events: slow.events,
            sfx: vec![]
        })[0]
            .0
    );
}

#[test]
fn a_boss_is_not_counted_as_a_monster_but_its_hits_are_answers() {
    let mut game = game();
    let log = beat_the_boss(&mut game, 1);
    assert_eq!(answered(&log).len(), boss_hits(1));
    assert!(!log.events.contains(&GameEvent::MonsterDefeated));
}

#[test]
fn a_clean_level_is_reported_as_flawless_right_before_it_completes() {
    let mut game = game();
    let log = beat_the_boss(&mut game, 1);
    let flawless = log
        .events
        .iter()
        .position(|e| *e == GameEvent::FlawlessLevel)
        .expect("a clean level is flawless");
    assert!(matches!(
        log.events[flawless + 1],
        GameEvent::LevelCompleted { .. }
    ));
}

#[test]
fn a_wrong_key_or_a_hit_spoils_the_level_but_the_next_starts_clean() {
    let mut game = game();
    with_monster(&mut game);
    game.update(&typing("hh"));
    assert!(!game.stage.flawless);
    // Only the boss is left to answer.
    game.world.enemies_mut().clear();
    let spoiled = beat_the_boss(&mut game, 1);
    assert!(!spoiled.events.contains(&GameEvent::FlawlessLevel));
    assert!(game.stage.flawless, "the next level starts clean");

    let mut game = game_from(1);
    with_collision(&mut game);
    game.update(&frame());
    assert!(!game.stage.flawless);
}

#[test]
fn long_numbers_are_reported_as_long_answers() {
    let mut game = game_from(22);
    let mut log = Log::default();
    game.add_points(points_to_clear(22));
    for _ in 0..boss_hits(22) {
        let answer = game.world.enemies()[0].answer().to_owned();
        log.add(game.update(&typing(&answer)));
    }
    let long = answered(&log).iter().filter(|a| a.1).count();
    assert!(long > 0, "{:?}", answered(&log));
}

#[test]
fn stars_depend_on_the_energy_left() {
    let mut game = game();
    // Beating the boss gives some energy back too, so start low enough to
    // end up in the middle.
    let start = 50.0 - boss_hits(1) as f32 * BOSS_ENERGY_PER_HIT;
    game.vitals = game.vitals.with_energy(start);
    game.add_points(points_to_clear(1));
    let mut log = Log::default();
    for _ in 0..boss_hits(1) {
        let answer = game.world.enemies()[0].answer().to_owned();
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
    game.world.enemies_mut()[0].pos = game.player.pos + vec2(20.0, 0.0);
    game.update(&frame());
    assert_eq!(game.vitals.energy(), FULL_ENERGY - COLLISION_PENALTY);
    assert!(game.world.enemies()[0].pos.distance(game.player.pos) > 100.0);
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
            .world
            .enemies()
            .iter()
            .map(|e| (e.pos, e.label.clone()))
            .collect();
        format!(
            "{:?} {:?} {:?} {}",
            enemies,
            log.sfx,
            log.events,
            game.vitals.energy()
        )
    };
    assert_eq!(run(), run());
}

#[test]
fn the_arrow_keys_move_her_at_a_steady_speed() {
    let mut game = game();
    let start = game.player.pos;
    play(
        &mut game,
        0.5,
        &Input {
            arrows: vec2(1.0, 0.0),
            ..frame()
        },
    );
    assert!((game.player.pos.x - start.x - PLAYER_SPEED * 0.5).abs() < 1.0);
    assert_eq!(game.player.pos.y, start.y);
}

#[test]
fn diagonal_movement_is_no_faster() {
    let mut game = game();
    let start = game.player.pos;
    play(
        &mut game,
        0.5,
        &Input {
            arrows: vec2(1.0, 1.0),
            ..frame()
        },
    );
    let travelled = game.player.pos.distance(start);
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
    assert!(game.player.pos.x <= ARENA_W && game.player.pos.y >= 0.0);
}

#[test]
fn the_touch_stick_moves_her_by_how_far_it_is_pushed() {
    let mut game = game();
    let start = game.player.pos;
    play(
        &mut game,
        0.5,
        &Input {
            stick: vec2(0.0, -0.5),
            ..frame()
        },
    );
    assert!((start.y - game.player.pos.y - PLAYER_SPEED * 0.25).abs() < 1.0);
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
    assert!((short.player.pos.x - long.player.pos.x).abs() < 0.5);
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
    let answer = game.world.enemies()[0].answer().to_owned();
    game.update(&typing(&answer));
    let log = play(
        &mut game,
        1.0,
        &Input {
            dt: 0.25,
            ..frame()
        },
    );
    assert!(game.world.spells().is_empty());
    assert!(log.sfx.contains(&Sfx::Explode));
}

#[test]
fn enemies_keep_their_ids_when_others_leave() {
    let mut game = game();
    for x in [50.0, 150.0, 250.0] {
        let mut enemy = Enemy::new(Question::single(PAIRS[x as usize / 100]), false, 0.5, 0.0);
        enemy.pos = vec2(x, 50.0);
        game.world.admit(enemy);
    }
    let ids: Vec<EnemyId> = game.world.enemies().iter().map(|e| e.id).collect();
    assert!(ids[0] != ids[1] && ids[1] != ids[2] && ids[0] != ids[2]);

    game.hit_enemy(ids[0]);
    let left: Vec<EnemyId> = game.world.enemies().iter().map(|e| e.id).collect();
    assert_eq!(left, [ids[1], ids[2]]);
    assert!(game.world.enemy(ids[0]).is_none());
    assert_eq!(game.world.enemy(ids[2]).map(|e| e.pos.x), Some(250.0));

    // Hitting one that is already gone does nothing.
    game.hit_enemy(ids[0]);
    assert_eq!(game.world.enemies().len(), 2);
}

/// Plays like a perfect typist who never moves: each frame, types the
/// answer to the first monster on screen. Stops after `max_seconds` or
/// once `done` says so; returns everything that happened.
fn autoplay(game: &mut Rig, max_seconds: f32, done: impl Fn(&Game) -> bool) -> Log {
    let mut log = Log::default();
    for _ in 0..(max_seconds / FRAME) as usize {
        let input = match game.world.enemies().first() {
            Some(enemy) => typing(enemy.answer()),
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
    let log = autoplay(&mut game, 600.0, |g| g.level == 5);

    assert_eq!(game.level, 5, "four levels are cleared within ten minutes");
    assert_eq!(completed_levels(&log), [1, 2, 3, 4]);
    assert_eq!(log.events.first(), Some(&GameEvent::Started { level: 1 }));
    assert!(
        !log.events
            .iter()
            .any(|e| matches!(e, GameEvent::Over { .. })),
        "a perfect typist never loses"
    );
    // Defeating monsters builds energy up beyond 100%, never past the cap.
    let energy = game.vitals.energy();
    assert!(energy > FULL_ENERGY && energy <= energy_cap(game.level));
    assert!(log.sfx.contains(&Sfx::Boss), "every level ends in a boss");
}

#[test]
fn the_first_long_number_level_can_be_cleared() {
    let start = long_numbers::first_long_level();
    let mut game = game_from(start);
    let log = autoplay(&mut game, 600.0, |g| g.level > start);

    assert_eq!(completed_levels(&log), [start]);
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
fn play_at(fps: f32, seconds: f32) -> Rig {
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
    assert!(!reference.world.enemies().is_empty(), "something happened");
    for fps in [30.0, 120.0, 240.0] {
        let other = play_at(fps, 20.0);
        assert_eq!(other.player, reference.player, "{fps} fps");
        assert_eq!(other.play_time, reference.play_time, "{fps} fps");
        assert_eq!(
            other.world.enemies().len(),
            reference.world.enemies().len(),
            "{fps} fps"
        );
        for (a, b) in other.world.enemies().iter().zip(reference.world.enemies()) {
            assert_eq!(a.question.number(false), b.question.number(false));
            assert_eq!(a.pos, b.pos, "{fps} fps");
        }
    }
}

#[test]
fn typing_on_a_frame_too_short_for_a_step_is_not_lost() {
    let mut game = game();
    with_monster(&mut game);
    let answer = game.world.enemies()[0].answer().to_owned();
    // A frame at 1000 fps is a fraction of a step.
    let short = |input: Input| Input { dt: 0.001, ..input };
    for c in answer.chars() {
        game.update(&short(typing(&c.to_string())));
    }
    idle(&mut game, 0.1);
    assert_eq!(game.vitals.score(), STAR_POINTS);
}
