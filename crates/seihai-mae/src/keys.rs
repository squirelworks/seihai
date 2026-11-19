//! Game keys.
//!
//! The defaults are the ones every readme from 1.33 to 2.05 lists for
//! player 1: arrows or I/J/K/L to walk, A action, S or Space attack, D
//! item, M map (or close a sign), Q weapon list, P pause and Tab for the
//! chat field. From 1.38 the keys are configurable, up to three keys an
//! action. The settings file takes either the `graalconfig.txt` form, one
//! slot a line (`keyset_attack_key0=S`, `_key1`, `_key2`; an empty value
//! leaves that slot unbound), or a short form with the keys in one line
//! (`keyset_attack=S,Space`). Joystick lines (`_joystickbutton`) are
//! ignored.

use minifb::Key;

/// The configurable actions, named as in graalconfig.txt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Up,
    Left,
    Down,
    Right,
    Item,
    Attack,
    Act,
    Map,
    SwitchToChat,
    ItemList,
    Pause,
}

/// The actions in script `keydown` order (see [`KeyMap::keydown_bits`]):
/// A is the action key, S the attack, D the item.
pub const KEYDOWN_ORDER: [Action; 11] = [
    Action::Up,
    Action::Left,
    Action::Down,
    Action::Right,
    Action::Act,
    Action::Attack,
    Action::Item,
    Action::Map,
    Action::SwitchToChat,
    Action::ItemList,
    Action::Pause,
];

impl Action {
    pub const ALL: [Action; 11] = [
        Action::Up,
        Action::Left,
        Action::Down,
        Action::Right,
        Action::Item,
        Action::Attack,
        Action::Act,
        Action::Map,
        Action::SwitchToChat,
        Action::ItemList,
        Action::Pause,
    ];

    /// The graalconfig name, as in `keyset_<name>`.
    pub fn name(self) -> &'static str {
        match self {
            Action::Up => "up",
            Action::Left => "left",
            Action::Down => "down",
            Action::Right => "right",
            Action::Item => "item",
            Action::Attack => "attack",
            Action::Act => "action",
            Action::Map => "map",
            Action::SwitchToChat => "switchtochat",
            Action::ItemList => "itemlist",
            Action::Pause => "pause",
        }
    }

    fn defaults(self) -> &'static [Key] {
        match self {
            Action::Up => &[Key::Up, Key::I],
            Action::Left => &[Key::Left, Key::J],
            Action::Down => &[Key::Down, Key::K],
            Action::Right => &[Key::Right, Key::L],
            Action::Item => &[Key::D],
            Action::Attack => &[Key::S, Key::Space],
            Action::Act => &[Key::A],
            Action::Map => &[Key::M],
            Action::SwitchToChat => &[Key::Tab],
            Action::ItemList => &[Key::Q],
            Action::Pause => &[Key::P],
        }
    }
}

/// Keys bound to each action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyMap {
    keys: Vec<(Action, Vec<Key>)>,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            keys: Action::ALL
                .iter()
                .map(|&a| (a, a.defaults().to_vec()))
                .collect(),
        }
    }
}

impl KeyMap {
    /// The defaults, with `keyset_<action>` overrides applied when the
    /// version has configurable keys. `keysets` holds the text after
    /// `keyset_` and the value, e.g. `("attack_key1", "Space")`.
    pub fn for_version(configurable: bool, keysets: &[(String, String)]) -> Self {
        let mut map = Self::default();
        if !configurable {
            return map;
        }
        let action_named = |n: &str| Action::ALL.iter().copied().find(|a| a.name() == n);
        // The slot form: three slots an action, filled from the lines given.
        let mut slots: Vec<(Action, [Option<Key>; 3])> = Vec::new();
        for (name, value) in keysets {
            if let Some((action, slot)) = name.rsplit_once("_key") {
                let (Some(action), Ok(slot @ 0..=2)) =
                    (action_named(action), slot.parse::<usize>())
                else {
                    continue;
                };
                let i = slots
                    .iter()
                    .position(|(a, _)| *a == action)
                    .unwrap_or_else(|| {
                        slots.push((action, [None; 3]));
                        slots.len() - 1
                    });
                slots[i].1[slot] = parse_key(value.trim());
            } else if let Some(action) = action_named(name) {
                let keys: Vec<Key> = value
                    .split(',')
                    .filter_map(|k| parse_key(k.trim()))
                    .take(3)
                    .collect();
                if !keys.is_empty() {
                    map.set(action, keys);
                }
            }
        }
        for (action, keys) in slots {
            map.set(action, keys.into_iter().flatten().collect());
        }
        map
    }

