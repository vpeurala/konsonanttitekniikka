//! Anonymous counts of how the game is played, for the web version's
//! statistics at GoatCounter: games started, levels completed and so on.
//! Nothing identifies the player. The apps send nothing. Sending is the
//! shell's job.

/// Something worth counting. The app describes what happened, and in which
/// mode (`hardcore` mode has no hints); the paths and titles GoatCounter
/// shows are made here, at the edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// The flash card practice was opened.
    Practice { hardcore: bool },
    /// The progress map was opened.
    Progress { hardcore: bool },
    /// The badge screen was opened.
    Badges { hardcore: bool },
    /// A game started (or started again) from this level.
    GameStarted { level: u32, hardcore: bool },
    /// A game ended on this level.
    GameOver { level: u32, hardcore: bool },
    /// A level was finished with this many stars.
    LevelCleared {
        level: u32,
        stars: u8,
        hardcore: bool,
    },
    /// A badge was earned.
    BadgeEarned {
        id: &'static str,
        name: &'static str,
        hardcore: bool,
    },
}

impl Event {
    /// Whether it happened in hardcore mode.
    pub fn hardcore(&self) -> bool {
        match *self {
            Event::Practice { hardcore }
            | Event::Progress { hardcore }
            | Event::Badges { hardcore }
            | Event::GameStarted { hardcore, .. }
            | Event::GameOver { hardcore, .. }
            | Event::LevelCleared { hardcore, .. }
            | Event::BadgeEarned { hardcore, .. } => hardcore,
        }
    }

    /// The path the count is filed under, like "taso-lapaisty/5". Hardcore
    /// mode's counts have "ankara" after the first part, like
    /// "taso-lapaisty/ankara/5", so the easy mode's paths are as they
    /// always were.
    pub fn path(&self) -> String {
        let path = match self {
            Event::Practice { .. } => "harjoittelu".to_owned(),
            Event::Progress { .. } => "edistyminen".to_owned(),
            Event::Badges { .. } => "kunniamerkit".to_owned(),
            Event::GameStarted { level, .. } => format!("peli-alkoi/taso-{level}"),
            Event::GameOver { level, .. } => format!("peli-paattyi/taso-{level}"),
            Event::LevelCleared { level, .. } => format!("taso-lapaisty/{level}"),
            Event::BadgeEarned { id, .. } => format!("merkki/{id}"),
        };
        if !self.hardcore() {
            return path;
        }
        match path.split_once('/') {
            Some((first, rest)) => format!("{first}/ankara/{rest}"),
            None => format!("{path}/ankara"),
        }
    }

    /// The Finnish title shown for the path in the statistics.
    pub fn title(&self) -> String {
        let title = match self {
            Event::Practice { .. } => "Harjoittelu".to_owned(),
            Event::Progress { .. } => "Edistyminen".to_owned(),
            Event::Badges { .. } => "Kunniamerkit".to_owned(),
            Event::GameStarted { level, .. } => format!("Peli alkoi tasolta {level}"),
            Event::GameOver { level, .. } => format!("Peli päättyi tasolla {level}"),
            Event::LevelCleared { level, stars, .. } => {
                format!("Taso {level} läpäisty ({stars} tähteä)")
            }
            Event::BadgeEarned { name, .. } => format!("Kunniamerkki: {name}"),
        };
        if self.hardcore() {
            format!("{title} (ankara tila)")
        } else {
            title
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The same event in both modes: (easy, hardcore).
    fn both(make: impl Fn(bool) -> Event) -> (Event, Event) {
        (make(false), make(true))
    }

    #[test]
    fn paths_and_titles_are_what_the_statistics_have_always_shown() {
        let (cleared, _) = both(|hardcore| Event::LevelCleared {
            level: 5,
            stars: 2,
            hardcore,
        });
        assert_eq!(cleared.path(), "taso-lapaisty/5");
        assert_eq!(cleared.title(), "Taso 5 läpäisty (2 tähteä)");
        let (started, _) = both(|hardcore| Event::GameStarted { level: 3, hardcore });
        assert_eq!(started.path(), "peli-alkoi/taso-3");
        let (over, _) = both(|hardcore| Event::GameOver { level: 7, hardcore });
        assert_eq!(over.path(), "peli-paattyi/taso-7");
        let (badge, _) = both(|hardcore| Event::BadgeEarned {
            id: "monsters-1",
            name: "Ensimmäinen",
            hardcore,
        });
        assert_eq!(badge.path(), "merkki/monsters-1");
        assert_eq!(badge.title(), "Kunniamerkki: Ensimmäinen");
        let (practice, _) = both(|hardcore| Event::Practice { hardcore });
        assert_eq!(practice.path(), "harjoittelu");
        let (progress, _) = both(|hardcore| Event::Progress { hardcore });
        assert_eq!(progress.title(), "Edistyminen");
        let (badges, _) = both(|hardcore| Event::Badges { hardcore });
        assert_eq!(badges.path(), "kunniamerkit");
    }

    #[test]
    fn hardcore_mode_is_counted_under_paths_of_its_own() {
        let paths = |make: fn(bool) -> Event| make(true).path();
        assert_eq!(
            paths(|hardcore| Event::GameStarted { level: 3, hardcore }),
            "peli-alkoi/ankara/taso-3"
        );
        assert_eq!(
            paths(|hardcore| Event::GameOver { level: 7, hardcore }),
            "peli-paattyi/ankara/taso-7"
        );
        assert_eq!(
            paths(|hardcore| Event::LevelCleared {
                level: 5,
                stars: 2,
                hardcore
            }),
            "taso-lapaisty/ankara/5"
        );
        assert_eq!(
            paths(|hardcore| Event::BadgeEarned {
                id: "monsters-1",
                name: "Ensimmäinen",
                hardcore
            }),
            "merkki/ankara/monsters-1"
        );
        assert_eq!(
            paths(|hardcore| Event::Practice { hardcore }),
            "harjoittelu/ankara"
        );
        assert_eq!(
            paths(|hardcore| Event::Progress { hardcore }),
            "edistyminen/ankara"
        );
        assert_eq!(
            paths(|hardcore| Event::Badges { hardcore }),
            "kunniamerkit/ankara"
        );
    }

    #[test]
    fn hardcore_titles_say_so() {
        let hard = Event::GameStarted {
            level: 3,
            hardcore: true,
        };
        assert_eq!(hard.title(), "Peli alkoi tasolta 3 (ankara tila)");
        let badge = Event::BadgeEarned {
            id: "monsters-1",
            name: "Ensimmäinen",
            hardcore: true,
        };
        assert_eq!(badge.title(), "Kunniamerkki: Ensimmäinen (ankara tila)");
    }
}
