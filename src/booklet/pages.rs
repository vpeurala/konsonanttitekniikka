//! The booklet's sixteen pages, in Finnish. Everything that names a number,
//! a word, a consonant or a level is looked up from the game's own data,
//! never typed in, so the booklet can't disagree with the game.

use super::{Assets, Page, escape, letter_tiles};
use crate::badges::BADGES;
use crate::curriculum::NEW_PAIRS_PER_LEVEL;
use crate::long_numbers::{Question, first_long_level};
use crate::pairs::{DIGIT_CONSONANTS, PAIR_COUNT, PAIRS, Pair, VOWELS, find};
use lukuloitsu_core::levels::LEVELS_PER_CHECKPOINT;

pub fn pages(assets: &Assets) -> Vec<Page> {
    let mut pages = vec![
        cover(),
        introduction(assets),
        consonants(assets),
        number_and_word(assets),
        long_numbers(assets),
        why_finnish(),
        how_to_play(assets),
        levels_and_practice(),
    ];
    pages.extend(pair_pages(assets));
    pages.push(exercises());
    pages.push(back_cover());
    pages
}

/// The pair for a number that the text needs, like "22".
fn pair(number: &str) -> Pair {
    find(number).unwrap_or_else(|| panic!("the booklet needs the pair {number}"))
}

/// A digit's consonant, uppercase, like "K" for 2.
fn letter(digit: usize) -> String {
    DIGIT_CONSONANTS[digit].to_uppercase().collect()
}

/// The word of a pair in capitals.
fn caps(pair: Pair) -> String {
    pair.word.to_uppercase()
}

/// A character picture placed on a page: `class` picks the character, and
/// `place` is the CSS that puts it somewhere.
fn character(class: &str, place: &str) -> String {
    format!("<div class=\"ch {class}\" style=\"{place}\"></div>")
}

/// Small stars and dots scattered over a dark page. A fixed recipe, so the
/// booklet comes out the same every time.
fn sparkles(count: usize, from_mm: f32, to_mm: f32) -> String {
    let mut state: u32 = 12345;
    let mut next = move || {
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
        (state >> 8) as f32 / 16_777_216.0
    };
    (0..count)
        .map(|_| {
            let (x, y, size, alpha) = (next() * 205.0, from_mm + next() * (to_mm - from_mm), 0.6 + next() * 1.8, 0.25 + next() * 0.6);
            format!(
                "<div class=\"spark\" style=\"left:{x:.1}mm;top:{y:.1}mm;width:{size:.1}mm;height:{size:.1}mm;opacity:{alpha:.2}\"></div>"
            )
        })
        .collect()
}

/// A pair's picture as an `<img>`; the image itself is the caller's.
fn picture(assets: &Assets, pair: Pair, class: &str) -> String {
    format!(
        "<img class=\"{class}\" src=\"{}\" alt=\"{}\">",
        assets.pictures[pair.id.index()],
        escape(pair.word)
    )
}

/// A pair as a card: picture, number and word.
fn card(assets: &Assets, pair: Pair, color_digit: usize) -> String {
    format!(
        "<div class=\"card d{color_digit}\">{}<div class=\"num\">{}</div><div class=\"word\">{}</div></div>",
        picture(assets, pair, "pic"),
        escape(pair.number),
        escape(pair.word)
    )
}

/// The colour of a single-digit pair's own digit, or of a two-digit pair's
/// first digit.
fn first_digit(pair: Pair) -> usize {
    pair.number.as_bytes()[0] as usize - b'0' as usize
}

/// "koho", "koho ja jää" or "koho, jää ja puu".
fn join_words(words: &[&str]) -> String {
    match words {
        [] => String::new(),
        [only] => (*only).to_string(),
        [rest @ .., last] => format!("{} ja {last}", rest.join(", ")),
    }
}

/// A small star that sits in a line of text.
fn inline_star() -> &'static str {
    "<span class=\"ch star\" style=\"position:static;display:inline-block;width:6mm;height:6mm;vertical-align:middle\"></span>"
}

