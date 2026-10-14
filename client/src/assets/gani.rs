//! GANI parser (SPRITE definitions + ANI frames).

use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct GaniSprite {
    pub id: u32,
    pub sheet: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub desc: String,
}

/// One placed sprite inside an animation frame.
#[derive(Debug, Clone)]
pub struct GaniPlacement {
    pub sprite_id: u32,
    pub x: i32,
    pub y: i32,
}

/// One animation frame (list of placements, optional sound).
#[derive(Debug, Clone)]
pub struct GaniFrame {
    pub placements: Vec<GaniPlacement>,
    pub sound: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Gani {
    pub sprites: Vec<GaniSprite>,
    pub frames: Vec<GaniFrame>,
    pub default_attr1: Option<String>,
    pub default_head: Option<String>,
    pub default_body: Option<String>,
    pub raw: String,
}

impl Gani {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        Self::parse(&text)
    }

    pub fn parse(text: &str) -> Result<Self, String> {
        let mut sprites = Vec::new();
        let mut frames = Vec::new();
        let mut default_attr1 = None;
        let mut default_head = None;
        let mut default_body = None;

        let mut in_ani = false;
        let mut current_placements: Vec<GaniPlacement> = Vec::new();

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            if line == "ANI" {
                in_ani = true;
                continue;
            }
            if line == "ANIEND" {
                if !current_placements.is_empty() {
                    frames.push(GaniFrame {
                        placements: std::mem::take(&mut current_placements),
                        sound: None,
                    });
                }
                in_ani = false;
                continue;
            }

            if line.starts_with("SPRITE ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 7 {
                    sprites.push(GaniSprite {
                        id: parts[1].parse().unwrap_or(0),
                        sheet: parts[2].to_string(),
                        x: parts[3].parse().unwrap_or(0),
                        y: parts[4].parse().unwrap_or(0),
                        w: parts[5].parse().unwrap_or(0),
                        h: parts[6].parse().unwrap_or(0),
                        desc: parts.get(7..).map(|p| p.join(" ")).unwrap_or_default(),
                    });
                }
                continue;
            }

            if line.starts_with("DEFAULTATTR1 ") {
                default_attr1 = Some(line[13..].trim().to_string());
                continue;
            }
            if line.starts_with("DEFAULTHEAD ") {
                default_head = Some(line[12..].trim().to_string());
                continue;
            }
            if line.starts_with("DEFAULTBODY ") {
                default_body = Some(line[12..].trim().to_string());
                continue;
            }

            if in_ani {
                if line.starts_with("PLAYSOUND ") {
                    // attach sound to last frame if we have one
                    if let Some(last) = frames.last_mut() {
                        last.sound = Some(line[10..].trim().to_string());
                    }
                    continue;
                }

                // Frame line: "id x y, id x y, ..."
                if line.contains(',') || line.split_whitespace().count() >= 3 {
                    if !current_placements.is_empty() {
                        frames.push(GaniFrame {
                            placements: std::mem::take(&mut current_placements),
                            sound: None,
                        });
                    }
                    for chunk in line.split(',') {
                        let parts: Vec<&str> = chunk.split_whitespace().collect();
                        if parts.len() >= 3 {
                            current_placements.push(GaniPlacement {
                                sprite_id: parts[0].parse().unwrap_or(0),
                                x: parts[1].parse().unwrap_or(0),
                                y: parts[2].parse().unwrap_or(0),
                            });
                        }
                    }
                }
            }
        }

        // flush last frame
        if !current_placements.is_empty() {
            frames.push(GaniFrame {
                placements: current_placements,
                sound: None,
            });
        }

        Ok(Gani {
            sprites,
            frames,
            default_attr1,
            default_head,
            default_body,
            raw: text.to_string(),
        })
    }

    /// Number of animation frames.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Frame at `index`, wrapping. None if there are no frames.
    pub fn frame(&self, index: usize) -> Option<&GaniFrame> {
        if self.frames.is_empty() {
            None
        } else {
            self.frames.get(index % self.frames.len())
        }
    }

    /// Advance a frame index by one step, wrapping.
    pub fn step(&self, index: usize) -> usize {
        if self.frames.is_empty() {
            0
        } else {
            (index + 1) % self.frames.len()
        }
    }
}

/// Playback state for a loaded GANI.
#[derive(Debug, Clone)]
pub struct GaniPlayer {
    pub gani: Gani,
    pub frame: usize,
}

impl GaniPlayer {
    pub fn new(gani: Gani) -> Self {
        Self { gani, frame: 0 }
    }

    /// Current frame placements.
    pub fn current(&self) -> Option<&GaniFrame> {
        self.gani.frame(self.frame)
    }

    /// Advance one frame. Returns the new index.
    pub fn step(&mut self) -> usize {
        self.frame = self.gani.step(self.frame);
        self.frame
    }

    /// Reset to the first frame.
    pub fn reset(&mut self) {
        self.frame = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stepper_wraps() {
        let text = "\
ANI
1 0 0, 2 1 1
3 0 0
ANIEND
";
        let g = Gani::parse(text).unwrap();
        assert_eq!(g.frame_count(), 2);
        let mut p = GaniPlayer::new(g);
        assert_eq!(p.frame, 0);
        p.step();
        assert_eq!(p.frame, 1);
        p.step();
        assert_eq!(p.frame, 0);
    }
}

