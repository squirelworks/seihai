//! The fields of a 3.x level as the server sent them: the board, its
//! links, chests and signs, the player's props and the server NPCs' props.
//! Logged under the level line; nothing here is drawn or acted on.
//!
//! Only a build with a Torque profile ([`crate::torque::of`]) logs them, so
//! the dump stays on the 3.x line. The values are the core readers'
//! (`parse_player_props_in` with the build's own prop set, `NpcProps`);
//! a player prop is shown as its wire bytes, with the text when it is one.

use seihai_core::assets::{tile_id, Level};
use seihai_core::difftest::names::prop_name;
use seihai_core::protocol::npcprops::{Head, NpcProps};
use seihai_core::protocol::{parse_player_props_in, ClientVersion, PlayerProp, SessionState};
use std::collections::BTreeSet;

/// The log lines for `level` and what `st` holds for it, or none when
/// `version` has no Torque profile.
pub fn lines(version: ClientVersion, level: &Level, name: &str, st: &SessionState) -> Vec<String> {
    if crate::torque::of(version).is_none() {
        return Vec::new();
    }
    let mut out = vec![board_line(level)];
    for l in &level.links {
        out.push(format!("  link to {} at ({}, {}) {}x{}", l.dest, l.x, l.y, l.width, l.height));
    }
    for c in &level.chests {
        out.push(format!("  chest at ({}, {}): {}, sign {}{}", c.x, c.y, c.item, c.sign_index, if c.opened { ", opened" } else { "" }));
    }
    for s in &level.signs {
        out.push(format!("  sign at ({}, {}): {} bytes", s.x, s.y, s.text.len()));
    }
    let props = parse_player_props_in(&st.player_props, st.bow_prop10, st.own_prop_ids.as_ref());
    out.push(format!("player props: {}", props.len()));
    out.extend(props.iter().map(|p| format!("  {}", player_prop(p))));
    let here = |l: &Option<String>| l.as_deref().is_none_or(|l| l.eq_ignore_ascii_case(name));
    for (id, (_, p)) in st.server_npcs.iter().filter(|(_, (l, _))| here(l)) {
        out.push(npc_line(*id, p));
    }
    out
}

/// The board in one line: its tile count and the distinct tiles in it.
fn board_line(level: &Level) -> String {
    let tiles: Vec<u16> = level.boards.iter().flat_map(|r| r.tiles.iter().map(|&t| tile_id(t))).collect();
    let distinct: BTreeSet<u16> = tiles.iter().copied().collect();
    let shown: Vec<String> = distinct.iter().take(8).map(u16::to_string).collect();
    let more = if distinct.len() > 8 { ", ..." } else { "" };
    format!(
        "board: {} tiles, {} distinct [{}{more}], {} links, {} chests, {} signs",
        tiles.len(),
        distinct.len(),
        shown.join(", "),
        level.links.len(),
        level.chests.len(),
        level.signs.len()
    )
}

/// One player prop: id, name, wire bytes, then the GCHAR value of a
/// one-byte prop, or the text after a GCHAR length when the value is
/// exactly that.
fn player_prop(p: &PlayerProp) -> String {
    let name = prop_name(p.id as u32).unwrap_or("?");
    let hex: String = p.raw.iter().map(|b| format!("{b:02x}")).collect();
    let text = p
        .raw
        .split_first()
        .filter(|(n, rest)| !rest.is_empty() && n.wrapping_sub(0x20) as usize == rest.len())
        .filter(|(_, rest)| rest.iter().all(|b| (0x20..0x7F).contains(b)))
        .map(|(_, rest)| format!(" {:?}", String::from_utf8_lossy(rest)))
        .or_else(|| (p.raw.len() == 1).then(|| format!(" = {}", p.raw[0].wrapping_sub(0x20))))
        .unwrap_or_default();
    format!("{} {name}: {hex}{text}", p.id)
}

/// One server NPC: the props its records carried, merged.
fn npc_line(id: u32, p: &NpcProps) -> String {
    let mut f = Vec::new();
    if let (Some(x), Some(y)) = (p.x, p.y) {
        f.push(format!("at ({x}, {y})"));
    }
    if let Some(v) = &p.image {
        f.push(format!("image {v:?}"));
    }
    if let Some(v) = &p.gani {
        f.push(format!("gani {v:?}"));
    }
    if let Some(v) = p.sprite {
        f.push(format!("sprite {v}"));
    }
    if let Some(v) = &p.nickname {
        f.push(format!("nick {v:?}"));
    }
    match &p.head {
        Some(Head::Index(i)) => f.push(format!("head {i}")),
        Some(Head::Name(n)) => f.push(format!("head {n:?}")),
        None => {}
    }
    if let Some(v) = &p.body {
        f.push(format!("body {v:?}"));
    }
    if let Some(c) = p.colors {
        f.push(format!("colors {c:?}"));
    }
    if let Some(v) = p.vis_flags {
        f.push(format!("vis {v}"));
    }
    if let Some(v) = p.block_flags {
        f.push(format!("block {v}"));
    }
    if let Some(v) = &p.message {
        f.push(format!("message {v:?}"));
    }
    if let Some(v) = &p.script {
        f.push(format!("GS1 script {} bytes", v.len()));
    }
    format!("npc {id} props: {}", if f.is_empty() { "none".into() } else { f.join(", ") })
}
