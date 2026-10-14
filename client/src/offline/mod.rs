//! Offline play support.
//!
//! Allows loading and simulating levels, NPCs, and a local player
//! without a network connection. Behavior is provisional and will
//! be refined against the original client’s offline/tutorial paths.

use crate::assets::{GaniPlayer, Level, TileTypes};
use crate::protocol::{BoardModify, PlayerUpdate};

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
        }
    }

    /// Load a .nw level from disk.
    pub fn load_level<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<(), String> {
        let level = Level::load(path)?;
        self.current_level = Some(level);
        Ok(())
    }

    /// Load the 4096-byte collision table.
    pub fn load_tiletypes<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<(), String> {
        self.tiletypes = Some(TileTypes::load(path)?);
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

    /// Move the player by (dx, dy) if the destination is not a wall.
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

        let nx = self.player_x + dx;
        let ny = self.player_y + dy;

        if let (Some(level), Some(types)) = (&self.current_level, &self.tiletypes) {
            if level.is_on_wall(types, nx, ny) {
                return None;
            }
        }

        self.player_x = nx;
        self.player_y = ny;
        self.in_water = self
            .current_level
            .as_ref()
            .zip(self.tiletypes.as_ref())
            .map(|(level, types)| level.is_on_water(types, nx, ny))
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