fn cover() -> Page {
    let body = format!(
        "{}<div class=\"cover-title\">Lukuloitsu</div>\
         <div class=\"cover-sub\">Opas konsonanttitekniikkaan</div>{}{}{}{}{}{}{}{}{}\
         <div class=\"cover-note\">Opi muistamaan mikä tahansa luku sanojen ja kuvien avulla<br>Versio {}</div>",
        sparkles(70, 0.0, 292.0),
        character("portal", "left:38mm;top:150mm;width:134mm;height:134mm"),
        character("cyclops", "right:14mm;top:92mm;width:84mm;height:84mm"),
        character("monster", "left:10mm;top:80mm;width:58mm;height:58mm"),
        character("boss", "left:64mm;top:68mm;width:82mm;height:82mm"),
        character(
            "girl-casting",
            "left:8mm;top:168mm;width:100mm;height:100mm"
        ),
        character("star", "left:150mm;top:14mm;width:22mm;height:22mm"),
        character("star", "left:24mm;top:24mm;width:14mm;height:14mm"),
        character("star", "right:10mm;top:190mm;width:16mm;height:16mm"),
        character("star", "left:176mm;top:120mm;width:10mm;height:10mm"),
        env!("CARGO_PKG_VERSION"),
    );
    Page {
        class: "dark",
        body,
    }
}

fn introduction(assets: &Assets) -> Page {
    let example = pair("22");
    let body = format!(
        "<h1>Mikä on konsonanttitekniikka?</h1>\
         <p>Oletko joskus yrittänyt muistaa pitkän numeron, vaikkapa puhelinnumeron tai vuosiluvun? \
         Numeroita on vaikea muistaa, koska ne eivät kuvaa mitään. Sanat ja kuvat ovat paljon helpompia: \
         koiran tai lohikäärmeen kuvan muistaa heti.</p>\
         <p><b>Konsonanttitekniikka</b> on muistisääntö, joka muuttaa numerot sanoiksi. \
         Kun numerosta tulee sana, siitä voi kuvitella kuvan, ja kuvan muistaa paljon paremmin kuin numeron.</p>\
         <p>Tekniikka on suomalainen versio Major-järjestelmästä (englanniksi <i>Major system</i>), \
         joka on yli 300 vuotta vanha muistitemppu. Nyt sinäkin voit oppia sen.</p>\
         <div class=\"steps\">\
           <div class=\"step\"><div class=\"no\">1</div><h3>Numerosta konsonantti</h3>\
             <p>Jokaista numeroa vastaa yksi konsonantti. Esimerkiksi numero 2 on {k}.</p></div>\
           <div class=\"step\"><div class=\"no\">2</div><h3>Konsonanteista sana</h3>\
             <p>Vokaalit täyttävät välit: kaksi {k}-kirjainta ja vokaaleja on <b>{word}</b>.</p></div>\
           <div class=\"step\"><div class=\"no\">3</div><h3>Sanasta kuva</h3>\
             <p>Kuvittele {word} mielessäsi. Kun näet sen, muistat luvun {number}.</p></div>\
         </div>\
         <div class=\"example\">{}<p><b>{}</b> = {}</p></div>\
         <div class=\"box gold\"><h3>Mitä Lukuloitsu opettaa?</h3>\
           <p>Lukuloitsu opettaa {PAIR_COUNT} valmista paria: luvut 0–9 ja 00–99. \
           Jokaisella luvulla on oma sana ja oma kuva. Kun osaat parit, voit muuttaa minkä tahansa luvun \
           sanoiksi, ja sanat kuviksi.</p></div>{}{}",
        picture(assets, example, "pic"),
        example.word,
        example.number,
        character("girl", "right:10mm;bottom:14mm;width:88mm;height:88mm"),
        character("monster", "right:14mm;top:11mm;width:26mm;height:26mm"),
        k = letter(2),
        word = example.word,
        number = example.number,
    );
    Page { class: "", body }
}

