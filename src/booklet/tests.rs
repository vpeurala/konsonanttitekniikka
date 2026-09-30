use super::*;
use crate::pairs::PAIRS;

/// Assets with recognisable stand-ins, so tests can tell which picture went where.
fn fake_assets() -> Assets {
    let uri = |name: &str| format!("data:image/png;base64,{name}");
    Assets {
        nunito_regular: "data:font/ttf;base64,R".into(),
        nunito_bold: "data:font/ttf;base64,B".into(),
        fredoka: "data:font/ttf;base64,F".into(),
        pictures: (0..PAIR_COUNT)
            .map(|i| uri(&format!("picture{i}x")))
            .collect(),
        characters: Characters {
            girl: uri("girl"),
            girl_casting: uri("girlcasting"),
            monster: uri("monster"),
            cyclops: uri("cyclops"),
            boss: uri("boss"),
            portal: uri("portal"),
            star: uri("star"),
        },
    }
}

#[test]
fn base64_matches_the_standard_examples() {
    // From RFC 4648.
    for (plain, encoded) in [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
    ] {
        assert_eq!(base64(plain.as_bytes()), encoded);
    }
}

#[test]
fn escaping_covers_the_html_special_characters() {
    assert_eq!(
        escape("<a href=\"x\">&"),
        "&lt;a href=&quot;x&quot;&gt;&amp;"
    );
}

/// The card the pair pages show for `pair`, with the stand-in picture.
fn card_of(pair: &crate::pairs::Pair) -> String {
    let digit = pair.number.as_bytes()[0] - b'0';
    format!(
        "<div class=\"card d{digit}\"><img class=\"pic\" src=\"data:image/png;base64,picture{}x\" alt=\"{}\">\
         <div class=\"num\">{}</div><div class=\"word\">{}</div></div>",
        pair.id.index(),
        pair.word,
        pair.number,
        pair.word
    )
}

#[test]
fn every_pair_is_listed_once_with_its_own_picture_number_and_word() {
    let html = html(&fake_assets());
    for pair in PAIRS {
        assert_eq!(
            html.matches(&card_of(pair)).count(),
            1,
            "the card for {}",
            pair.word
        );
    }
}

#[test]
fn the_consonant_table_is_the_games() {
    let html = html(&fake_assets());
    for (digit, consonant) in DIGIT_CONSONANTS.iter().enumerate() {
        let upper: String = consonant.to_uppercase().collect();
        let cell = format!(
            "<div class=\"digit d{digit}\"><span class=\"n\">{digit}</span><span class=\"eq\">=</span><span class=\"l\">{upper}</span></div>"
        );
        assert!(html.contains(&cell), "the table lacks {digit} = {upper}");
    }
}

#[test]
fn letter_tiles_colour_consonants_and_grey_vowels() {
    let tiles = letter_tiles("keko");
    assert_eq!(tiles.matches("tile d2").count(), 2);
    assert_eq!(tiles.matches("tile vowel").count(), 2);
    assert!(tiles.contains("<b>K</b><i>2</i>"));
}

#[test]
fn the_booklet_has_a_whole_number_of_sheets() {
    // Sixteen pages fold and staple as a booklet.
    let html = html(&fake_assets());
    let pages = html.matches("<section class=\"page").count();
    assert_eq!(pages % 4, 0, "{pages} pages");
    assert_eq!(pages, 16);
}

#[test]
fn every_embedded_picture_is_used() {
    let html = html(&fake_assets());
    for i in 0..PAIR_COUNT {
        assert!(
            html.contains(&format!("picture{i}x")),
            "picture {i} is missing"
        );
    }
    for name in [
        "girl",
        "girlcasting",
        "monster",
        "cyclops",
        "boss",
        "portal",
        "star",
    ] {
        assert!(
            html.contains(&format!("base64,{name})")),
            "{name} is not defined"
        );
    }
}

#[test]
fn nothing_is_left_unfilled() {
    let html = html(&fake_assets());
    assert!(!html.contains("{}"), "an unfilled placeholder");
    assert!(!html.contains("{{"), "an unescaped brace");
    assert!(html.contains("lang=\"fi\""));
    assert!(html.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn the_examples_are_worked_out_by_the_games_own_rules() {
    let html = html(&fake_assets());
    // Long numbers split greedily, two digits at a time.
    for split in ["201 → 20 1", "1377 → 13 77", "12345 → 12 34 5"] {
        assert!(html.contains(split), "no example {split}");
    }
    // 22 is keko, and its letters are shown as tiles.
    let keko = crate::pairs::find("22").unwrap();
    assert_eq!(keko.word, "keko");
    assert!(html.contains(&letter_tiles(keko.word)));
}

#[test]
fn the_level_numbers_come_from_the_game() {
    let html = html(&fake_assets());
    let first_long = crate::long_numbers::first_long_level();
    assert!(html.contains(&format!("tasolla {}.", first_long - 1)));
    assert!(html.contains(&format!("Tasolta {first_long} alkaen")));
    assert!(html.contains(&format!("{PAIR_COUNT} paria")));
}

#[test]
fn the_page_about_badges_gives_their_number_from_the_game() {
    let html = html(&fake_assets());
    assert!(html.contains(&format!(
        "Pelissä on {} kunniamerkkiä",
        crate::badges::BADGES.len()
    )));
}
