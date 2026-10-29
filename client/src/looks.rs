//! The player's looks: head and body pictures and the five body colours
//! (skin, coat, sleeves, shoes, belt), as the GS1 commands `sethead`,
//! `setbody` and `setskincolor` … `setbeltcolor` and prop 13 (COLORS) set
//! them. See `documentation/appearance.md`.
//!
//! From the 2.05 client: a body picture is a palette image, and before it is
//! drawn six palette entries are replaced by colours of the 20-colour list.
//! Colour slot 0 (skin) goes to entry 5, coat to 6, sleeves to 4, shoes to 3,
//! belt to 1, and entry 0 always gets colour 18 (black, the outline).

use crate::scripting::engine::vocab::COLORS;

/// The 20 colours of the GS1 colour list, in list order (2.05 table).
pub const COLOR_RGB: [[u8; 3]; 20] = [
    [255, 255, 255],
    [255, 255, 0],
    [255, 173, 107],
    [255, 132, 132],
    [255, 0, 0],
    [206, 24, 41],
    [132, 255, 132],
    [0, 255, 0],
    [0, 198, 0],
    [132, 132, 255],
    [0, 0, 255],
    [0, 0, 198],
    [132, 0, 0],
    [0, 255, 255],
    [255, 0, 255],
    [132, 0, 132],
    [206, 206, 206],
    [132, 132, 132],
    [0, 0, 8],
    [0, 0, 0],
];

/// The body palette entry each colour slot replaces (skin, coat, sleeves,
/// shoes, belt).
pub const SLOT_PALETTE_INDEX: [usize; 5] = [5, 6, 4, 3, 1];

/// The colour the 2.05 client puts in body palette entry 0.
pub const OUTLINE_COLOR: u8 = 18;

/// The colours a 2.05 player starts with: orange skin, dark red coat, black
/// sleeves, dark red shoes, black belt.
pub const DEFAULT_COLORS: [u8; 5] = [2, 5, 18, 5, 18];

/// The default body and head pictures.
pub const DEFAULT_BODY: &str = "body.png";
pub const DEFAULT_HEAD: &str = "head0.png";

/// One player's looks. `None` is the default picture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Looks {
    pub head: Option<String>,
    pub body: Option<String>,
    pub colors: [u8; 5],
}

impl Default for Looks {
    fn default() -> Self {
        Looks { head: None, body: None, colors: DEFAULT_COLORS }
    }
}

impl Looks {
    pub fn head_name(&self) -> &str {
        self.head.as_deref().filter(|h| !h.is_empty()).unwrap_or(DEFAULT_HEAD)
    }

    pub fn body_name(&self) -> &str {
        self.body.as_deref().filter(|b| !b.is_empty()).unwrap_or(DEFAULT_BODY)
    }
}

/// The palette changes that draw a body in `colors`. A colour number past
/// the list is left as the picture has it.
pub fn body_recolor(colors: &[u8; 5]) -> Vec<(usize, [u8; 3])> {
    let mut out: Vec<(usize, [u8; 3])> = SLOT_PALETTE_INDEX
        .iter()
        .zip(colors)
        .filter_map(|(&idx, &c)| COLOR_RGB.get(usize::from(c)).map(|&rgb| (idx, rgb)))
        .collect();
    out.push((0, COLOR_RGB[usize::from(OUTLINE_COLOR)]));
    out
}

/// The key a body picture drawn in `colors` is kept under, next to the
/// plain pictures.
pub fn colored_body_key(body: &str, colors: &[u8; 5]) -> String {
    format!("{body}#colors={},{},{},{},{}", colors[0], colors[1], colors[2], colors[3], colors[4])
}

/// The colour slot a command or chat word names: `setskincolor` / `setskin`
/// is 0, and so on to the belt (4).
pub fn color_slot(name: &str) -> Option<usize> {
    match name {
        "setskincolor" | "setskin" => Some(0),
        "setcoatcolor" | "setcoat" => Some(1),
        "setsleevecolor" | "setsleeves" => Some(2),
        "setshoecolor" | "setshoes" => Some(3),
        "setbeltcolor" | "setbelt" => Some(4),
        _ => None,
    }
}

/// A colour by name, as GServer reads the chat commands (`grey` is `gray`).
pub fn color_by_name(name: &str) -> Option<u8> {
    let n = name.trim().to_ascii_lowercase();
    let n = if n == "grey" { "gray".to_string() } else { n };
    COLORS.iter().position(|c| *c == n).map(|i| i as u8)
}

/// Prop 13 (COLORS): five GCHARs.
pub fn colors_from_prop(raw: &[u8]) -> Option<[u8; 5]> {
    let b = raw.get(..5)?;
    Some([0, 1, 2, 3, 4].map(|i| b[i].wrapping_sub(0x20)))
}

/// A change of looks typed in chat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatLook {
    Head(String),
    Body(String),
    Color { slot: usize, color: u8 },
}

impl ChatLook {
    /// The line as the GServer chat command (`sethead head927`), which is
    /// what a server acts on.
    pub fn server_line(&self) -> String {
        match self {
            ChatLook::Head(h) => format!("sethead {h}"),
            ChatLook::Body(b) => format!("setbody {b}"),
            ChatLook::Color { slot, color } => {
                let word = ["setskin", "setcoat", "setsleeves", "setshoes", "setbelt"][*slot];
                format!("{word} {}", COLORS[usize::from(*color)])
            }
        }
    }
}

