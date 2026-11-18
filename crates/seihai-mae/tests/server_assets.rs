//! Contract tests for the server art pack (`assets/server/`).
//!
//! Every file a server hands to a real client is loaded with Seihai's own
//! readers: each PNG and its size, each GANI (every placement names a sprite
//! it defines, every cue names a sound in the pack), each WAV, the update
//! package and the fonts. Regenerate with `python3 tools/gen_server_assets.py`.

use std::fs;
use std::path::{Path, PathBuf};

use seihai::assets::{read_png_rgba, Gani, GaniPlacementReference};
use seihai::sound::decode_wav;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
        .join("server")
}

fn read(rel: &str) -> Vec<u8> {
    fs::read(root().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn files(dir: &str, ext: &str) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(root().join(dir))
        .unwrap_or_else(|e| panic!("{dir}: {e}"))
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|n| n.ends_with(ext))
        .collect();
    v.sort();
    v
}

fn size(rel: &str) -> (u32, u32) {
    let (w, h, _) = read_png_rgba(&read(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
    (w, h)
}

#[test]
fn images_have_the_client_sizes() {
    let want: &[(&str, (u32, u32))] = &[
        ("images/pics1.png", (2048, 512)),
        ("images/sprites.png", (128, 402)),
        ("images/state.png", (224, 145)),
        ("images/chest.png", (32, 32)),
        ("images/wbowi1.png", (32, 32)),
        ("images/wbomb1.png", (32, 32)),
    ];
    for (rel, wh) in want {
        assert_eq!(size(rel), *wh, "{rel}");
    }
    for (dir, wh, at_least) in [
        ("bodies", (128, 720), 3),
        ("heads", (32, 560), 4),
        ("swords", (128, 96), 4),
        ("shields", (38, 20), 3),
    ] {
        let pngs = files(dir, ".png");
        assert!(pngs.len() >= at_least, "{dir}: {pngs:?}");
        for f in pngs {
            let rel = format!("{dir}/{f}");
            assert_eq!(size(&rel), wh, "{rel}");
            // builds up to 2.05 ask for the .gif name
            assert!(
                root().join(dir).join(f.replace(".png", ".gif")).exists(),
                "{rel} has no .gif copy"
            );
        }
    }
    for n in ["sword1", "sword2", "sword3", "sword4"] {
        assert!(root().join(format!("swords/{n}.png")).exists());
    }
}

#[test]
fn ganis_parse_and_their_cues_are_in_the_pack() {
    let sounds = files("sounds", ".wav");
    let ganis = files("ganis", ".gani");
    for name in [
        "idle",
        "walk",
        "swim",
        "sword",
        "sit",
        "hurt",
        "dead",
        "carry",
        "carrystill",
        "push",
        "pull",
        "grab",
        "lift",
        "spin",
        "hidden",
        "hiddenstill",
    ] {
        assert!(
            ganis.contains(&format!("{name}.gani")),
            "{name}.gani missing"
        );
    }
    for f in ganis {
        let g = Gani::load(root().join("ganis").join(&f)).unwrap_or_else(|e| panic!("{f}: {e}"));
        assert!(g.frame_count() >= 1, "{f}");
        for (i, fr) in g.frames.iter().enumerate() {
            assert_eq!(fr.rows.len(), 4, "{f} step {i}");
            for row in &fr.rows {
                for p in row {
                    let GaniPlacementReference::Sprite(id) = p.reference else {
                        panic!("{f}: non-numeric placement");
                    };
                    assert!(
                        g.sprites.iter().any(|s| s.id == id),
                        "{f} step {i}: sprite {id} undefined"
                    );
                }
            }
            for s in &fr.sounds {
                assert!(
                    sounds.contains(&s.file),
                    "{f}: cue {} not in sounds/",
                    s.file
                );
            }
        }
        if let Some(back) = &g.setback_to {
            assert!(
                root().join("ganis").join(format!("{back}.gani")).exists(),
                "{f}: SETBACKTO {back}"
            );
        }
    }
}

#[test]
fn sounds_decode() {
    let wavs = files("sounds", ".wav");
    for n in [
        "steps.wav",
        "steps2.wav",
        "sword.wav",
        "water.wav",
        "crush.wav",
        "lift.wav",
        "item.wav",
        "bomb.wav",
        "chest.wav",
        "swordon.wav",
    ] {
        assert!(wavs.iter().any(|w| w == n), "{n} missing");
    }
    for w in wavs {
        let pcm = decode_wav(&read(&format!("sounds/{w}"))).unwrap_or_else(|e| panic!("{w}: {e}"));
        assert_eq!(pcm.channels, 1, "{w}");
        assert!(!pcm.samples.is_empty(), "{w}");
    }
}

#[test]
fn base_package_and_fonts() {
    let gupd = String::from_utf8(read("packages/basepackage.gupd")).unwrap();
    assert!(gupd.starts_with("GRPKG001\n"));
    for key in [
        "NAME basepackage",
        "VERSION ",
        "PLATFORM any",
        "MODE required",
    ] {
        assert!(gupd.lines().any(|l| l.starts_with(key)), "{key}");
    }
    for f in [
        "arial.ttf",
        "arialbd.ttf",
        "helvetica.ttf",
        "helveticabd.ttf",
        "tempsitc.ttf",
        "tempsitcbd.ttf",
    ] {
        let b = read(&format!("fonts/{f}"));
        assert_eq!(&b[..4], &[0, 1, 0, 0], "{f}: not a TrueType file");
    }
    assert!(root().join("fonts/LICENSE-OFL.txt").exists());
}