fn consonants(assets: &Assets) -> Page {
    let digits: String = (0..10)
        .map(|d| {
            format!(
                "<div class=\"digit d{d}\"><span class=\"n\">{d}</span><span class=\"eq\">=</span>\
                 <span class=\"l\">{}</span></div>",
                letter(d)
            )
        })
        .collect();
    let vowels = VOWELS.iter().map(|c| c.to_string()).collect::<Vec<_>>();
    let example_row = |number: &str| {
        let p = pair(number);
        format!(
            "<div class=\"box row compact\">{}<div style=\"flex:1\"><h3>{}</h3>{}\
             <div class=\"result\">{}</div></div></div>",
            picture(assets, p, "pic"),
            caps(p),
            letter_tiles(p.word),
            p.number
        )
    };
    let body = format!(
        "<h1>Jokainen numero on konsonantti</h1>\
         <p>Jokaista numeroa vastaa yksi konsonantti. Opettele ensin tämä taulukko:</p>\
         <div class=\"digits\">{digits}</div>\
         <p>Vokaalit eli {} ovat pelkkää täytettä. Ne eivät tarkoita mitään, mutta niiden avulla \
         konsonanteista saa sanoja. Muita kirjaimia ei käytetä.</p>\
         <h2>Jokainen konsonantti lasketaan</h2>\
         <p>Katso sanan konsonantteja yksi kerrallaan. Sanassa {} on kaksi konsonanttia, {} ja {}. \
         Koska {} on {}, sana tarkoittaa lukua {}. Vokaalit ovat harmaina: niitä ei lasketa.</p>\
         {}{}{}\
         <div class=\"box gold\"><h3>Ei tarvitse opetella ulkoa heti</h3>\
         <p>Taulukko tulee tutuksi itsestään, kun kirjoitat lukuja ja sanoja yhä uudelleen. \
         Peli auttaa sinua matkan varrella.</p></div>{}",
        join_words(&vowels.iter().map(String::as_str).collect::<Vec<_>>()),
        caps(pair("22")),
        letter(2),
        letter(2),
        letter(2),
        2,
        pair("22").number,
        example_row("22"),
        example_row("44"),
        example_row("77"),
        character("cyclops", "right:12mm;top:9mm;width:28mm;height:28mm"),
    );
    Page { class: "", body }
}

fn number_and_word(assets: &Assets) -> Page {
    let to_word = pair("13");
    let to_number = pair("37");
    let digit = |p: Pair, i: usize| (p.number.as_bytes()[i] - b'0') as usize;
    let body = format!(
        "<h1>Numerosta sanaksi ja takaisin</h1>\
         <div class=\"cols\">\
           <div class=\"box blue\"><h3>Numerosta sanaksi</h3>\
             <p>Otetaan luku <b>{}</b>. Numero {} on {} ja numero {} on {}. Kun väliin laitetaan vokaaleja, \
             tulee sana <b>{}</b>.</p>{}</div>\
           <div class=\"box green\"><h3>Sanasta numeroksi</h3>\
             <p>Otetaan sana <b>{}</b>. Sen konsonantit ovat {} ja {}. Koska {} on {} ja {} on {}, \
             sana tarkoittaa lukua <b>{}</b>.</p>{}</div>\
         </div>\
         <p>Pelissä sinun ei tarvitse keksiä sanoja itse: jokaiselle luvulle on valmis sana. \
         Sinun tarvitsee vain muistaa se.</p>\
         <h2>Yksi luku, yksi sana</h2>\
         <p>Jokaista lukua vastaa vain yksi oikea sana. Peli hyväksyy vain sen, koska muuten sanat \
         sekoittuisivat päässä. Kun näet luvun {}, mielessäsi on aina {}.</p>\
         <div class=\"box pink\"><h3>Varo nollia!</h3>\
           <p>Luku {} ja luku {} ovat eri lukuja. Luku {} on {}, mutta luku {} on {}. \
           Ensimmäinen nolla on {} ja se kuuluu sanaan. Samoin {} on {} ja {} on {}.</p>\
           <div style=\"display:flex;gap:4mm;justify-content:center\">{}{}{}{}</div></div>{}",
        to_word.number,
        digit(to_word, 0),
        letter(digit(to_word, 0)),
        digit(to_word, 1),
        letter(digit(to_word, 1)),
        caps(to_word),
        letter_tiles(to_word.word),
        caps(to_number),
        letter(digit(to_number, 0)),
        letter(digit(to_number, 1)),
        letter(digit(to_number, 0)),
        digit(to_number, 0),
        letter(digit(to_number, 1)),
        digit(to_number, 1),
        to_number.number,
        letter_tiles(to_number.word),
        to_number.number,
        to_number.word,
        pair("6").number,
        pair("06").number,
        pair("6").number,
        pair("6").word,
        pair("06").number,
        pair("06").word,
        letter(0),
        pair("0").number,
        pair("0").word,
        pair("00").number,
        pair("00").word,
        mini(assets, pair("0")),
        mini(assets, pair("00")),
        mini(assets, pair("6")),
        mini(assets, pair("06")),
        character(
            "girl-casting",
            "right:8mm;bottom:14mm;width:84mm;height:84mm"
        ),
    );
    Page { class: "", body }
}

