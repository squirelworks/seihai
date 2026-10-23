//! Offline play support.
//!
//! Allows loading and simulating levels, NPCs, and a local player
//! without a network connection. Behavior is provisional and will
//! be refined against the original client’s offline/tutorial paths.

use crate::assets::{GaniPlayer, Level, TileTypes};
use crate::protocol::{BoardModify, PlayerUpdate};
use std::collections::{HashMap, HashSet};

/// Facing, in Graal order (ganis and the 2.05 wire): 0 up, 1 left,
/// 2 down, 3 right.
pub const DIR_NORTH: u8 = 0;
pub const DIR_WEST: u8 = 1;
pub const DIR_SOUTH: u8 = 2;
pub const DIR_EAST: u8 = 3;

/// A locally simulated game session (no server).
#[derive(Debug)]
pub struct OfflineSession {
    pub current_level: Option<Level>,
    pub tiletypes: Option<TileTypes>,
    pub player_x: f32,
    pub player_y: f32,
    pub player_dir: u8,
    /// True when the player is standing on a water tile (type == 11).
    pub in_water: bool,
    /// Optional walk animation.
    pub walk_gani: Option<GaniPlayer>,
    /// Player health (for `hurt`).
    pub player_health: i32,
    /// Last message from `say` / `message`.
    pub last_message: Option<String>,
    /// Flags set by `set` / `unset`.
    pub flags: HashMap<String, bool>,
    /// String variables (`setstring` / `addstring`).
    pub strings: HashMap<String, String>,
    /// Player properties (`setplayerprop`).
    pub props: HashMap<String, String>,
    /// Pending link destination if the player walked into a LINK area.
    pub pending_link: Option<(String, f32, f32)>,
    /// Rupees (gralats).
    pub rupees: i32,
    pub bombs: i32,
    pub darts: i32,
    /// Sword power 1–4 and shield power 1–3.
    pub sword_power: u8,
    pub shield_power: u8,
    /// Whether the glove is held, and its power.
    pub glove_power: u8,
    /// Heart containers, in half hearts as Graal counts them.
    pub max_hearts: i32,
    /// Chests opened in the current level, by tile position.
    pub opened_chests: HashSet<(i32, i32)>,
    /// Sign or chest text being read. The game waits until it is closed.
    pub reading: Option<String>,
}

/// What an interact press did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Interaction {
    /// A closed chest was opened at `(x, y)`; `got` describes the item.
    Chest { x: i32, y: i32, item: String, got: String },
    /// A sign is being read.
    Sign(String),
    Nothing,
}

impl OfflineSession {
    pub fn new() -> Self {
        Self {
            current_level: None,
            tiletypes: None,
            player_x: 30.0,
            player_y: 30.0,
            player_dir: DIR_SOUTH,
            in_water: false,
            walk_gani: None,
            player_health: 100,
            last_message: None,
            flags: HashMap::new(),
            strings: HashMap::new(),
            props: HashMap::new(),
            pending_link: None,
            rupees: 0,
            bombs: 10,
            darts: 10,
            sword_power: 1,
            shield_power: 1,
            glove_power: 0,
            max_hearts: 6,
            opened_chests: HashSet::new(),
            reading: None,
        }
    }

    /// The point one tile ahead of the player's feet, in tiles. The feet
    /// are the collision point used by [`Self::try_move`].
    pub fn facing_point(&self) -> (f32, f32) {
        let (fx, fy) = (self.player_x + 2.0, self.player_y + 2.75);
        match self.player_dir {
            DIR_NORTH => (fx, fy - 1.0),
            DIR_WEST => (fx - 1.0, fy - 0.5),
            DIR_EAST => (fx + 1.0, fy - 0.5),
            _ => (fx, fy + 1.0),
        }
    }

    /// Interact with what the player faces: open a closed chest (2×2
    /// tiles), or read a sign (2×1 tiles). A chest is tried first.
    pub fn interact(&mut self) -> Interaction {
        let Some(level) = &self.current_level else {
            return Interaction::Nothing;
        };
        let (px, py) = self.facing_point();
        let inside = |x: i32, y: i32, w: f32, h: f32| {
            px >= x as f32 && px < x as f32 + w && py >= y as f32 && py < y as f32 + h
        };
        let chest = level
            .chests
            .iter()
            .find(|c| inside(c.x, c.y, 2.0, 2.0) && !self.opened_chests.contains(&(c.x, c.y)))
            .cloned();
        if let Some(c) = chest {
            // A chest's sign index points into the level's signs.
            let note = usize::try_from(c.sign_index)
                .ok()
                .filter(|_| c.sign_index >= 0)
                .and_then(|i| level.signs.get(i))
                .map(|s| s.text.clone());
            self.opened_chests.insert((c.x, c.y));
            let got = self.give_item(&c.item);
            self.reading = note;
            return Interaction::Chest { x: c.x, y: c.y, item: c.item, got };
        }
        if let Some(sign) = level.signs.iter().find(|s| inside(s.x, s.y, 2.0, 1.0)) {
            let text = sign.text.clone();
            self.reading = Some(text.clone());
            return Interaction::Sign(text);
        }
        Interaction::Nothing
    }