    pub fn set(&mut self, action: Action, keys: Vec<Key>) {
        if let Some(entry) = self.keys.iter_mut().find(|(a, _)| *a == action) {
            entry.1 = keys;
        }
    }

    pub fn keys(&self, action: Action) -> &[Key] {
        self.keys
            .iter()
            .find(|(a, _)| *a == action)
            .map(|(_, k)| k.as_slice())
            .unwrap_or(&[])
    }

    /// The letter shown on an item button for this action.
    pub fn label(&self, action: Action) -> char {
        self.keys(action)
            .first()
            .map(|k| key_char(*k))
            .unwrap_or(' ')
    }

    pub fn down(&self, window: &minifb::Window, action: Action) -> bool {
        self.keys(action).iter().any(|k| window.is_key_down(*k))
    }

    /// The held keys as script `keydown(i)` numbers them: bit i for index
    /// i, in the order 2.05's `newfeatures2001.txt` gives ("0..3:
    /// movement, 4..6 A/S/D, 7 map, 8 tab, 9 Q, 10 P").
    pub fn keydown_bits(&self, window: &minifb::Window) -> u16 {
        KEYDOWN_ORDER
            .iter()
            .enumerate()
            .filter(|(_, a)| self.down(window, **a))
            .fold(0, |b, (i, _)| b | 1 << i)
    }

    /// Any of the four direction actions is held.
    pub fn any_direction_down(&self, window: &minifb::Window) -> bool {
        [Action::Up, Action::Left, Action::Down, Action::Right]
            .into_iter()
            .any(|a| self.down(window, a))
    }

    pub fn pressed(&self, window: &minifb::Window, action: Action) -> bool {
        self.keys(action)
            .iter()
            .any(|k| window.is_key_pressed(*k, minifb::KeyRepeat::No))
    }
}

/// A key name such as `A`, `space`, `tab`, `up` or `numpad5`, or one of
/// the `graalconfig.txt` names: `Num 5` or `N5`, `P_Up`, `P_Down`, `Add`,
/// `Sub`, `Mul`, `Div`, `Dec` and `F9` to `F15`.
pub fn parse_key(name: &str) -> Option<Key> {
    let n = name.to_ascii_lowercase().replace(' ', "");
    let letters = [
        Key::A,
        Key::B,
        Key::C,
        Key::D,
        Key::E,
        Key::F,
        Key::G,
        Key::H,
        Key::I,
        Key::J,
        Key::K,
        Key::L,
        Key::M,
        Key::N,
        Key::O,
        Key::P,
        Key::Q,
        Key::R,
        Key::S,
        Key::T,
        Key::U,
        Key::V,
        Key::W,
        Key::X,
        Key::Y,
        Key::Z,
    ];
    let digits = [
        Key::Key0,
        Key::Key1,
        Key::Key2,
        Key::Key3,
        Key::Key4,
        Key::Key5,
        Key::Key6,
        Key::Key7,
        Key::Key8,
        Key::Key9,
    ];
    let pad = [
        Key::NumPad0,
        Key::NumPad1,
        Key::NumPad2,
        Key::NumPad3,
        Key::NumPad4,
        Key::NumPad5,
        Key::NumPad6,
        Key::NumPad7,
        Key::NumPad8,
        Key::NumPad9,
    ];
    let b = n.as_bytes();
    if b.len() == 1 && b[0].is_ascii_lowercase() {
        return Some(letters[(b[0] - b'a') as usize]);
    }
    if b.len() == 1 && b[0].is_ascii_digit() {
        return Some(digits[(b[0] - b'0') as usize]);
    }
    let pad_digit = ["numpad", "num", "n"]
        .iter()
        .find_map(|p| n.strip_prefix(p).and_then(|d| d.parse::<usize>().ok()));
    if let Some(d) = pad_digit {
        return pad.get(d).copied();
    }
    let f_keys = [
        Key::F9,
        Key::F10,
        Key::F11,
        Key::F12,
        Key::F13,
        Key::F14,
        Key::F15,
    ];
    if let Some(f) = n.strip_prefix('f').and_then(|d| d.parse::<usize>().ok()) {
        return f.checked_sub(9).and_then(|i| f_keys.get(i)).copied();
    }
    Some(match n.as_str() {
        "up" => Key::Up,
        "down" => Key::Down,
        "left" => Key::Left,
        "right" => Key::Right,
        "space" => Key::Space,
        "tab" => Key::Tab,
        "enter" | "return" => Key::Enter,
        "shift" => Key::LeftShift,
        "ctrl" | "control" => Key::LeftCtrl,
        "p_up" | "pageup" => Key::PageUp,
        "p_down" | "pagedown" => Key::PageDown,
        "add" => Key::NumPadPlus,
        "sub" => Key::NumPadMinus,
        "mul" => Key::NumPadAsterisk,
        "div" => Key::NumPadSlash,
        "dec" => Key::NumPadDot,
        _ => return None,
    })
}

