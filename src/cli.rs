//! What the program was asked to do on the command line. Parsing is pure;
//! `main` carries the command out.

/// What to do: play, or make one of the files the project is built from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Open the game.
    Play,
    /// `--render-icons`: regenerate the app icon files.
    RenderIcons,
    /// `--render-music [FILE.wav]`: write the music loop to a file, for
    /// listening to it outside the game.
    RenderMusic { path: String },
    /// `--render-booklet [FILE.html]`: write the user instruction booklet,
    /// which `scripts/booklet.sh` turns into a PDF.
    RenderBooklet { path: String },
}

impl Command {
    /// The command in `args`, the program's arguments after its name.
    pub fn parse(args: &[String]) -> Command {
        let path_after = |flag: &str, default: &str| {
            args.iter()
                .position(|a| a == flag)
                .map(|i| args.get(i + 1).map_or(default, String::as_str).to_owned())
        };
        if args.iter().any(|a| a == "--render-icons") {
            Command::RenderIcons
        } else if let Some(path) = path_after("--render-music", "music.wav") {
            Command::RenderMusic { path }
        } else if let Some(path) = path_after("--render-booklet", "opas.html") {
            Command::RenderBooklet { path }
        } else {
            Command::Play
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Command {
        Command::parse(&args.iter().map(|a| (*a).to_owned()).collect::<Vec<_>>())
    }

    #[test]
    fn no_arguments_play_the_game() {
        assert_eq!(parse(&[]), Command::Play);
        assert_eq!(parse(&["--unknown"]), Command::Play);
    }

    #[test]
    fn render_commands_take_a_path_or_use_a_default() {
        assert_eq!(parse(&["--render-icons"]), Command::RenderIcons);
        assert_eq!(
            parse(&["--render-music", "loop.wav"]),
            Command::RenderMusic {
                path: "loop.wav".to_owned()
            }
        );
        assert_eq!(
            parse(&["--render-music"]),
            Command::RenderMusic {
                path: "music.wav".to_owned()
            }
        );
        assert_eq!(
            parse(&["--render-booklet", "b.html"]),
            Command::RenderBooklet {
                path: "b.html".to_owned()
            }
        );
        assert_eq!(
            parse(&["--render-booklet"]),
            Command::RenderBooklet {
                path: "opas.html".to_owned()
            }
        );
    }
}