/// A small card for examples: picture, number and word.
fn mini(assets: &Assets, p: Pair) -> String {
    format!(
        "<div class=\"card mini d{}\">{}<div class=\"num\">{}</div><div class=\"word\">{}</div></div>",
        first_digit(p),
        picture(assets, p, "pic"),
        p.number,
        p.word
    )
}

fn long_numbers(assets: &Assets) -> Page {
    let example = |digits: &str| {
        let question = Question::for_number(digits).expect("an example of digits");
        let cards: String = question
            .pairs()
            .iter()
            .map(|p| mini(assets, *p))
            .collect::<Vec<_>>()
            .join("<div class=\"plus\">+</div>");
        format!(
            "<div class=\"box compact\"><h3>{digits} → {}</h3>\
             <div style=\"display:flex;align-items:center;justify-content:center;gap:3mm\">{cards}</div></div>",
            question.number(true),
        )
    };
    let story = Question::for_number("1377").expect("an example of digits");
    let story_words: Vec<&str> = story.pairs().iter().map(|p| p.word).collect();
    let body = format!(
        "<h1>Pitkät luvut</h1>\
         <p>Kun luku on pitkä, se pilkotaan kahden numeron paloihin vasemmalta alkaen. \
         Jokaisesta palasta tulee yksi sana. Jos yksi numero jää viimeiseksi yli, sillä on oma yhden numeron sanansa.</p>\
         <ol><li>Pilko luku kahden numeron paloihin vasemmalta.</li>\
         <li>Jos numero jää yli, se on oma palansa.</li>\
         <li>Muuta jokainen pala sanaksi ja kuvittele sanat samaan kuvaan.</li></ol>\
         {}{}{}\
         <div class=\"box gold\"><h3>Mitä hullumpi, sitä parempi</h3>\
         <p>Kuvittele vaikka {} samassa kuvassa ja keksi niille pieni tarina. Tarinan ei tarvitse olla järkevä. \
         Päinvastoin: outo ja hauska kuva jää mieleen paremmin kuin tavallinen. Kun tarina on valmis, luku on tallessa.</p></div>\
         <p>Pelissä pitkät luvut tulevat vastaan tasolta {}, kun kaikki parit on jo tavattu.</p>{}",
        example("201"),
        example("1377"),
        example("12345"),
        join_words(&story_words),
        first_long_level(),
        character("boss", "right:12mm;top:5mm;width:27mm;height:27mm"),
    );
    Page { class: "", body }
}

fn why_finnish() -> Page {
    let body = format!(
        "<h1>Miksi suomi sopii tähän?</h1>\
         <p>Suomea kirjoitetaan niin kuin se lausutaan: yksi kirjain on yksi äänne. Siksi konsonanttitekniikan \
         säännöt ovat suomeksi helppoja. Ei tarvitse miettiä, miten sana äännetään, vaan riittää katsoa \
         sen konsonanttikirjaimia.</p>\
         <p>Monessa muussa kielessä tämä ei onnistu. Englannin sana <i>knee</i> alkaa k-kirjaimella, \
         mutta k ei kuulu. Siksi englanninkielinen Major-järjestelmä lasketaan äänteiden eikä \
         kirjainten mukaan, ja sen taulukko on toisenlainen kuin tämä.</p>\
         <p>Suomessa on myös paljon vokaaleja, ja se auttaa: vokaalit ovat täytettä, joten konsonanteista \
         saa helposti sanoja.</p>\
         <div class=\"box blue\"><h3>Sanat on valittu huolella</h3>\
         <p>Jokaisessa sanassa on vain taulukon konsonantteja, eikä esimerkiksi n:ää tai d:tä. Jokainen sana on \
         jotain, minkä voi kuvitella mielessään: eläin, esine, ruoka tai paikka. Siksi jokaiselle sanalle \
         on myös oma kuva.</p></div>\
         <h2>Mihin tätä voi käyttää?</h2>\
         <ul>\
         <li><b>Puhelinnumerot.</b> Numerosta tulee tarina, jonka muistaa.</li>\
         <li><b>Vuosiluvut ja päivämäärät.</b> Historian koe helpottuu, kun vuosiluvut ovat kuvia.</li>\
         <li><b>Huoneiden, bussien ja kaappien numerot.</b></li>\
         <li><b>Koodit.</b> Pidä salaiset koodit kuitenkin vain omana tietonasi: älä kerro niitä kenellekään.</li>\
         <li><b>Pitkät luvut</b>, kuten piin (π) desimaalit. Muistiurheilijat opettelevat niitä satoja!</li>\
         </ul>\
         <div class=\"box pink\"><h3>Kokeile heti</h3>\
         <p>Katso jotain lukua ympärilläsi, vaikkapa kellonaikaa, ja muuta se sanoiksi. Mikä kuva siitä tulee?</p></div>{}",
        character("monster", "right:10mm;bottom:14mm;width:72mm;height:72mm"),
    );
    Page { class: "", body }
}