/// The keys pressed this frame as the GS1 `keypressed` event numbers
/// them: `#p(0)` the Windows virtual-key code and `#p(1)` the lower-case
/// character (empty for a key with none). The real 2.22 and 2.31 gave A as
/// 65 and `a`, S as 83 and `s` (lab key sweep, 2026-11-19, Graal_analysis
/// `protocol/keysweep/`). The other codes follow the same virtual-key table
/// the 4.20r8 and 6.03r7 scripts get; whether 2.x adds the modifier
/// offsets those builds add is not checked, so none are added here.
pub fn script_presses(window: &minifb::Window) -> Vec<(u32, String)> {
    window
        .get_keys_pressed(minifb::KeyRepeat::No)
        .into_iter()
        .filter_map(virtual_key)
        .collect()
}

/// (virtual-key code, character) of a key.
pub fn virtual_key(k: Key) -> Option<(u32, String)> {
    use Key as K;
    let c = |s: &str| s.to_string();
    let name = format!("{k:?}");
    if name.len() == 1 {
        let ch = name.chars().next()?;
        if ch.is_ascii_uppercase() {
            return Some((ch as u32, ch.to_ascii_lowercase().to_string()));
        }
    }
    if let Some(d) = name.strip_prefix("Key").and_then(|d| d.parse::<u32>().ok()) {
        return Some((48 + d, d.to_string()));
    }
    // The real 2.31 gave the keypad 0 as 48 and "0", the digit's code,
    // not the keypad code 96 the 4.x and later builds give.
    if let Some(d) = name
        .strip_prefix("NumPad")
        .and_then(|d| d.parse::<u32>().ok())
    {
        return Some((48 + d, d.to_string()));
    }
    if let Some(n) = name.strip_prefix('F').and_then(|d| d.parse::<u32>().ok()) {
        return (1..=12).contains(&n).then(|| (111 + n, String::new()));
    }
    Some(match k {
        K::Escape => (27, c("")),
        K::Tab => (9, c("")),
        K::Backspace => (8, c("")),
        K::Enter | K::NumPadEnter => (13, c("")),
        K::Space => (32, c(" ")),
        K::Insert => (45, c("")),
        K::Delete => (46, c("")),
        K::Home => (36, c("")),
        K::End => (35, c("")),
        K::PageUp => (33, c("")),
        K::PageDown => (34, c("")),
        K::Left => (37, c("")),
        K::Up => (38, c("")),
        K::Right => (39, c("")),
        K::Down => (40, c("")),
        K::LeftShift | K::RightShift => (160, c("")),
        K::LeftCtrl | K::RightCtrl => (162, c("")),
        K::LeftAlt | K::RightAlt => (164, c("")),
        K::Backquote => (192, c("`")),
        K::Minus => (189, c("-")),
        K::Equal => (187, c("=")),
        K::LeftBracket => (219, c("[")),
        K::RightBracket => (221, c("]")),
        K::Backslash => (220, c("\\")),
        K::Semicolon => (186, c(";")),
        K::Apostrophe => (222, c("'")),
        K::Comma => (188, c(",")),
        K::Period => (190, c(".")),
        K::Slash => (191, c("/")),
        K::NumPadPlus => (107, c("+")),
        K::NumPadMinus => (109, c("-")),
        K::NumPadAsterisk => (106, c("*")),
        K::NumPadSlash => (111, c("/")),
        K::NumPadDot => (110, c(".")),
        _ => return None,
    })
}

