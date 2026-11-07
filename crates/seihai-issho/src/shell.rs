//! The v6 shell Issho owns. Data only: nothing here draws, opens a sound
//! device, runs GS2 bytecode, or connects.
//!
//! Shape of the unpacked 6.00 Windows images: one `quattro_lib` program.
//! OpenGL presents the frame after a Direct3D 9 capability check. Sound is
//! FMOD. The GUI shell is GS2 (`StartScript_GraalGui` and the dialogs).
//! The game connection is `TSocket`. World objects are the `TServer*`
//! types. `GuiGraal3DCtrl` is a playfield beside `GuiGraalCtrl`.
//! `horizSizing` and `GuiTilesCtrl` are absent in 6.00.
//!
//! The GS2 bytecode VM is not ported. Graal_analysis pull requests 276 and
//! 278 describe it for a later pass.

use crate::profile::{select_version, EngineProfile, SelectError};
use seihai_core::protocol::ClientVersion;

/// How the frame is presented. Direct3D 9 is the capability check, not this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presenter {
    OpenGl,
}

/// Adapter check that runs before the first frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityCheck {
    Direct3D9,
}

/// Live playfield controls. `GuiGraal3DCtrl` is not a Torque viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Playfield {
    GuiGraalCtrl,
    GuiGraal3DCtrl,
}

/// Draw path for every v6 profile. The scaffold does not issue GL calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Draw {
    pub presenter: Presenter,
    pub capability_check: CapabilityCheck,
    pub playfields: &'static [Playfield],
    /// `GuiTilesCtrl` is absent in 6.00. Tiles are a draw pass, not that control.
    pub gui_tiles_ctrl: bool,
    /// `horizSizing` is absent in 6.00.
    pub horiz_sizing: bool,
}

const PLAYFIELDS: &[Playfield] = &[Playfield::GuiGraalCtrl, Playfield::GuiGraal3DCtrl];

impl Draw {
    fn v6() -> Self {
        Self {
            presenter: Presenter::OpenGl,
            capability_check: CapabilityCheck::Direct3D9,
            playfields: PLAYFIELDS,
            gui_tiles_ctrl: false,
            horiz_sizing: false,
        }
    }
}

/// Sound slot. FMOD is the 6.00 backend. This scaffold does not open a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sound {
    pub backend: SoundBackend,
    pub device_open: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundBackend {
    Fmod,
}

impl Sound {
    fn v6() -> Self {
        Self {
            backend: SoundBackend::Fmod,
            device_open: false,
        }
    }
}

/// GS2 script shell. The names are the startup scripts. No bytecode runner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gs2Shell {
    pub scripts: &'static [&'static str],
}

const SHELL_SCRIPTS: &[&str] = &[
    "StartScript_GraalGui",
    "StartScript_Profiles",
    "StartScript_MessageBoxDialog",
    "StartScript_SetupDialog",
    "StartScript_SelectFileDialog",
    "StartScript_FileScanDialog",
];

impl Gs2Shell {
    fn v6() -> Self {
        Self {
            scripts: SHELL_SCRIPTS,
        }
    }
}

/// Game connection. Not Torque's `GameConnection`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TSocket;

impl TSocket {
    pub const NAME: &'static str = "TSocket";
}

/// A world object the GS2 shell sees. These are `TServer*` types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldObject {
    Player,
    Npc,
    Level,
    LevelLink,
    Weapon,
    Bomb,
    Carry,
    Chest,
    Horse,
    Sign,
    Leap,
    Flying,
    Extra,
}

impl WorldObject {
    pub fn script_name(self) -> &'static str {
        match self {
            Self::Player => "TServerPlayer",
            Self::Npc => "TServerNPC",
            Self::Level => "TServerLevel",
            Self::LevelLink => "TServerLevelLink",
            Self::Weapon => "TServerWeapon",
            Self::Bomb => "TServerBomb",
            Self::Carry => "TServerCarry",
            Self::Chest => "TServerChest",
            Self::Horse => "TServerHorse",
            Self::Sign => "TServerSign",
            Self::Leap => "TServerLeap",
            Self::Flying => "TServerFlying",
            Self::Extra => "TServerExtra",
        }
    }
}

const WORLD: &[WorldObject] = &[
    WorldObject::Player,
    WorldObject::Npc,
    WorldObject::Level,
    WorldObject::LevelLink,
    WorldObject::Weapon,
    WorldObject::Bomb,
    WorldObject::Carry,
    WorldObject::Chest,
    WorldObject::Horse,
    WorldObject::Sign,
    WorldObject::Leap,
    WorldObject::Flying,
    WorldObject::Extra,
];