fn how_to_play(assets: &Assets) -> Page {
    let number_monster = pair("22");
    let word_monster = pair("77");
    let body = format!(
        "<h1>Näin pelaat</h1>\
         <p>Lukuloitsussa olet nuori loitsija. Hirviöt tulevat portaaleista ja kantavat mukanaan lukua tai sanaa. \
         Kirjoita sen vastine ennen kuin hirviö saa sinut kiinni! Liiku nuolinäppäimillä.</p>\
         <div class=\"cols\">\
           <div class=\"box pink example\"><h3>Hirviöllä on luku</h3>{}<p>Hirviö kantaa lukua <b>{}</b>. \
             Kirjoita sen sana: <b>{}</b>.</p></div>\
           <div class=\"box blue example\"><h3>Hirviöllä on sana</h3>{}<p>Hirviö kantaa sanaa <b>{}</b>. \
             Kirjoita sen luku: <b>{}</b>.</p></div>\
         </div>\
         <p>Kun kirjoitat oikein, loitsusi lentää hirviöön ja hirviö räjähtää. Numerot menevät omaan paikkaansa ja kirjaimet \
         omaansa, joten peli tietää, kumpaa vastausta tarkoitat.</p>\
         <h2>Näppäimet</h2>\
         <table class=\"keys\">\
         <tr><td><kbd>←</kbd> <kbd>↑</kbd> <kbd>↓</kbd> <kbd>→</kbd></td><td>Liiku</td></tr>\
         <tr><td><kbd>0</kbd>–<kbd>9</kbd>, kirjaimet</td><td>Kirjoita vastaus</td></tr>\
         <tr><td><kbd>Askelpalautin</kbd></td><td>Tyhjennä kirjoittamasi</td></tr>\
         <tr><td><kbd>Välilyönti</kbd></td><td>Tauko</td></tr>\
         <tr><td><kbd>Tab</kbd></td><td>Äänet päälle tai pois</td></tr>\
         <tr><td><kbd>Esc</kbd></td><td>Takaisin</td></tr>\
         </table>\
         <p>Kosketusnäytöllä peli näyttää oman näppäimistön ja ohjaussauvan. Käännä laite silloin vaakasuuntaan. \
         Ä ja Ö toimivat kaikilla näppäimistöillä.</p>\
         <h2>Vastaan tulee</h2>\
         <div style=\"display:flex;justify-content:space-around;align-items:flex-end;text-align:center\">\
           <div><div class=\"ch monster\" style=\"position:static;width:34mm;height:34mm\"></div><b>Hirviö</b></div>\
           <div><div class=\"ch cyclops\" style=\"position:static;width:34mm;height:34mm\"></div><b>Yksisilmä</b></div>\
           <div><div class=\"ch boss\" style=\"position:static;width:44mm;height:44mm\"></div><b>Pomo</b></div>\
           <div><div class=\"ch portal\" style=\"position:static;width:40mm;height:40mm\"></div><b>Portaali</b></div>\
         </div>",
        picture(assets, number_monster, "pic"),
        number_monster.number,
        number_monster.word,
        picture(assets, word_monster, "pic"),
        word_monster.word,
        word_monster.number,
    );
    Page { class: "", body }
}