fn key_char(k: Key) -> char {
    let name = format!("{k:?}");
    match name.as_str() {
        n if n.len() == 1 => n.chars().next().unwrap_or(' '),
        "Space" => ' ',
        _ => name.chars().next().unwrap_or(' '),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The codes the real 2.22 gave its `keypressed` scripts.
    #[test]
    fn virtual_keys_match_the_real_2_22() {
        assert_eq!(virtual_key(Key::A), Some((65, "a".into())));
        assert_eq!(virtual_key(Key::S), Some((83, "s".into())));
        assert_eq!(virtual_key(Key::Key0), Some((48, "0".into())));
        assert_eq!(virtual_key(Key::F1), Some((112, String::new())));
        assert_eq!(virtual_key(Key::F12), Some((123, String::new())));
        assert_eq!(virtual_key(Key::Left), Some((37, String::new())));
        assert_eq!(virtual_key(Key::NumPad0), Some((48, "0".into())));
        assert_eq!(virtual_key(Key::F13), None);
    }

    #[test]
    fn readme_defaults() {
        let m = KeyMap::default();
        assert_eq!(m.keys(Action::Attack), &[Key::S, Key::Space]);
        assert_eq!(m.keys(Action::Up), &[Key::Up, Key::I]);
        assert_eq!(m.label(Action::Act), 'A');
        assert_eq!(m.label(Action::Item), 'D');
    }

    #[test]
    fn keysets_only_when_configurable() {
        let sets = vec![("action".to_string(), "e, space".to_string())];
        assert_eq!(KeyMap::for_version(false, &sets).label(Action::Act), 'A');
        let m = KeyMap::for_version(true, &sets);
        assert_eq!(m.keys(Action::Act), &[Key::E, Key::Space]);
        assert_eq!(m.label(Action::Act), 'E');
    }

    #[test]
    fn graalconfig_slot_lines() {
        let sets = vec![
            ("attack_key0".to_string(), "W".to_string()),
            ("attack_key2".to_string(), "Num 8".to_string()),
            ("attack_key1".to_string(), "".to_string()),
            ("attack_joystickbutton".to_string(), "Button1".to_string()),
            ("map_key0".to_string(), "".to_string()),
        ];
        let m = KeyMap::for_version(true, &sets);
        assert_eq!(m.keys(Action::Attack), &[Key::W, Key::NumPad8]);
        assert_eq!(
            m.keys(Action::Map),
            &[] as &[Key],
            "an empty slot line unbinds it"
        );
        assert_eq!(
            m.keys(Action::Act),
            &[Key::A],
            "untouched actions keep the readme key"
        );
    }

    #[test]
    fn key_names() {
        assert_eq!(parse_key("numpad5"), Some(Key::NumPad5));
        assert_eq!(parse_key("N3"), Some(Key::NumPad3));
        assert_eq!(parse_key("P_Up"), Some(Key::PageUp));
        assert_eq!(parse_key("F15"), Some(Key::F15));
        assert_eq!(parse_key("F2"), None);
        assert_eq!(parse_key("Add"), Some(Key::NumPadPlus));
        assert_eq!(parse_key("Q"), Some(Key::Q));
        assert_eq!(parse_key("tab"), Some(Key::Tab));
        assert_eq!(parse_key("nonsense"), None);
    }
}