    /// Apply a chest item. Returns a short description.
    ///
    /// Item names are the CHEST record's (`greenrupee`, `fullheart`, ...).
    pub fn give_item(&mut self, item: &str) -> String {
        let item = item.to_ascii_lowercase();
        let mut gain = |what: &mut i32, n: i32, label: &str| {
            *what += n;
            format!("{n} {label}")
        };
        match item.as_str() {
            "greenrupee" => gain(&mut self.rupees, 1, "rupee"),
            "bluerupee" => gain(&mut self.rupees, 5, "rupees"),
            "redrupee" => gain(&mut self.rupees, 30, "rupees"),
            "goldrupee" => gain(&mut self.rupees, 100, "rupees"),
            "bombs" => gain(&mut self.bombs, 5, "bombs"),
            "darts" => gain(&mut self.darts, 5, "arrows"),
            "heart" => {
                self.player_health = 100;
                "a heart".into()
            }
            "fullheart" => {
                self.max_hearts = (self.max_hearts + 2).min(40);
                self.player_health = 100;
                "a heart container".into()
            }
            "spinattack" => {
                self.flags.insert("spinattack".into(), true);
                "the spin attack".into()
            }
            "glove1" | "glove2" => {
                self.glove_power = self.glove_power.max(if item == "glove1" { 1 } else { 2 });
                "a glove".into()
            }
            "sword" | "battleaxe" | "lizardsword" | "goldensword" => {
                let p = match item.as_str() {
                    "sword" => 1,
                    "battleaxe" => 2,
                    "lizardsword" => 3,
                    _ => 4,
                };
                self.sword_power = self.sword_power.max(p);
                format!("sword {p}")
            }
            "shield" | "mirrorshield" | "lizardshield" => {
                let p = match item.as_str() {
                    "shield" => 1,
                    "mirrorshield" => 2,
                    _ => 3,
                };
                self.shield_power = self.shield_power.max(p);
                format!("shield {p}")
            }
            other => {
                self.flags.insert(format!("item:{other}"), true);
                other.to_string()
            }
        }
    }

    /// Load a .nw level from disk.
    pub fn load_level<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<(), String> {
        let level = Level::load(path)?;
        self.current_level = Some(level);
        self.opened_chests.clear();
        self.reading = None;
        Ok(())
    }

    /// Load the 4096-byte collision table from a text tiledefs file.
    pub fn load_tiletypes<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<(), String> {
        self.tiletypes = Some(TileTypes::load(path)?);
        Ok(())
    }

    /// Load type-0 collision from `arrays.dat` (first array = nowall list).
    /// Tiles in the list get type 0 (passable); everything else gets type 22.
    pub fn load_tiletypes_arrays<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<(), String> {
        self.tiletypes = Some(TileTypes::load_arrays(path)?);
        Ok(())
    }

    /// Current player state as a protocol update.
    pub fn player_update(&self) -> PlayerUpdate {
        PlayerUpdate {
            x: self.player_x,
            y: self.player_y,
            dir: self.player_dir,
        }
    }

