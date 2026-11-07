//! Read Torque engine version from the checked-out Torque3D MIT submodule (no Rust handshake).

use std::env;
use std::fs;
use std::path::PathBuf;

fn parse_game_engine_version(h: &str) -> Option<String> {
    for line in h.lines() {
        let line = line.trim();
        if !line.starts_with("#define TORQUE_GAME_ENGINE_VERSION_STRING") {
            continue;
        }
        let rest = line
            .strip_prefix("#define TORQUE_GAME_ENGINE_VERSION_STRING")?
            .trim();
        let q = rest.strip_prefix('"')?.strip_suffix('"')?;
        return Some(q.to_owned());
    }
    None
}

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let header = manifest.join("../../vendor/Torque3D/Engine/source/app/version.h");
    if !header.is_file() {
        panic!(
            "Torque3D submodule missing (expected {}). Clone with: git submodule update --init --recursive",
            header.display()
        );
    }
    let text =
        fs::read_to_string(&header).unwrap_or_else(|e| panic!("read {}: {e}", header.display()));
    let version = parse_game_engine_version(&text).unwrap_or_else(|| {
        panic!(
            "could not parse TORQUE_GAME_ENGINE_VERSION_STRING from {}",
            header.display()
        )
    });

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let out = out_dir.join("torque_engine_version.rs");
    fs::write(
        out,
        format!(
            r#"// Generated from `vendor/Torque3D/Engine/source/app/version.h` (Torque3D MIT).

/// Value of `TORQUE_GAME_ENGINE_VERSION_STRING` in the pinned Torque3D tree.
pub const TORQUE_GAME_ENGINE_VERSION: &str = "{version}";
"#
        ),
    )
    .expect("write torque_engine_version.rs");

    println!("cargo:rerun-if-changed={}", header.display());
}
