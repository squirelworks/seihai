//! Version profiles for the level editor.
//!
//! A profile is the target the editor is imitating: which level format it
//! reads and writes, and (later) which features apply. Today only the 2.22
//! and 2.17.1 text (`GLEVNW01` / `.nw`) profiles actually open and save.
//! Binary profiles are listed so the picker and CLI can name them; their
//! I/O is parked until the matching reader/writer is wired.

use crate::EditError;

/// Whether this profile's level I/O is ready to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoStatus {
    /// Open / save work through the current text path.
    Ready,
    /// Listed in the picker, but open / save refuse with a clear message.
    Parked,
}

/// The target profile: format and (later) feature switches.
///
/// The default and the only fully working path is [`VersionProfile::TextNw222`]
/// (2.22 `GLEVNW01` text). Binary profiles stay parked so the scaffold does
/// not block on GR-V1.05 reader/writer work landing elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VersionProfile {
    /// 2.22 client's built-in editor: `GLEVNW01` text (`.nw`).
    TextNw222,
    /// 2.17.1: same board and text rules as 2.22 (no difference found).
    TextNw217,
    /// Binary `GR-V1.00` (`.graal`).
    GrV100,
    /// Binary `GR-V1.01` (`.graal`).
    GrV101,
    /// Binary `GR-V1.02` (`.graal`).
    GrV102,
    /// Binary `GR-V1.03` (`.graal`) — the usual single-layer binary write.
    GrV103,
    /// Binary `GR-V1.05` multi-layer (`.graal`). Parked until core binary I/O lands.
    GrV105,
    /// Binary `Z3-V1.03` (`.zelda`).
    Z3V103,
    /// Binary `Z3-V1.04` (`.zelda`).
    Z3V104,
}

impl VersionProfile {
    /// Default profile: 2.22 text.
    pub const DEFAULT: VersionProfile = VersionProfile::TextNw222;

    /// Every profile the picker and `versions` list, in display order.
    pub const ALL: [VersionProfile; 9] = [
        VersionProfile::TextNw222,
        VersionProfile::TextNw217,
        VersionProfile::GrV100,
        VersionProfile::GrV101,
        VersionProfile::GrV102,
        VersionProfile::GrV103,
        VersionProfile::GrV105,
        VersionProfile::Z3V103,
        VersionProfile::Z3V104,
    ];

    /// Profiles whose open / save path works today.
    pub const WORKING: [VersionProfile; 2] = [VersionProfile::TextNw222, VersionProfile::TextNw217];