fn levels_and_practice() -> Page {
    let body = format!(
        "<h1>Tasot, tähdet ja harjoittelu</h1>\
         <div class=\"cols\">\
           <div><h3>Energia ja tähdet</h3>\
             <p>Oikea vastaus antaa energiaa. Väärä vastaus ja hirviöön törmääminen vievät sitä. \
             Jos energia loppuu, peli päättyy. Mitä enemmän energiaa on jäljellä tason lopussa, sitä \
             enemmän tähtiä saat: yhdestä kolmeen. {}{}{}</p>\
             <h3>Vihjeet</h3>\
             <p>Jos et muista vastausta, odota hetki: vastaus ilmestyy vihjeeksi hirviön alle. \
             Uusissa pareissa vihje tulee nopeammin.</p></div>\
           <div><h3>Tasot</h3>\
             <p>Ensimmäisellä tasolla harjoitellaan numeroita 0–9. Jokainen uusi taso tuo mukaan \
             {} uutta paria, ja kaikki {PAIR_COUNT} paria on tavattu tasolla {}.</p>\
             <h3>Pomot ja pitkät luvut</h3>\
             <p>Jokaisen tason lopussa saapuu pomo, jonka voittamiseen tarvitaan monta osumaa. \
             Tasolta {} alkaen osa osumista on pitkiä lukuja.</p>\
             <h3>Aloita mistä haluat</h3>\
             <p>Uuden pelin voi aloittaa jokaiselta {}. tasolta, jolle olet jo päässyt: 1, {}, {} ja niin edelleen.</p></div>\
         </div>\
         <h2>Harjoittele</h2>\
         <p>Harjoittelutilassa ei ole hirviöitä, vain rauhallisia korttiharjoituksia. Kortissa on luku tai sana, \
         ja sinä kirjoitat sen vastineen. Voit katsoa vastauksen välilyönnillä. Vastauksesi opettavat peliä \
         tuntemaan, mitä osaat.</p>\
         <h2>Edistyminen</h2>\
         <p>Edistymiskartassa on kaikki parit, ja jokaisen väri kertoo, kuinka hyvin osaat sen. \
         Pisteet kertovat saman, jos värit ovat vaikeita erottaa.</p>\
         <div class=\"legend\">\
           <span><i class=\"swatch\" style=\"background:#333342\"></i>Ei vielä nähty</span>\
           <span><i class=\"swatch\" style=\"background:#cc3833\"></i>Harjoittele <i class=\"dots\">●○○</i></span>\
           <span><i class=\"swatch\" style=\"background:#ebbf33\"></i>Melkein <i class=\"dots\">●●○</i></span>\
           <span><i class=\"swatch\" style=\"background:#40b34d\"></i>Osaat <i class=\"dots\">●●●</i></span>\
         </div>\
         <div class=\"box green\"><h3>Peli muistaa puolestasi</h3>\
         <p>Peli seuraa, kuinka nopeasti vastaat. Parit, joita et vielä osaa, tulevat vastaan useammin. \
         Kun osaat parin hyvin, se lepää hetken ja palaa, kun sen kertaaminen on taas ajankohtaista: \
         tunneista päiviin, ja hyvin osatuilla pareilla jopa viikkoihin. Peli laskee myös, kuinka monta päivää \
         putkeen olet pelannut.</p></div>\
         <div class=\"box gold compact\" style=\"margin-right:46mm\"><h3>Kunniamerkit</h3>\
         <p>Pelissä on {} kunniamerkkiä, helpoista aivan vaikeisiin. Ne kertovat hirviöistä, \
         tasoista, opituista pareista ja pelipäivistä. Katso ne aloitusnäytön <kbd>K</kbd>-napista. \
         Ansaittua merkkiä ei oteta pois.</p></div>{}",
        inline_star(),
        inline_star(),
        inline_star(),
        NEW_PAIRS_PER_LEVEL,
        first_long_level() - 1,
        first_long_level(),
        LEVELS_PER_CHECKPOINT,
        1 + LEVELS_PER_CHECKPOINT,
        1 + 2 * LEVELS_PER_CHECKPOINT,
        BADGES.len(),
        character("girl", "right:10mm;bottom:12mm;width:64mm;height:64mm"),
    );
    Page { class: "", body }
}