/// Read a looks command from a chat line. The GServer forms
/// (`sethead head927`, `setbody body13.png`, `setskin orange`, `setcoat`,
/// `setsleeves`, `setshoes`, `setbelt`) are taken, and so are the spaced
/// forms `set head head927` and `set body body13`. A colour that is not in
/// the list is no command.
pub fn parse_chat_look(line: &str) -> Option<ChatLook> {
    let words: Vec<&str> = line.split_whitespace().collect();
    let (cmd, arg) = match words.as_slice() {
        ["set", what, arg] => (format!("set{}", what.to_ascii_lowercase()), *arg),
        [cmd, arg] => (cmd.to_ascii_lowercase(), *arg),
        _ => return None,
    };
    match cmd.as_str() {
        "sethead" => Some(ChatLook::Head(arg.to_string())),
        "setbody" => Some(ChatLook::Body(arg.to_string())),
        c => {
            let slot = color_slot(c).filter(|_| !c.ends_with("color"))?;
            Some(ChatLook::Color { slot, color: color_by_name(arg)? })
        }
    }
}

/// A picture name as typed, with `.png` added when there is no extension
/// (GServer tries `.png`, `.mng` and `.gif`; Seihai draws only PNG).
pub fn with_image_ext(name: &str) -> String {
    let lower = name.to_ascii_lowercase();
    if [".png", ".gif", ".mng"].iter().any(|e| lower.ends_with(e)) {
        name.to_string()
    } else {
        format!("{name}.png")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recolor_puts_each_slot_in_its_palette_entry() {
        let r = body_recolor(&[2, 0, 4, 5, 10]);
        assert_eq!(
            r,
            vec![(5, [255, 173, 107]), (6, [255, 255, 255]), (4, [255, 0, 0]), (3, [206, 24, 41]), (1, [0, 0, 255]), (0, [0, 0, 8])]
        );
    }

    #[test]
    fn chat_forms() {
        assert_eq!(parse_chat_look("set head head927"), Some(ChatLook::Head("head927".into())));
        assert_eq!(parse_chat_look("set body body13"), Some(ChatLook::Body("body13".into())));
        assert_eq!(parse_chat_look("sethead head927.png"), Some(ChatLook::Head("head927.png".into())));
        assert_eq!(parse_chat_look("setcoat Grey"), Some(ChatLook::Color { slot: 1, color: 17 }));
        assert_eq!(parse_chat_look("setbelt black"), Some(ChatLook::Color { slot: 4, color: 18 }));
        assert_eq!(parse_chat_look("setcoat plaid"), None);
        assert_eq!(parse_chat_look("setcoatcolor red"), None);
        assert_eq!(parse_chat_look("hello there"), None);
        assert_eq!(parse_chat_look("sethead"), None);
        assert_eq!(ChatLook::Head("head927".into()).server_line(), "sethead head927");
        assert_eq!(ChatLook::Color { slot: 2, color: 4 }.server_line(), "setsleeves red");
    }

    #[test]
    fn colors_prop_and_names() {
        assert_eq!(colors_from_prop(&[0x22, 0x20, 0x2a, 0x24, 0x32]), Some([2, 0, 10, 4, 18]));
        assert_eq!(colors_from_prop(&[0x22]), None);
        assert_eq!(with_image_ext("head927"), "head927.png");
        assert_eq!(with_image_ext("body13.PNG"), "body13.PNG");
        assert_eq!(Looks::default().body_name(), "body.png");
    }
}

#[cfg(test)]
mod default_body {
    use super::*;
    use crate::assets::{read_png_rgba, read_png_rgba_recolored};

    /// Seihai's own body picture keeps the 2.05 palette layout, so the
    /// colours reach the right parts: the idle body facing down has coat,
    /// sleeve, skin, shoe and belt pixels in the colours asked for, and the
    /// clear colour stays clear.
    #[test]
    fn default_body_takes_the_colours() {
        let data = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/default/bodies/body.png")).unwrap();
        let colors = [13, 7, 14, 1, 10];
        let (w, _, plain) = read_png_rgba(&data).unwrap();
        let (_, _, px) = read_png_rgba_recolored(&data, &body_recolor(&colors)).unwrap();
        let cell = |x: u32, y: u32| {
            let i = ((y * w + 64 + x) * 4) as usize; // down-facing idle, column 2
            (px[i..i + 4].to_vec(), plain[i + 3])
        };
        let rgba = |c: u8| {
            let [r, g, b] = COLOR_RGB[usize::from(c)];
            vec![r, g, b, 255]
        };
        let mut seen = [false; 5];
        for y in 0..32 {
            for x in 0..32 {
                let (p, a) = cell(x, y);
                if a == 0 {
                    assert_eq!(p[3], 0);
                    continue;
                }
                for (slot, &c) in colors.iter().enumerate() {
                    seen[slot] |= p == rgba(c);
                }
            }
        }
        assert_eq!(seen, [true; 5], "skin, coat, sleeves, shoes, belt");
    }
}