    /// Apply a GS1 command against this offline session.
    ///
    /// Handles the high-frequency commands from the scraped set:
    /// `hide`/`show` (NPC visibility), `warp` (local position),
    /// `setlevel` (noted), `hurt` (health), `say`/`message` (last message),
    /// `set`/`unset` (flags).
    /// Returns true if the command was recognized and applied.
    pub fn handle_command(&mut self, name: &str, args: &[crate::scripting::Value]) -> bool {
        match name {
            "hide" | "hideimg" => {
                if let Some(level) = self.current_level.as_mut() {
                    for npc in &mut level.npcs {
                        npc.visible = false;
                    }
                }
                true
            }
            "show" | "showimg" => {
                if let Some(level) = self.current_level.as_mut() {
                    for npc in &mut level.npcs {
                        npc.visible = true;
                    }
                }
                true
            }
            "warp" => {
                if args.len() >= 2 {
                    if let (crate::scripting::Value::Num(x), crate::scripting::Value::Num(y)) =
                        (&args[0], &args[1])
                    {
                        self.player_x = *x;
                        self.player_y = *y;
                    }
                }
                true
            }
            "setlevel" => {
                // Level switch is recorded; actual load is done by the caller
                // once a level path or name is resolved.
                true
            }
            "hurt" => {
                let amt = args
                    .first()
                    .and_then(|v| match v {
                        crate::scripting::Value::Num(n) => Some(*n as i32),
                        _ => None,
                    })
                    .unwrap_or(1);
                self.player_health = (self.player_health - amt).max(0);
                true
            }
            "say" | "message" => {
                if let Some(crate::scripting::Value::Str(s)) = args.first() {
                    self.last_message = Some(s.clone());
                }
                true
            }
            "set" => {
                if let Some(crate::scripting::Value::Str(name)) = args.first() {
                    self.flags.insert(name.clone(), true);
                }
                true
            }
            "unset" => {
                if let Some(crate::scripting::Value::Str(name)) = args.first() {
                    self.flags.insert(name.clone(), false);
                }
                true
            }
            "setstring" => {
                if args.len() >= 2 {
                    if let (crate::scripting::Value::Str(name), crate::scripting::Value::Str(val)) =
                        (&args[0], &args[1])
                    {
                        self.strings.insert(name.clone(), val.clone());
                    }
                }
                true
            }
            "addstring" => {
                if args.len() >= 2 {
                    if let (crate::scripting::Value::Str(name), crate::scripting::Value::Str(val)) =
                        (&args[0], &args[1])
                    {
                        let entry = self.strings.entry(name.clone()).or_default();
                        entry.push_str(val);
                    }
                }
                true
            }
            "setplayerprop" => {
                if args.len() >= 2 {
                    if let (crate::scripting::Value::Str(name), crate::scripting::Value::Str(val)) =
                        (&args[0], &args[1])
                    {
                        self.props.insert(name.clone(), val.clone());
                    }
                }
                true
            }
            "setimg" | "setshape" => {
                // Image/shape change is recorded as a prop for now.
                if let Some(crate::scripting::Value::Str(name)) = args.first() {
                    self.props.insert("image".into(), name.clone());
                }
                true
            }
            "move" => {
                // Relative move: args are dx, dy.
                if args.len() >= 2 {
                    if let (crate::scripting::Value::Num(dx), crate::scripting::Value::Num(dy)) =
                        (&args[0], &args[1])
                    {
                        self.player_x += *dx;
                        self.player_y += *dy;
                    }
                }
                true
            }
            _ => false,
        }
    }

    /// Place a named stamp from an object set onto the current level.
    /// Returns true if the stamp was found and written.
    pub fn place_named_stamp(&mut self, set: &crate::assets::ObjectSet, name: &str, x: i32, y: i32) -> bool {
        if let Some(stamp) = set.find(name) {
            if let Some(level) = self.current_level.as_mut() {
                level.place_stamp(stamp, x, y);
                return true;
            }
        }
        false
    }
    /// If so, record the destination in `pending_link`.
    /// Returns true if a link was entered.
    pub fn check_links(&mut self) -> bool {
        // Use the foot point, not the sprite top-left.
        let (fx, fy) = (self.player_x + 2.0, self.player_y + 2.75);
        let px = fx as i32;
        let py = fy as i32;
        let dest = if let Some(level) = &self.current_level {
            level.links.iter().find_map(|link| {
                if px >= link.x
                    && px < link.x + link.width
                    && py >= link.y
                    && py < link.y + link.height
                {
                    Some((
                        link.dest.clone(),
                        link.new_x.unwrap_or(self.player_x),
                        link.new_y.unwrap_or(self.player_y),
                    ))
                } else {
                    None
                }
            })
        } else {
            None
        };
        self.pending_link = dest;
        self.pending_link.is_some()
    }

    /// Check for player-touch-NPC events.
    ///
    /// If the player is within 1 tile of a visible NPC whose script contains
    /// a simple `hide` or `warp` pattern, apply the corresponding command.
    /// Returns the number of NPCs that fired.
    pub fn check_touch(&mut self) -> usize {
        let mut fired = 0;
        let px = self.player_x;
        let py = self.player_y;
        let mut actions: Vec<(String, Vec<crate::scripting::Value>)> = Vec::new();

        if let Some(level) = &self.current_level {
            for npc in &level.npcs {
                if !npc.visible {
                    continue;
                }
                let dx = (npc.x - px).abs();
                let dy = (npc.y - py).abs();
                if dx <= 1.0 && dy <= 1.0 {
                    let script = npc.script.to_lowercase();
                    if script.contains("hide") {
                        actions.push(("hide".into(), vec![]));
                    }
                    if script.contains("warp") {
                        // Minimal: warp to a nearby open cell if no args parsed.
                        actions.push((
                            "warp".into(),
                            vec![
                                crate::scripting::Value::Num(px + 2.0),
                                crate::scripting::Value::Num(py),
                            ],
                        ));
                    }
                }
            }
        }

        for (name, args) in actions {
            if self.handle_command(&name, &args) {
                fired += 1;
            }
        }
        fired
    }

