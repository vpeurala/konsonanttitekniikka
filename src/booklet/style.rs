//! The booklet's stylesheet, for A4 pages. Fonts and pictures are added by
//! `booklet::html`, because they are embedded data.

/// The colours of the digits 0-9, as (background, text). The same colours
/// mark a digit and its consonant everywhere in the booklet, like the
/// rainbow in the game's progress map.
pub const DIGIT_COLORS: [(&str, &str); 10] = [
    ("#e63946", "#ffffff"),
    ("#f4842a", "#ffffff"),
    ("#f7c325", "#3a2a00"),
    ("#7cc242", "#12300a"),
    ("#17b890", "#04302a"),
    ("#33b5e5", "#032b3d"),
    ("#3d5bd9", "#ffffff"),
    ("#7d3cc8", "#ffffff"),
    ("#e83e9b", "#ffffff"),
    ("#8a5a44", "#ffffff"),
];

pub const CSS: &str = r#"
@page { size: A4; margin: 0; }
:root {
  --ink: #2b1d55; --paper: #fffaf0; --violet: #5a2ca0; --pink: #e83e9b;
  --gold: #f7c325; --soft: #ece6f7; --line: #cfc3ea;
  --rainbow: linear-gradient(90deg, #e63946, #f4842a, #f7c325, #7cc242,
    #17b890, #33b5e5, #3d5bd9, #7d3cc8, #e83e9b, #8a5a44);
}
* { box-sizing: border-box; }
html, body { margin: 0; padding: 0; }
body {
  font-family: Nunito, sans-serif; font-size: 11.5pt; line-height: 1.45;
  color: var(--ink); background: #d8d3e6;
  -webkit-print-color-adjust: exact; print-color-adjust: exact;
}
.page {
  position: relative; width: 210mm; height: 296.6mm; margin: 0 auto 8mm;
  padding: 15mm 16mm 18mm; overflow: hidden; background: var(--paper);
  break-after: page; page-break-after: always;
}
@media print {
  body { background: none; }
  .page { margin: 0; }
  .page:last-child { break-after: auto; page-break-after: auto; }
}
.page.dark {
  color: #fff;
  background: radial-gradient(circle at 50% 58%, #8a4de0 0, #4b2a94 32%, #2a1d5c 62%, #1b1240 100%);
}
h1, h2, h3, .fred { font-family: Fredoka, sans-serif; font-weight: 600; }
h1 { font-size: 27pt; line-height: 1.1; margin: 0 0 6mm; color: var(--violet); }
h1::after {
  content: ""; display: block; width: 62mm; height: 2.4mm; margin-top: 2.5mm;
  border-radius: 2mm; background: var(--rainbow);
}
h2 { font-size: 16pt; line-height: 1.15; margin: 6mm 0 2mm; color: var(--pink); }
h3 { font-size: 12.5pt; margin: 0 0 1.5mm; color: var(--violet); }
p { margin: 0 0 3.2mm; }
ul, ol { margin: 0 0 3.2mm; padding-left: 6mm; }
li { margin-bottom: 1.4mm; }
b, strong { font-weight: 700; }
.foot {
  position: absolute; left: 16mm; right: 16mm; bottom: 7mm;
  display: flex; justify-content: space-between; font-size: 8.5pt; color: #8a7cb5;
}
.dark .foot { color: #c9b8f5; }
.ch { position: absolute; background-repeat: no-repeat; background-size: contain; background-position: center; }
.spark { position: absolute; border-radius: 50%; background: #fff; }
.box {
  background: #fff; border: .6mm solid var(--line); border-radius: 5mm;
  padding: 4mm 5mm; margin: 0 0 4mm; box-shadow: 0 1mm 0 var(--line);
}
.box.gold { background: #fff3c4; border-color: #f0cf5a; box-shadow: 0 1mm 0 #f0cf5a; }
.box.pink { background: #ffe4f1; border-color: #f4a6cc; box-shadow: 0 1mm 0 #f4a6cc; }
.box.blue { background: #dff4fd; border-color: #8ed3ee; box-shadow: 0 1mm 0 #8ed3ee; }
.box.green { background: #e6f6d8; border-color: #aedb85; box-shadow: 0 1mm 0 #aedb85; }
.box > :last-child { margin-bottom: 0; }
.cols { display: grid; grid-template-columns: 1fr 1fr; gap: 5mm; }
.steps { display: grid; grid-template-columns: repeat(3, 1fr); gap: 4mm; margin: 4mm 0; }
.step { text-align: center; }
.step .no {
  width: 12mm; height: 12mm; margin: 0 auto 2mm; border-radius: 50%; color: #fff;
  font: 600 17pt/12mm Fredoka, sans-serif; background: var(--pink);
}
.digits { display: grid; grid-template-columns: repeat(10, 1fr); gap: 2mm; margin: 4mm 0 5mm; }
.digit {
  border-radius: 3.5mm; text-align: center; padding: 2.5mm 0 2mm;
  background: var(--c); color: var(--t);
}
.digit .n { display: block; font: 600 19pt/1.1 Fredoka, sans-serif; }
.digit .eq { display: block; font-size: 9pt; opacity: .8; }
.digit .l { display: block; font: 600 22pt/1.1 Fredoka, sans-serif; }
.tiles { display: flex; align-items: flex-start; justify-content: center; margin: 2mm 0; }
.tile {
  width: 10.5mm; margin: 0 .7mm; text-align: center; border-radius: 2.5mm; overflow: hidden;
  background: var(--c); color: var(--t);
}
.tile b { display: block; font: 600 18pt/1.2 Fredoka, sans-serif; padding-top: 1mm; }
.tile i { display: block; font: normal 700 12pt/1.2 Nunito, sans-serif; padding-bottom: 1mm; }
.tile.vowel { background: var(--soft); color: #8b7fb0; }
.result { text-align: center; font: 600 20pt Fredoka, sans-serif; margin: 1mm 0 0; color: var(--violet); }
.example { text-align: center; }
.example .pic { width: 30mm; height: 30mm; border-radius: 4mm; margin: 0 auto 2mm; display: block; }
.group { margin: 0 0 6mm; }
.band {
  display: flex; align-items: center; gap: 3mm; color: var(--t); background: var(--c);
  border-radius: 4mm 4mm 0 0; padding: 1.6mm 4mm; font: 600 14pt Fredoka, sans-serif;
}
.band .chip {
  background: #fff; color: var(--c); width: 8.5mm; height: 8.5mm; border-radius: 50%;
  text-align: center; font: 600 13pt/8.5mm Fredoka, sans-serif;
}
.band small { font: 600 10pt Nunito, sans-serif; margin-left: auto; opacity: .9; }
.cards {
  display: grid; grid-template-columns: repeat(5, 1fr); gap: 2.5mm; padding: 3mm;
  border: .6mm solid var(--c); border-top: none; border-radius: 0 0 4mm 4mm; background: #fff;
}
.card { text-align: center; }
.card img { display: block; width: 100%; aspect-ratio: 1; border-radius: 3mm; }
.card .num {
  display: inline-block; margin-top: 1.2mm; min-width: 9mm; padding: 0 2mm; border-radius: 3mm;
  background: var(--c); color: var(--t); font: 600 13pt/1.35 Fredoka, sans-serif;
}
.card .word { font: 700 12.5pt/1.2 Nunito, sans-serif; margin-top: .6mm; }
.keys { width: 100%; border-collapse: separate; border-spacing: 0 1.6mm; }
.keys td { vertical-align: middle; }
.keys td:first-child { width: 42mm; }
kbd {
  display: inline-block; min-width: 9mm; padding: .3mm 2.2mm; text-align: center;
  background: #fff; border: .5mm solid var(--line); border-bottom-width: 1.1mm;
  border-radius: 2mm; font: 700 10pt Nunito, sans-serif;
}
.dn { --c: #5a2ca0; --t: #ffffff; }
.row { display: flex; align-items: center; gap: 5mm; }
.row .pic { width: 27mm; height: 27mm; border-radius: 4mm; flex: none; }
.row.compact .pic { width: 22mm; height: 22mm; }
.row .tiles { justify-content: flex-start; }
.row .result { text-align: left; }
.card.mini { width: 28mm; flex: none; }
.card.mini .pic { width: 20mm; margin: 0 auto; }
.box.compact { padding: 3mm 5mm; margin-bottom: 3mm; }
.box.compact h3 { margin-bottom: 1mm; }
.plus { font: 600 20pt Fredoka, sans-serif; color: var(--pink); }
.lines {
  height: 88mm; margin-top: 3mm;
  background: repeating-linear-gradient(to bottom, transparent 0, transparent 8.4mm, #e6cf7a 8.4mm, #e6cf7a 9mm);
}
.lines.short { height: 22mm; margin-top: 1mm; }
.legend { display: flex; gap: 4mm; margin: 2mm 0 3mm; }
.legend span { display: flex; align-items: center; gap: 1.5mm; font-size: 10pt; }
.swatch { width: 7mm; height: 7mm; border-radius: 2mm; display: inline-block; }
.dots { letter-spacing: .5mm; font-size: 9pt; }
.exercise { margin: 0 0 3mm; }
.exercise li { margin-bottom: 2.4mm; }
.blank { display: inline-block; min-width: 38mm; border-bottom: .5mm solid var(--ink); }
.upside { transform: rotate(180deg); font-size: 9pt; color: #6b5c9a; }
.upside b { color: var(--violet); }
.credits { font-size: 9.5pt; }
.dark h1, .dark h2, .dark p, .dark ul, .dark .credits { position: relative; z-index: 1; }
.dark h1 { color: #ffd166; }
.dark h2 { color: #ffb3d9; }
.dark p, .dark li { color: #efe8ff; }
.cover-title {
  position: absolute; left: 0; right: 0; top: 22mm; text-align: center;
  font: 600 66pt/1 Fredoka, sans-serif; color: #ffd166;
  text-shadow: 0 1.4mm 0 #b4438e, 0 3mm 6mm rgba(0, 0, 0, .35);
}
.cover-sub {
  position: absolute; left: 0; right: 0; top: 55mm; text-align: center;
  font: 600 22pt Fredoka, sans-serif; color: #fff;
}
.cover-note {
  position: absolute; left: 0; right: 0; bottom: 14mm; text-align: center;
  font-size: 11pt; color: #d9c9ff;
}
"#;
