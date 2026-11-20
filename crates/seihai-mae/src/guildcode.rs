//! The 1.x guild: `guildname.code` in the player's own install folder, a
//! 48-byte `.code` file (seihai-core `graaldes::guild_name`). From 5th
//! Graal on, the client reads it for the GS1 `playeringuild` check: an
//! exact, case-sensitive match on the name; no file, or a damaged one,
//! matches no name (lab, 1.1 and 1.1.9 offline, 2026-11-20; Graal_analysis
//! `formats/graalsav.md`, squirelworks/seihai-core#63).

use std::path::Path;

use seihai_core::protocol::offline_builds::OfflineBuild;

/// The builds before 5th Graal, which have no guild file.
const NO_GUILD_FILE: &[&str] = &[
    "1st Graal",
    "2nd Graal",
    "3rd Graal",
    "4th Graal",
    "4th Graal alpha",
];

/// Whether `build` reads `guildname.code`: the Delphi builds from 5th Graal
/// to 1.1.9.
pub fn reads_guild_file(build: &OfflineBuild) -> bool {
    build.family == seihai_core::protocol::offline_builds::Family::DelphiBeta
        && !NO_GUILD_FILE.contains(&build.label)
}

/// The guild name in `dir/guildname.code`, for `PlayerView::guild`. `None`
/// when the file is missing or does not read back.
pub fn guild_name(dir: &Path) -> Option<String> {
    let bytes = std::fs::read(dir.join("guildname.code")).ok()?;
    seihai_core::graaldes::guild_name(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use seihai_core::protocol::offline_builds::by_label;

    #[test]
    fn reads_a_generated_guild_file_and_refuses_a_damaged_one() {
        let dir = std::env::temp_dir().join(format!("seihai-guildcode-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(guild_name(&dir), None, "no file");
        let file = seihai_core::graaldes::code_write(b"Baddy", || 0x5a);
        std::fs::write(dir.join("guildname.code"), file).unwrap();
        assert_eq!(guild_name(&dir).as_deref(), Some("Baddy"));
        let mut bad = file;
        bad[30] ^= 1;
        std::fs::write(dir.join("guildname.code"), bad).unwrap();
        assert_eq!(guild_name(&dir), None, "halves differ");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_guild_file_starts_with_5th_graal() {
        for label in NO_GUILD_FILE {
            let b = by_label(label).unwrap_or_else(|| panic!("no build {label}"));
            assert!(!reads_guild_file(b), "{label}");
        }
        for label in ["5th Graal", "15th Graal (1.1)", "1.1", "1.1.9 Gold Edition"] {
            let b = by_label(label).unwrap_or_else(|| panic!("no build {label}"));
            assert!(reads_guild_file(b), "{label}");
        }
        assert!(!reads_guild_file(by_label("Java Graal").unwrap()));
    }
}
