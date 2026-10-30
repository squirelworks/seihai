//! The key list shown by F1, and the on-screen hints.
//!
//! [`BINDINGS`] lists every key the game loop in `bin/main.rs` reads:
//! its `minifb::Key::X` names, and the default keys of each
//! [`crate::keys::Action`] it uses. A test reads that file and checks the
//! two agree in both directions: a key in the table that main.rs never
//! reads, or one it reads that the table does not list, fails the build.
//! So the help cannot drift from the real bindings. The help shows the
//! defaults; `keyset_` lines in the settings file can change them.

use crate::dev::DevMode;
use crate::render::font::{text_width, CELL_H};
use crate::render::Framebuffer;

/// One line of the help.
pub struct KeyHelp {
    /// What is shown for the key, e.g. `"A / Space"`.
    pub label: &'static str,
    /// The `minifb::Key` variants this line stands for.
    pub keys: &'static [&'static str],
    /// What it does.
    pub does: &'static str,
    /// Only listed (and only works) in a build with developer mode.
    pub dev_only: bool,
}

const fn k(label: &'static str, keys: &'static [&'static str], does: &'static str) -> KeyHelp {
    KeyHelp { label, keys, does, dev_only: false }
}

/// Every bound key, in the order they are shown.
pub const BINDINGS: &[KeyHelp] = &[
    k("F1 / ?", &["F1"], "show or hide this help"),
    k("Arrows / IJKL", &["Left", "Right", "Up", "Down", "I", "J", "K", "L"], "walk; two at once for a diagonal"),
    k("A", &["A"], "read a sign, open a chest, close a sign, lift a bush or rock, throw what you carry"),
    k("S / Space", &["S", "Space"], "swing the sword (others judge the hit); while carrying, throw instead"),
    k("M", &["M"], "close a sign (the map is not done yet)"),
    k("Enter / T / Tab", &["Enter", "NumPadEnter", "T", "Tab"], "open the chat bar; Enter sends what you typed"),
    k("Backspace", &["Backspace"], "delete a typed letter"),
    k("Up / Down", &[], "in the chat bar: earlier lines (1.39 and later)"),
    k("Esc", &["Escape"], "close the chat bar or a sign; else ask to quit"),
    k("Y / N", &["Y", "N"], "answer the quit question (Enter = yes, Esc = no)"),
    k("F2", &["F2"], "private message to the nearest player"),
    k("Q", &["Q"], "open or close the item list; arrows pick a weapon, Esc/D/S/A/Enter close it"),
    k("D", &["D"], "weapon key: closes the item list (firing weapons is not done yet)"),
    k("P", &["P"], "pause or go on; others see \"(paused)\" and cannot hurt you (also while the window has no focus)"),
    k("keyset_...", &[], "settings file: rebind the game keys above (1.38 and later)"),
    k("F7", &["F7"], "show or hide the player list"),
    k("/ignore /buddy /mute", &[], "chat bar: add a name (/unignore ... removes); /lists shows"),
    k("PgUp / PgDn", &["PageUp", "PageDown"], "scroll the chat"),
    k("F3", &["F3"], "debug information"),
    k("F4", &["F4"], "level editor (not built yet)"),
    k("F5", &["F5"], "open a level file (clients with an editor)"),
    k("F12", &["F12"], "save a screenshot to screenshots/"),
    KeyHelp {
        label: "F9",
        keys: &["F9"],
        does: "developer mode: walk through walls (off at start)",
        dev_only: true,
    },
];

/// Lines for the overlay: (key, what it does).
pub fn help_lines() -> Vec<(&'static str, &'static str)> {
    BINDINGS
        .iter()
        .filter(|b| !b.dev_only || DevMode::AVAILABLE)
        .map(|b| (b.label, b.does))
        .collect()
}

/// The hint shown at the start.
pub const HINT: &str = "F1 help";

/// Draw the help overlay, centred.
pub fn draw_help(fb: &mut Framebuffer) {
    let lines = help_lines();
    let key_w = lines.iter().map(|(l, _)| text_width(l)).max().unwrap_or(0) + 12;
    let desc_w = lines.iter().map(|(_, d)| text_width(d)).max().unwrap_or(0);
    let w = (key_w + desc_w + 24).min(fb.width as i32 - 8);
    let line_h = CELL_H + 3;
    let h = (lines.len() as i32 + 3) * line_h + 8;
    let x = (fb.width as i32 - w) / 2;
    let y = ((fb.height as i32 - h) / 2).max(2);
    fb.shade_rect(x, y, w, h, 40);
    fb.fill_rect(x, y, w, 1, [0xC0, 0xC0, 0xC0]);
    fb.fill_rect(x, y + h - 1, w, 1, [0xC0, 0xC0, 0xC0]);
    fb.draw_text(x + 8, y + 6, "Seihai keys", [0xFF, 0xE0, 0x70]);
    for (i, (key, does)) in lines.iter().enumerate() {
        let ly = y + 6 + (i as i32 + 2) * line_h;
        let dev = key.starts_with("F9");
        fb.draw_text(x + 8, ly, key, if dev { [0xFF, 0x90, 0x90] } else { [0x90, 0xE0, 0xFF] });
        fb.draw_text(x + 8 + key_w, ly, does, if dev { [0xFF, 0xB0, 0xB0] } else { [0xFF, 0xFF, 0xFF] });
    }
    let hint = "F1 or ? to close";
    fb.draw_text(x + w - 8 - text_width(hint), y + h - 4 - CELL_H, hint, [0xA0, 0xA0, 0xA0]);
}

/// Draw the one-line hint at the top of the screen.
pub fn draw_hint(fb: &mut Framebuffer) {
    let x = (fb.width as i32 - text_width(HINT)) / 2;
    fb.draw_text_outlined(x, 3, HINT, [0xFF, 0xFF, 0xFF]);
}

/// Draw the developer-mode badge when it is on.
pub fn draw_dev_badge(fb: &mut Framebuffer, dev: &DevMode) {
    if let Some(text) = crate::dev::badge(dev) {
        let x = (fb.width as i32 - text_width(text)) / 2;
        fb.draw_text_outlined(x, 14, text, [0xFF, 0x50, 0x50]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Every `minifb::Key::Name` read by the game loop, and the default
    /// keys of every `Action::Name` it uses (the sign keys are decided in
    /// `render/signs.rs` for it).
    fn keys_in_main() -> BTreeSet<String> {
        let signs = include_str!("render/signs.rs");
        let signs = signs.split("#[cfg(test)]").next().unwrap_or(signs);
        let src = [include_str!("bin/main.rs"), signs].concat();
        let src = src.as_str();
        let mut out = BTreeSet::new();
        let mut rest = src;
        while let Some(i) = rest.find("minifb::Key::") {
            rest = &rest[i + "minifb::Key::".len()..];
            let name: String = rest.chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
            out.insert(name);
        }
        let map = crate::keys::KeyMap::default();
        let uses_direction = src.contains("any_direction_down");
        for a in crate::keys::Action::ALL {
            let direction = matches!(a, crate::keys::Action::Up | crate::keys::Action::Left | crate::keys::Action::Down | crate::keys::Action::Right);
            if src.contains(&format!("Action::{a:?}")) || (direction && uses_direction) {
                out.extend(map.keys(a).iter().map(|k| format!("{k:?}")));
            }
        }
        out
    }

    #[test]
    fn help_lists_exactly_the_keys_main_reads() {
        let in_main = keys_in_main();
        let in_table: BTreeSet<String> =
            BINDINGS.iter().flat_map(|b| b.keys.iter().map(|s| s.to_string())).collect();
        let unlisted: Vec<_> = in_main.difference(&in_table).collect();
        let unhandled: Vec<_> = in_table.difference(&in_main).collect();
        assert!(unlisted.is_empty(), "keys main.rs reads but the help omits: {unlisted:?}");
        assert!(unhandled.is_empty(), "keys the help lists but main.rs never reads: {unhandled:?}");
    }

    #[test]
    fn question_mark_opens_help_too() {
        assert!(include_str!("bin/main.rs").contains("'?'"));
        assert!(BINDINGS[0].label.contains('?'));
    }

    #[test]
    fn dev_key_is_marked_developer_only() {
        let f9 = BINDINGS.iter().find(|b| b.keys.contains(&"F9")).unwrap();
        assert!(f9.dev_only);
        assert_eq!(help_lines().iter().any(|(l, _)| *l == "F9"), DevMode::AVAILABLE);
    }

    #[test]
    fn overlay_fits_the_screen() {
        let mut fb = Framebuffer::new(640, 480);
        draw_help(&mut fb);
        draw_hint(&mut fb);
        let longest = help_lines().iter().map(|(k, d)| text_width(k) + 12 + text_width(d)).max().unwrap();
        assert!(longest + 24 <= 640, "help line too wide: {longest}");
        assert!(help_lines().len() as i32 * (CELL_H + 3) < 440);
    }
}