/// What the two unpacked 6.00 images differed by.
///
/// Seihai's profile table has 6.00r2 and does not have 6.00r4, so [`R4`](Self::R4)
/// is not a selectable version. It records the static read: `resolveHost`
/// and an HTTP keep-alive header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unpacked600 {
    /// Product 6.002. No `resolveHost`. No HTTP keep-alive header.
    R2,
    /// Product 6.004. Adds `resolveHost` and `Connection: Keep-Alive`.
    R4,
}

impl Unpacked600 {
    pub fn resolve_host(self) -> bool {
        matches!(self, Self::R4)
    }

    pub fn http_keep_alive(self) -> bool {
        matches!(self, Self::R4)
    }
}

/// The unpacked 6.00 image a profile row corresponds to, when one was read.
pub fn unpacked(version: ClientVersion) -> Option<Unpacked600> {
    match version {
        ClientVersion::V600R2 => Some(Unpacked600::R2),
        _ => None,
    }
}

/// A selected v6 shell. Constructed only by [`Session::open`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Session {
    pub version: ClientVersion,
    pub draw: Draw,
    pub sound: Sound,
    pub shell: Gs2Shell,
    pub world: &'static [WorldObject],
}

impl Session {
    /// Open the v6 shell. Any other version, including v3–v5, is refused.
    /// v7 cannot be passed: it is not a [`ClientVersion`].
    pub fn open(version: ClientVersion) -> Result<Self, SelectError> {
        match select_version(version) {
            Ok(EngineProfile::V6) => Ok(Self {
                version,
                draw: Draw::v6(),
                sound: Sound::v6(),
                shell: Gs2Shell::v6(),
                world: WORLD,
            }),
            Ok(EngineProfile::V7(_)) => Err(SelectError::UnsupportedV7),
            Err(err) => Err(err),
        }
    }

    pub fn summary(self) -> String {
        format!(
            "seihai-issho: {} ({}) quattro_lib shell; OpenGL after a Direct3D 9 capability check; FMOD (device closed); GS2 shell; {}; TServer world objects. No playfield yet.",
            self.version.label(),
            self.version.token(),
            TSocket::NAME,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use seihai_core::protocol::ClientVersion;

    #[test]
    fn v6_opens_the_quattro_shell() {
        let session = Session::open(ClientVersion::V603R7).unwrap();
        assert_eq!(session.draw.presenter, Presenter::OpenGl);
        assert_eq!(session.draw.capability_check, CapabilityCheck::Direct3D9);
        assert_eq!(session.draw.playfields, PLAYFIELDS);
        assert!(!session.draw.gui_tiles_ctrl);
        assert!(!session.draw.horiz_sizing);
        assert_eq!(session.sound.backend, SoundBackend::Fmod);
        assert!(!session.sound.device_open);
        assert!(session.shell.scripts.contains(&"StartScript_GraalGui"));
        assert_eq!(TSocket::NAME, "TSocket");
        assert_ne!(TSocket::NAME, "GameConnection");
        assert!(session
            .world
            .iter()
            .any(|object| object.script_name() == "TServerPlayer"));
        assert!(session.summary().contains("quattro_lib"));
    }

    #[test]
    fn v3_to_v5_do_not_open_issho() {
        let torque = [
            ClientVersion::V304,
            ClientVersion::V3153,
            ClientVersion::V404R2,
            ClientVersion::V420R6,
            ClientVersion::V420R8,
            ClientVersion::V500R7,
            ClientVersion::V512,
            ClientVersion::V524R4,
            ClientVersion::V530R3,
            ClientVersion::V531R4,
        ];
        for version in torque {
            assert!(Session::open(version).is_err(), "{}", version.label());
        }
    }

    #[test]
    fn earlier_clients_do_not_open_issho() {
        assert!(Session::open(ClientVersion::V139).is_err());
        assert!(Session::open(ClientVersion::V222).is_err());
    }

    #[test]
    fn unpacked_600r4_is_recorded_and_not_a_profile() {
        assert_eq!(unpacked(ClientVersion::V600R2), Some(Unpacked600::R2));
        assert!(!Unpacked600::R2.resolve_host());
        assert!(!Unpacked600::R2.http_keep_alive());
        assert!(Unpacked600::R4.resolve_host());
        assert!(Unpacked600::R4.http_keep_alive());
        assert_eq!(unpacked(ClientVersion::V603R7), None);
        assert_eq!(unpacked(ClientVersion::V512), None);
    }
}
