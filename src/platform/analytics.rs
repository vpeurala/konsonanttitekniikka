//! Anonymous counts of how the game is played, for the web version's
//! statistics at GoatCounter: games started, levels completed and so on.
//! Nothing identifies the player. The apps send nothing.

/// Something worth counting. The app describes what happened; the paths
/// and titles GoatCounter shows are made here, at the edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// The flash card practice was opened.
    Practice,
    /// The progress map was opened.
    Progress,
    /// The badge screen was opened.
    Badges,
    /// A game started (or started again) from this level.
    GameStarted { level: u32 },
    /// A game ended on this level.
    GameOver { level: u32 },
    /// A level was finished with this many stars.
    LevelCleared { level: u32, stars: u8 },
    /// A badge was earned.
    BadgeEarned {
        id: &'static str,
        name: &'static str,
    },
}

#[cfg_attr(
    not(target_arch = "wasm32"),
    allow(dead_code, reason = "only the web version sends counts")
)]
impl Event {
    /// The path the count is filed under, like "taso-lapaisty/5".
    pub fn path(&self) -> String {
        match self {
            Event::Practice => "harjoittelu".to_owned(),
            Event::Progress => "edistyminen".to_owned(),
            Event::Badges => "kunniamerkit".to_owned(),
            Event::GameStarted { level } => format!("peli-alkoi/taso-{level}"),
            Event::GameOver { level } => format!("peli-paattyi/taso-{level}"),
            Event::LevelCleared { level, .. } => format!("taso-lapaisty/{level}"),
            Event::BadgeEarned { id, .. } => format!("merkki/{id}"),
        }
    }

    /// The Finnish title shown for the path in the statistics.
    pub fn title(&self) -> String {
        match self {
            Event::Practice => "Harjoittelu".to_owned(),
            Event::Progress => "Edistyminen".to_owned(),
            Event::Badges => "Kunniamerkit".to_owned(),
            Event::GameStarted { level } => format!("Peli alkoi tasolta {level}"),
            Event::GameOver { level } => format!("Peli päättyi tasolla {level}"),
            Event::LevelCleared { level, stars } => {
                format!("Taso {level} läpäisty ({stars} tähteä)")
            }
            Event::BadgeEarned { name, .. } => format!("Kunniamerkki: {name}"),
        }
    }
}

/// Sends the count to the statistics.
pub fn send(event: &Event) {
    #[cfg(target_arch = "wasm32")]
    crate::platform::web::event(&event.path(), &event.title());
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_and_titles_are_what_the_statistics_have_always_shown() {
        let cleared = Event::LevelCleared { level: 5, stars: 2 };
        assert_eq!(cleared.path(), "taso-lapaisty/5");
        assert_eq!(cleared.title(), "Taso 5 läpäisty (2 tähteä)");
        assert_eq!(Event::GameStarted { level: 3 }.path(), "peli-alkoi/taso-3");
        assert_eq!(Event::GameOver { level: 7 }.path(), "peli-paattyi/taso-7");
        let badge = Event::BadgeEarned {
            id: "monsters-1",
            name: "Ensimmäinen",
        };
        assert_eq!(badge.path(), "merkki/monsters-1");
        assert_eq!(badge.title(), "Kunniamerkki: Ensimmäinen");
        assert_eq!(Event::Practice.path(), "harjoittelu");
        assert_eq!(Event::Progress.title(), "Edistyminen");
        assert_eq!(Event::Badges.path(), "kunniamerkit");
    }
}