    /// Move the player by (dx, dy) if the destination is not a wall.
    /// Uses the single-point foot test from the observed client path:
    /// test point is (pixel + move + 32, pixel + move + 44) in 16px units.
    /// Updates facing from the dominant axis.
    /// Returns the emitted `PlayerUpdate` if the move succeeded.
    pub fn try_move(&mut self, dx: f32, dy: f32) -> Option<PlayerUpdate> {
        if dx == 0.0 && dy == 0.0 {
            return None;
        }
        // Facing from dominant axis.
        if dx.abs() > dy.abs() {
            self.player_dir = if dx > 0.0 { DIR_EAST } else { DIR_WEST };
        } else {
            self.player_dir = if dy > 0.0 { DIR_SOUTH } else { DIR_NORTH };
        }

        // Single-point foot test (observed client offsets).
        // Convert tile-space position to pixel, add move, then +32/+44, divide by 16.
        let px = self.player_x * 16.0;
        let py = self.player_y * 16.0;
        let test_px = px + dx * 16.0 + 32.0;
        let test_py = py + dy * 16.0 + 44.0;
        let test_x = test_px / 16.0;
        let test_y = test_py / 16.0;

        if let (Some(level), Some(types)) = (&self.current_level, &self.tiletypes) {
            if level.is_on_wall(types, test_x, test_y) {
                return None;
            }
        }
        // Chests are solid, 2×2 tiles.
        if let Some(level) = &self.current_level {
            let blocked = level.chests.iter().any(|c| {
                test_x >= c.x as f32
                    && test_x < c.x as f32 + 2.0
                    && test_y >= c.y as f32
                    && test_y < c.y as f32 + 2.0
            });
            if blocked {
                return None;
            }
        }

        self.player_x += dx;
        self.player_y += dy;
        self.in_water = self
            .current_level
            .as_ref()
            .zip(self.tiletypes.as_ref())
            .map(|(level, types)| level.is_on_water(types, self.player_x, self.player_y))
            .unwrap_or(false);
        if let Some(gani) = self.walk_gani.as_mut() {
            gani.step();
        }
        Some(self.player_update())
    }

    /// Attach a walk GANI. Stepped on every successful move.
    pub fn set_walk_gani(&mut self, gani: GaniPlayer) {
        self.walk_gani = Some(gani);
    }

    /// Current walk-frame index, if a GANI is attached.
    pub fn walk_frame(&self) -> Option<usize> {
        self.walk_gani.as_ref().map(|g| g.frame)
    }

    /// Apply a board update to the current level (if any).
    pub fn apply_board_update(&mut self, m: &BoardModify) {
        if let Some(level) = self.current_level.as_mut() {
            level.apply_board_modify(m);
        }
    }

    /// Whether a level is currently loaded.
    pub fn has_level(&self) -> bool {
        self.current_level.is_some()
    }
}

impl Default for OfflineSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chest_and_sign_interact() {
        let text = "GLEVNW01\nCHEST 20 24 greenrupee 0\nCHEST 10 10 fullheart -1\nSIGN 30 10\nWelcome\nSIGNEND\n";
        let mut s = OfflineSession::new();
        s.current_level = Some(Level::parse(text).unwrap());

        // Walk up into the chest at (20, 24): it stops the player below it.
        s.player_x = 18.5;
        s.player_y = 25.0;
        while s.try_move(0.0, -0.5).is_some() {}
        assert_eq!(s.player_y, 23.5);
        assert_eq!(s.player_dir, DIR_NORTH);
        let got = s.interact();
        assert_eq!(
            got,
            Interaction::Chest { x: 20, y: 24, item: "greenrupee".into(), got: "1 rupee".into() }
        );
        assert_eq!(s.rupees, 1);
        // Sign 0 is the chest's note.
        assert_eq!(s.reading.as_deref(), Some("Welcome"));
        // Opened once only.
        s.reading = None;
        assert_eq!(s.interact(), Interaction::Nothing);

        // Sign at (30, 10), read from below.
        s.player_x = 28.5;
        s.player_y = 8.5;
        assert_eq!(s.interact(), Interaction::Sign("Welcome".into()));

        // A heart container with no note.
        s.reading = None;
        s.player_x = 8.5;
        s.player_y = 9.0;
        assert!(matches!(s.interact(), Interaction::Chest { x: 10, y: 10, .. }));
        assert_eq!(s.max_hearts, 8);
        assert!(s.reading.is_none());
    }
}