    /// Short id for the CLI (`--version` / `--profile`) and the combo box.
    pub fn name(self) -> &'static str {
        match self {
            VersionProfile::TextNw222 => "2.22",
            VersionProfile::TextNw217 => "2.17.1",
            VersionProfile::GrV100 => "GR-V1.00",
            VersionProfile::GrV101 => "GR-V1.01",
            VersionProfile::GrV102 => "GR-V1.02",
            VersionProfile::GrV103 => "GR-V1.03",
            VersionProfile::GrV105 => "GR-V1.05",
            VersionProfile::Z3V103 => "Z3-V1.03",
            VersionProfile::Z3V104 => "Z3-V1.04",
        }
    }

    /// Longer label for the window (format + extension).
    pub fn label(self) -> &'static str {
        match self {
            VersionProfile::TextNw222 => "2.22 text (.nw)",
            VersionProfile::TextNw217 => "2.17.1 text (.nw)",
            VersionProfile::GrV100 => "GR-V1.00 (.graal)",
            VersionProfile::GrV101 => "GR-V1.01 (.graal)",
            VersionProfile::GrV102 => "GR-V1.02 (.graal)",
            VersionProfile::GrV103 => "GR-V1.03 (.graal)",
            VersionProfile::GrV105 => "GR-V1.05 (.graal)",
            VersionProfile::Z3V103 => "Z3-V1.03 (.zelda)",
            VersionProfile::Z3V104 => "Z3-V1.04 (.zelda)",
        }
    }

    /// File magic this profile targets (header of the level bytes).
    pub fn magic(self) -> &'static [u8] {
        match self {
            VersionProfile::TextNw222 | VersionProfile::TextNw217 => b"GLEVNW01",
            VersionProfile::GrV100 => b"GR-V1.00",
            VersionProfile::GrV101 => b"GR-V1.01",
            VersionProfile::GrV102 => b"GR-V1.02",
            VersionProfile::GrV103 => b"GR-V1.03",
            VersionProfile::GrV105 => b"GR-V1.05",
            VersionProfile::Z3V103 => b"Z3-V1.03",
            VersionProfile::Z3V104 => b"Z3-V1.04",
        }
    }

    /// Usual extension for a new file of this profile.
    pub fn extension(self) -> &'static str {
        match self {
            VersionProfile::TextNw222 | VersionProfile::TextNw217 => "nw",
            VersionProfile::GrV100
            | VersionProfile::GrV101
            | VersionProfile::GrV102
            | VersionProfile::GrV103
            | VersionProfile::GrV105 => "graal",
            VersionProfile::Z3V103 | VersionProfile::Z3V104 => "zelda",
        }
    }

    /// Whether this profile uses the text board path.
    pub fn is_text(self) -> bool {
        matches!(self, VersionProfile::TextNw222 | VersionProfile::TextNw217)
    }

    /// Whether open / save for this profile are ready.
    pub fn io_status(self) -> IoStatus {
        if self.is_text() {
            IoStatus::Ready
        } else {
            IoStatus::Parked
        }
    }

    /// True when open / save use the working text path.
    pub fn io_ready(self) -> bool {
        self.io_status() == IoStatus::Ready
    }

    /// Refuse with a parked message when binary I/O is not wired yet.
    pub fn require_io_ready(self) -> Result<(), EditError> {
        if self.io_ready() {
            return Ok(());
        }
        let why = match self {
            VersionProfile::GrV105 => {
                "binary GR-V1.05 read/write is parked until the shared core lands it; the text 2.22 path stays the working default"
            }
            VersionProfile::GrV100
            | VersionProfile::GrV101
            | VersionProfile::GrV102
            | VersionProfile::GrV103 => {
                "binary GR-V1 read/write is not yet wired in the editor; pick 2.22 (text .nw) for now"
            }
            VersionProfile::Z3V103 | VersionProfile::Z3V104 => {
                "binary Z3-V1 read/write is not yet wired in the editor; pick 2.22 (text .nw) for now"
            }
            VersionProfile::TextNw222 | VersionProfile::TextNw217 => unreachable!("text is ready"),
        };
        Err(EditError::Version(format!(
            "profile {} is parked: {why}",
            self.name()
        )))
    }

    /// Parse a `--version` / `--profile` value (or a magic string).
    pub fn parse(text: &str) -> Result<VersionProfile, EditError> {
        let t = text.trim().trim_start_matches('v');
        let lower = t.to_ascii_lowercase();
        match lower.as_str() {
            "2.22" | "glevnw01" | "text" | "nw" => return Ok(VersionProfile::TextNw222),
            "2.17" | "2.17.1" => return Ok(VersionProfile::TextNw217),
            "gr-v1.00" | "gr-v1.0" | "1.00" => return Ok(VersionProfile::GrV100),
            "gr-v1.01" | "1.01" => return Ok(VersionProfile::GrV101),
            "gr-v1.02" | "1.02" => return Ok(VersionProfile::GrV102),
            "gr-v1.03" | "1.03" | "graal" => return Ok(VersionProfile::GrV103),
            "gr-v1.05" | "1.05" => return Ok(VersionProfile::GrV105),
            "z3-v1.03" => return Ok(VersionProfile::Z3V103),
            "z3-v1.04" | "zelda" => return Ok(VersionProfile::Z3V104),
            _ => {}
        }
        // Client-editor versions that are not format profiles yet keep a clear refusal.
        let nums: Vec<u32> = t.split('.').map_while(|p| p.parse().ok()).collect();
        let why = match (nums.first(), nums.get(1)) {
            (Some(2), Some(5)) if t == "2.05" || t == "2.5" => {
                "2.05 differs from 2.22 (no HEIGHTS block, no predefined NPCs, a *.graal-only open filter); not a format profile yet"
            }
            (Some(2), Some(31)) | (Some(2), Some(3)) if t.starts_with("2.3") => {
                "2.31 has no built-in editor (the standalone GraalEditor replaced it)"
            }
            (Some(2), Some(m)) if *m < 5 => "this build has no text level format at all",
            (Some(1), _) | (Some(0), _) => "this build has no text level format",
            (Some(2), Some(_)) => "this build is not covered by the editor notes",
            _ => {
                return Err(EditError::Version(format!(
                    "unknown profile \"{text}\"; supported: {}",
                    Self::names_csv()
                )));
            }
        };
        Err(EditError::Version(format!(
            "version {text} is not yet supported: {why}. Profiles: {}",
            Self::names_csv()
        )))
    }

    fn names_csv() -> String {
        Self::ALL
            .iter()
            .map(|p| p.name())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Guess a profile from the first bytes of a level file.
    pub fn detect(data: &[u8]) -> Option<VersionProfile> {
        for p in Self::ALL {
            if data.starts_with(p.magic()) {
                // TextNw217 shares the GLEVNW01 magic with TextNw222; prefer the default.
                if p == VersionProfile::TextNw217 {
                    continue;
                }
                return Some(p);
            }
        }
        None
    }
}

impl Default for VersionProfile {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Old name kept so call sites that still say `Version` keep compiling during the rename.
pub type Version = VersionProfile;
