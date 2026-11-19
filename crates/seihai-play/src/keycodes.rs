//! The key numbers a GS2 script's `onKeyPressed(code, key, scan)` gets.
//!
//! Lab key sweep, 2026-11-19 (Graal_analysis `client-internals/keysweep.md`):
//! the real 4.20r8 (Windows) and 6.03r7 (Linux) clients both pass every key
//! to the scripts, F-keys, Esc and the modifiers alone included, and send
//! no packet for a key with no game action of its own.
//!
//! - `code` is the Windows virtual-key code, on Linux too: F1 to F12 are
//!   112 to 123, letters 65 to 90, digits 48 to 57, the arrows 37 to 40.
//!   Shift adds 256, Ctrl 512 and Alt 1024 (Shift+Left is 293, Alt+Return
//!   1037). Shift, Ctrl and Alt alone are 160, 162 and 164 (the left keys).
//! - `key` is the character: a lower-case letter for a letter key, the
//!   digit or sign for the others, a space for Space, empty for a key with
//!   no character. With Shift held a letter is upper case (inferred: the
//!   sweep only pressed Shift with the arrows).
//! - `scan` is the keyboard scan code: set 1 on Windows (A 30, F1 59);
//!   the Linux 6.03r7 passes the X key code (set 1 plus 8). Seihai always
//!   gives set 1, as the Windows builds do.

/// Shift, Ctrl and Alt add these to `code`.
pub const SHIFT: u32 = 256;
pub const CTRL: u32 = 512;
pub const ALT: u32 = 1024;

/// The left Shift, Ctrl and Alt alone: (code, scan).
pub const SHIFT_ALONE: (u32, u32) = (160, 42);
pub const CTRL_ALONE: (u32, u32) = (162, 29);
pub const ALT_ALONE: (u32, u32) = (164, 56);

/// One key press as a script sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Press {
    pub code: u32,
    pub key: String,
    pub scan: u32,
}

/// The press for `key` with these modifiers, or `None` for a key the
/// real clients have no code for (the copy, cut and paste events).
pub fn press(key: egui::Key, mods: egui::Modifiers) -> Option<Press> {
    let (vk, scan, ch) = base(key)?;
    let mut code = vk;
    if mods.shift {
        code += SHIFT;
    }
    if mods.ctrl {
        code += CTRL;
    }
    if mods.alt {
        code += ALT;
    }
    let key = match ch {
        Some(c) if mods.shift && c.is_ascii_alphabetic() => c.to_ascii_uppercase().to_string(),
        Some(c) => c.to_string(),
        None => String::new(),
    };
    Some(Press { code, key, scan })
}

/// The press for a modifier pressed alone.
pub fn modifier(alone: (u32, u32)) -> Press {
    Press {
        code: alone.0,
        key: String::new(),
        scan: alone.1,
    }
}