/// The band above a group of pairs: the tens digit's consonant, or the
/// single digits.
struct Group {
    title: String,
    chip: String,
    hint: String,
    color: String,
    pairs: Vec<Pair>,
}

fn groups() -> Vec<Group> {
    let single: Vec<Pair> = PAIRS
        .iter()
        .filter(|p| p.number.len() == 1)
        .copied()
        .collect();
    let mut groups = vec![Group {
        title: "Yhden numeron parit".to_string(),
        chip: "0–9".to_string(),
        hint: "kaikki numerot yksinään".to_string(),
        color: "dn".to_string(),
        pairs: single,
    }];
    for tens in 0..10usize {
        let pairs: Vec<Pair> = PAIRS
            .iter()
            .filter(|p| p.number.len() == 2 && first_digit(**p) == tens)
            .copied()
            .collect();
        groups.push(Group {
            title: format!("Parit {tens}0–{tens}9"),
            chip: letter(tens),
            hint: format!("ensimmäinen numero {tens} = {}", letter(tens)),
            color: format!("d{tens}"),
            pairs,
        });
    }
    groups
}

fn render_group(assets: &Assets, group: &Group) -> String {
    let cards: String = group
        .pairs
        .iter()
        .map(|p| card(assets, *p, first_digit(*p)))
        .collect();
    format!(
        "<div class=\"group {}\"><div class=\"band\"><span class=\"chip\">{}</span>{}<small>{}</small></div>\
         <div class=\"cards\">{cards}</div></div>",
        group.color, group.chip, group.title, group.hint
    )
}

fn pair_pages(assets: &Assets) -> Vec<Page> {
    let groups = groups();
    let mut pages = Vec::new();
    for (i, two) in groups.chunks(2).enumerate() {
        let title = if i == 0 {
            "Kaikki parit"
        } else {
            "Kaikki parit, jatkuu"
        };
        let mut body = format!("<h1>{title}</h1>");
        for group in two {
            body += &render_group(assets, group);
        }
        if two.len() == 1 {
            body += "<div class=\"box gold\"><h3>Oma tarinani</h3>\
                     <p>Tähän voit piirtää tai kirjoittaa oman kuvasi: mikä pari oli vaikein, ja miten keksit sen muistamaan?</p>\
                     <div class=\"lines\"></div></div>";
        }
        pages.push(Page { class: "", body });
    }
    pages
}

