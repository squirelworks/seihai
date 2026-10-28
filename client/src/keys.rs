//! Game keys.
//!
//! The defaults are the ones every readme from 1.33 to 2.05 lists for
//! player 1: arrows or I/J/K/L to walk, A action, S or Space attack, D
//! item, M map (or close a sign), Q weapon list, P pause and Tab for the
//! chat field. From 1.38 the keys are configurable: `keyset_<action>=`
//! lines in the settings file replace a default, up to three keys each,
//! as in graalconfig.txt.

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
            keys: Action::ALL.iter().map(|&a| (a, a.defaults().to_vec())).collect(),
        }
    }
}

impl KeyMap {
    /// The defaults, with `keyset_<action>` overrides applied when the
    /// version has configurable keys.
    pub fn for_version(configurable: bool, keysets: &[(String, String)]) -> Self {
        let mut map = Self::default();
        if !configurable {
            return map;
        }
        for (name, value) in keysets {
            let Some(action) = Action::ALL.iter().copied().find(|a| a.name() == name) else {
                continue;
            };
            let keys: Vec<Key> = value.split(',').filter_map(|k| parse_key(k.trim())).take(3).collect();
            if !keys.is_empty() {
                map.set(action, keys);
            }
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
        self.keys(action).first().map(|k| key_char(*k)).unwrap_or(' ')
    }

    pub fn down(&self, window: &minifb::Window, action: Action) -> bool {
        self.keys(action).iter().any(|k| window.is_key_down(*k))
    }

    pub fn pressed(&self, window: &minifb::Window, action: Action) -> bool {
        self.keys(action)
            .iter()
            .any(|k| window.is_key_pressed(*k, minifb::KeyRepeat::No))
    }
}

/// A key name such as `A`, `space`, `tab`, `up` or `numpad5`.
pub fn parse_key(name: &str) -> Option<Key> {
    let n = name.to_ascii_lowercase();
    let letters = [
        Key::A, Key::B, Key::C, Key::D, Key::E, Key::F, Key::G, Key::H, Key::I, Key::J, Key::K,
        Key::L, Key::M, Key::N, Key::O, Key::P, Key::Q, Key::R, Key::S, Key::T, Key::U, Key::V,
        Key::W, Key::X, Key::Y, Key::Z,
    ];
    let digits = [
        Key::Key0, Key::Key1, Key::Key2, Key::Key3, Key::Key4, Key::Key5, Key::Key6, Key::Key7,
        Key::Key8, Key::Key9,
    ];
    let pad = [
        Key::NumPad0, Key::NumPad1, Key::NumPad2, Key::NumPad3, Key::NumPad4, Key::NumPad5,
        Key::NumPad6, Key::NumPad7, Key::NumPad8, Key::NumPad9,
    ];
    let b = n.as_bytes();
    if b.len() == 1 && b[0].is_ascii_lowercase() {
        return Some(letters[(b[0] - b'a') as usize]);
    }
    if b.len() == 1 && b[0].is_ascii_digit() {
        return Some(digits[(b[0] - b'0') as usize]);
    }
    if let Some(d) = n.strip_prefix("numpad").and_then(|d| d.parse::<usize>().ok()) {
        return pad.get(d).copied();
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
    fn key_names() {
        assert_eq!(parse_key("numpad5"), Some(Key::NumPad5));
        assert_eq!(parse_key("Q"), Some(Key::Q));
        assert_eq!(parse_key("tab"), Some(Key::Tab));
        assert_eq!(parse_key("nonsense"), None);
    }
}