/// (virtual key, set-1 scan code, character) of a key.
fn base(key: egui::Key) -> Option<(u32, u32, Option<char>)> {
    use egui::Key as K;
    const LETTER_SCANS: [u32; 26] = [
        30, 48, 46, 32, 18, 33, 34, 35, 23, 36, 37, 38, 50, 49, 24, 25, 16, 19, 31, 20, 22, 47, 17,
        45, 21, 44,
    ];
    const LETTERS: [K; 26] = [
        K::A,
        K::B,
        K::C,
        K::D,
        K::E,
        K::F,
        K::G,
        K::H,
        K::I,
        K::J,
        K::K,
        K::L,
        K::M,
        K::N,
        K::O,
        K::P,
        K::Q,
        K::R,
        K::S,
        K::T,
        K::U,
        K::V,
        K::W,
        K::X,
        K::Y,
        K::Z,
    ];
    if let Some(i) = LETTERS.iter().position(|&k| k == key) {
        let c = (b'a' + i as u8) as char;
        return Some((65 + i as u32, LETTER_SCANS[i], Some(c)));
    }
    const DIGITS: [K; 10] = [
        K::Num0,
        K::Num1,
        K::Num2,
        K::Num3,
        K::Num4,
        K::Num5,
        K::Num6,
        K::Num7,
        K::Num8,
        K::Num9,
    ];
    if let Some(i) = DIGITS.iter().position(|&k| k == key) {
        // Scan codes: 1 is 2 ... 9 is 10, 0 is 11.
        let scan = if i == 0 { 11 } else { i as u32 + 1 };
        return Some((48 + i as u32, scan, Some((b'0' + i as u8) as char)));
    }
    const FKEYS: [K; 12] = [
        K::F1,
        K::F2,
        K::F3,
        K::F4,
        K::F5,
        K::F6,
        K::F7,
        K::F8,
        K::F9,
        K::F10,
        K::F11,
        K::F12,
    ];
    if let Some(i) = FKEYS.iter().position(|&k| k == key) {
        let scan = match i {
            10 => 87,
            11 => 88,
            _ => 59 + i as u32,
        };
        return Some((112 + i as u32, scan, None));
    }
    Some(match key {
        K::Escape => (27, 1, None),
        K::Tab => (9, 15, None),
        K::Backspace => (8, 14, None),
        K::Enter => (13, 28, None),
        K::Space => (32, 57, Some(' ')),
        K::Insert => (45, 82, None),
        K::Delete => (46, 83, None),
        K::Home => (36, 71, None),
        K::End => (35, 79, None),
        K::PageUp => (33, 73, None),
        K::PageDown => (34, 81, None),
        K::ArrowLeft => (37, 75, None),
        K::ArrowUp => (38, 72, None),
        K::ArrowRight => (39, 77, None),
        K::ArrowDown => (40, 80, None),
        K::Backtick => (192, 41, Some('`')),
        K::Minus => (189, 12, Some('-')),
        K::Equals | K::Plus => (187, 13, Some('=')),
        K::OpenBracket => (219, 26, Some('[')),
        K::CloseBracket => (221, 27, Some(']')),
        K::Backslash | K::Pipe => (220, 43, Some('\\')),
        K::Semicolon | K::Colon => (186, 39, Some(';')),
        K::Quote => (222, 40, Some('\'')),
        K::Comma => (188, 51, Some(',')),
        K::Period => (190, 52, Some('.')),
        K::Slash | K::Questionmark => (191, 53, Some('/')),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(key: egui::Key, mods: egui::Modifiers) -> Press {
        press(key, mods).unwrap()
    }

    /// The codes the lab sweep read from the real clients.
    #[test]
    fn codes_match_the_real_clients() {
        let none = egui::Modifiers::NONE;
        assert_eq!(p(egui::Key::F1, none).code, 112);
        assert_eq!(p(egui::Key::F12, none).code, 123);
        assert_eq!(p(egui::Key::Escape, none).code, 27);
        assert_eq!(
            p(egui::Key::A, none),
            Press {
                code: 65,
                key: "a".into(),
                scan: 30
            }
        );
        assert_eq!(p(egui::Key::F1, none).scan, 59);
        assert_eq!(p(egui::Key::Backtick, none).code, 192);
        assert_eq!(p(egui::Key::Quote, none).code, 222);
        assert_eq!(p(egui::Key::Num0, none).code, 48);
        assert_eq!(p(egui::Key::PageUp, none).code, 33);
        assert_eq!(p(egui::Key::ArrowLeft, egui::Modifiers::SHIFT).code, 293);
        assert_eq!(p(egui::Key::ArrowLeft, egui::Modifiers::CTRL).code, 549);
        assert_eq!(p(egui::Key::Num7, egui::Modifiers::ALT).code, 1079);
        assert_eq!(p(egui::Key::Enter, egui::Modifiers::ALT).code, 1037);
        assert_eq!(modifier(SHIFT_ALONE).code, 160);
        assert!(press(egui::Key::Copy, none).is_none());
    }
}