fn exercises() -> Page {
    // A fixed, scattered choice of pairs; 37 and 110 share no factor, so no
    // pair repeats within a list.
    let pick = |offset: usize, count: usize| -> Vec<Pair> {
        (0..count)
            .map(|k| PAIRS[(k * 37 + offset) % PAIR_COUNT])
            .collect()
    };
    let to_words = pick(11, 8);
    let to_numbers = pick(60, 8);
    let longs = ["365", "1917", "12345"];
    let blank = "<span class=\"blank\"></span>";
    let words_list: String = to_words
        .iter()
        .map(|p| format!("<li><b>{}</b> = {blank}</li>", p.number))
        .collect();
    let numbers_list: String = to_numbers
        .iter()
        .map(|p| format!("<li><b>{}</b> = {blank}</li>", escape(p.word)))
        .collect();
    let longs_list: String = longs
        .iter()
        .map(|digits| {
            // One blank for each word the number breaks into.
            let words = Question::for_number(digits).expect("digits").pairs().len();
            format!(
                "<li><b>{digits}</b> = {}</li>",
                vec![blank; words].join(" ")
            )
        })
        .collect();
    let answers = |pairs: &[Pair], number_first: bool| -> String {
        pairs
            .iter()
            .map(|p| {
                if number_first {
                    format!("{} = {}", p.number, p.word)
                } else {
                    format!("{} = {}", p.word, p.number)
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let long_answers: String = longs
        .iter()
        .map(|digits| {
            let q = Question::for_number(digits).expect("digits");
            format!("{digits} = {}", q.words())
        })
        .collect::<Vec<_>>()
        .join(", ");
    let body = format!(
        "<h1>Kokeile itse!</h1>\
         <div class=\"cols\">\
           <div><h2>Mikä sana?</h2><ol class=\"exercise\">{words_list}</ol></div>\
           <div><h2>Mikä luku?</h2><ol class=\"exercise\">{numbers_list}</ol></div>\
         </div>\
         <h2>Pitkät luvut</h2>\
         <p>Pilko luku kahden numeron paloihin ja kirjoita sanat. Vinkki: 365 on päivien määrä vuodessa \
         ja 1917 on Suomen itsenäisyyden vuosi.</p>\
         <ol class=\"exercise\">{longs_list}</ol>\
         <div class=\"box gold\"><h3>Vastaukset</h3>\
         <p class=\"upside\">Mikä sana: {}.<br>Mikä luku: {}.<br>Pitkät luvut: {}.</p>\
         <p style=\"font-size:9pt;margin-top:2mm\">Vastaukset ovat ylösalaisin. Käännä sivu, kun olet valmis!</p></div>\
         <div class=\"box blue\"><h3>Keksi oma!</h3>\
         <p>Valitse luku, esimerkiksi oma puhelinnumerosi, ja muuta se sanoiksi. Kirjoita sanat tähän ja keksi niistä tarina.</p>\
         <div class=\"lines short\"></div></div>{}",
        answers(&to_words, true),
        answers(&to_numbers, false),
        long_answers,
        character("cyclops", "right:12mm;top:9mm;width:26mm;height:26mm"),
    );
    Page { class: "", body }
}

fn back_cover() -> Page {
    let body = format!(
        "{}<h1>Vinkkejä muistamiseen</h1>\
         <ul>\
         <li><b>Kuvittele elävästi.</b> Mieti sanan väri, koko ja ääni. Mitä hullumpi kuva, sen paremmin se jää mieleen.</li>\
         <li><b>Sano ääneen.</b> Sano luku ja sana ääneen yhdessä.</li>\
         <li><b>Harjoittele vähän mutta usein.</b> Kymmenen minuuttia joka päivä on parempi kuin tunti kerran viikossa.</li>\
         <li><b>Virheet kuuluvat asiaan.</b> Peli tuo vaikeat parit takaisin, kunnes ne ovat tuttuja.</li>\
         <li><b>Kokeile oikeassa elämässä.</b> Muuta puhelinnumero tai bussin numero sanoiksi.</li>\
         </ul>\
         <h2>Vanhemmille ja opettajille</h2>\
         <p>Lukuloitsu on ilmainen ja mainokseton, ja sen lähdekoodi on avointa. Peli on tehty lapsille ja nuorille. \
         Peli ei kerää tietoja pelaajista: edistyminen tallentuu vain omalle laitteelle. Pelin verkkosivu laskee vierailut \
         nimettömästi ilman evästeitä, ja sovellukset eivät lähetä mitään mihinkään.</p>\
         <p>Tämä opas on tehty samasta ohjelmasta kuin peli, versiosta {}. Siksi luvut, sanat ja kuvat ovat \
         täsmälleen samat kuin pelissä.</p>\
         <h2>Tekijä ja lisenssit</h2>\
         <div class=\"credits\">\
         <p>Lukuloitsun on tehnyt Ville Peurala. Pelin ja oppaan ohjelmakoodi on lisensoitu ehdoilla MIT tai Apache-2.0. \
         Lukujen ja sanojen lista on lisensoitu CC BY-SA 4.0 -lisenssillä: sitä saa käyttää ja muokata \
         myös kaupallisesti, tekijä mainiten ja samoilla ehdoilla. Kirjasimet Nunito ja Fredoka ovat SIL Open Font License \
         -lisenssin alaisia. Nimi Lukuloitsu ja pelin kuvake eivät kuulu lisensseihin.</p>\
         <p><b>Pelaa: lukuloitsu.fi</b><br>Lähdekoodi: github.com/vpeurala/lukuloitsu.fi</p></div>{}{}{}{}{}",
        sparkles(24, 192.0, 292.0),
        env!("CARGO_PKG_VERSION"),
        character(
            "girl-casting",
            "right:6mm;bottom:8mm;width:110mm;height:110mm"
        ),
        character("monster", "left:10mm;bottom:14mm;width:60mm;height:60mm"),
        character("star", "left:84mm;bottom:36mm;width:16mm;height:16mm"),
        character("star", "left:14mm;bottom:90mm;width:10mm;height:10mm"),
        character("star", "right:22mm;bottom:112mm;width:12mm;height:12mm"),
    );
    Page {
        class: "dark",
        body,
    }
}
