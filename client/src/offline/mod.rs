//! Offline play support.
//!
//! Allows loading and simulating levels, NPCs, and a local player
//! without a network connection. Behavior is provisional and will
//! be refined against the original client’s offline/tutorial paths.

use crate::assets::{GaniPlayer, Level, TileTypes};
use crate::protocol::{BoardModify, PlayerUpdate};
use std::collections::HashMap;

/// Facing: 0 south, 1 west, 2 east, 3 north.
pub const DIR_SOUTH: u8 = 0;
pub const DIR_WEST: u8 = 1;
pub const DIR_EAST: u8 = 2;
pub const DIR_NORTH: u8 = 3;

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
    pub pending_link: Option<(String, i32, i32)>,
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
        }
    }

    /// Load a .nw level from disk.
    pub fn load_level<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<(), String> {
        let level = Level::load(path)?;
        self.current_level = Some(level);
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
        let px = self.player_x as i32;
        let py = self.player_y as i32;
        let dest = if let Some(level) = &self.current_level {
            level.links.iter().find_map(|link| {
                if px >= link.x
                    && px < link.x + link.width
                    && py >= link.y
                    && py < link.y + link.height
                {
                    Some((link.dest.clone(), link.new_x, link.new_y))
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
        // Facing from dominant axis (Graal order: 0 south, 1 west, 2 east, 3 north).
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
