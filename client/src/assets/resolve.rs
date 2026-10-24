//! Version-aware asset lookup in a client data folder.
//!
//! The 2.22 data folder ("Graal Reborn 2.22.0") is the reference layout.
//! Every client version looks for the same kinds of files in it; the
//! differences are in which names and extensions a build can use, and those
//! come from the version's profile row, not from branches here.
//!
//! What the resolver does that a plain `root.join(name)` does not:
//!
//! * **Several folders per kind.** Levels sit at the root (`new1.nw`), in
//!   `maps/` (`.gmap`) or in `levels/` (where downloads are saved); sounds in
//!   `sounds/`; GANIs in `levels/ganis/` (and `levels/bomys/`); player art in `levels/bodies/` and
//!   friends; everything else image-like in the other `levels/<kind>/`
//!   folders.
//! * **Case-insensitive names.** Windows clients ignore case. A Linux file
//!   system does not, and the folder has names such as `TORERO-2.MID`.
//! * **Extension rules from the profile.** `files.image_ext` is added to a
//!   bare image name (GServer adds `.gif` for clients below 2.1); level
//!   extensions are limited to what the build opens (`levels.nw`,
//!   `levels.graal`, `levels.zelda`, `levels.gmap`); GANIs only where `ui.ganis` is set.
//! * **Safe names.** Names come from the server. `..`, absolute paths and
//!   drive letters never leave the folder.
//!
//! Lookups never cache a miss: a file that arrives from the server a moment
//! later is found on the next call.
//!
//! Not settled (see `documentation/assets.md`): the order in which a real
//! client prefers a downloaded copy over a shipped one, and the real
//! client's own list of folders per kind. The orders below are Seihai's.

use std::fs;
use std::path::{Path, PathBuf};

use super::AssetRoot;
use crate::protocol::VersionProfile;

/// Player and item art folders under `levels/` in the 2.22 data folder.
/// The same names are also tried directly under the root (older layouts).
pub const IMAGE_CATEGORIES: &[&str] = &[
    "images", "ganis", "bodies", "heads", "hats", "shields", "swords", "weapons", "bomys", "baddies", "emoticons",
    "horses", "backpals", "smilies", "letters", "snapshot", "webgifs", "tutorial",
];

/// Where a download is saved, and where a downloaded file is looked for
/// first. This is `levels/`, as `main.rs` writes files there.
const DOWNLOADS: &str = "levels";

/// Finds files in one data folder for one version.
#[derive(Debug, Clone)]
pub struct AssetResolver {
    root: AssetRoot,
    profile: &'static VersionProfile,
}

impl AssetResolver {
    pub fn new(root: AssetRoot, profile: &'static VersionProfile) -> Self {
        Self { root, profile }
    }

    pub fn root(&self) -> &AssetRoot {
        &self.root
    }

    pub fn profile(&self) -> &'static VersionProfile {
        self.profile
    }

    // ---- levels ----------------------------------------------------------

    /// Find a level, gmap, `.graal` or `.zelda` file. A name with an extension must be
    /// a format this version opens; a bare name gets the version's level
    /// extensions in turn.
    pub fn level(&self, name: &str) -> Option<PathBuf> {
        let rel = clean_name(name)?;
        let candidates: Vec<String> = match level_ext(&rel) {
            Some(ext) => {
                if !self.opens(ext) {
                    return None;
                }
                vec![rel]
            }
            None => self.profile.level_extensions().iter().map(|e| format!("{rel}{e}")).collect(),
        };
        let dirs = [
            self.root.root.join(DOWNLOADS),
            self.root.root.clone(),
            self.root.root.join("maps"),
        ];
        first_in(&dirs, &candidates)
    }

    /// Does this version open files with this level extension (".nw")?
    pub fn opens(&self, ext: &str) -> bool {
        let l = &self.profile.levels;
        match ext.to_ascii_lowercase().as_str() {
            ".nw" => l.nw,
            ".graal" => l.graal,
            ".zelda" => l.zelda,
            ".gmap" => l.gmap,
            _ => false,
        }
    }

    /// The path of a level for loading or saving: the file if it exists,
    /// else where a download of that name would be saved. A name that
    /// would leave the folder falls back to the downloads folder joined
    /// with its last component.
    pub fn level_path(&self, name: &str) -> PathBuf {
        self.level(name).unwrap_or_else(|| {
            let last = name.rsplit(['/', '\\']).next().unwrap_or(name);
            self.root.level(last)
        })
    }

    // ---- GANIs -----------------------------------------------------------

    /// Find a GANI. Builds without GANIs (`ui.ganis` false) find none. The
    /// `.gani` extension is added when the name has none.
    pub fn gani(&self, name: &str) -> Option<PathBuf> {
        if !self.profile.ui.ganis {
            return None;
        }
        let rel = clean_name(name)?;
        let file = if has_ext(&rel) { rel } else { format!("{rel}.gani") };
        let dirs = [
            self.root.gani_dir(),
            self.root.root.join(DOWNLOADS).join("ganis"),
            // The bomy (pet) animations sit in their own folder.
            self.root.root.join(DOWNLOADS).join("bomys"),
            self.root.root.join(DOWNLOADS),
            self.root.root.clone(),
        ];
        first_in(&dirs, &[file])
    }

    // ---- images ----------------------------------------------------------

    /// Find an image (tile sheet, NPC image, HUD art, player art, ...).
    pub fn image(&self, name: &str) -> Option<PathBuf> {
        self.image_in("", name)
    }

    /// As [`image`](Self::image), preferring the folder `category`
    /// (`"bodies"`, `"heads"`, ...). An empty category has no preference.
    ///
    /// Name rules: a bare name gets the profile's `image_ext` first, then
    /// `.png`, `.gif`, `.mng`. A `.gif` or `.png` name that is not found
    /// is tried with the other extension: the 2.22 folder ships PNG only,
    /// and a pre-2.1 server names images `.gif`. (Seihai's reading; the
    /// real clients' fallback is not captured.)
    pub fn image_in(&self, category: &str, name: &str) -> Option<PathBuf> {
        let rel = clean_name(name)?;
        let candidates = self.image_names(&rel);
        let r = &self.root.root;
        let mut dirs: Vec<PathBuf> = Vec::new();
        if !category.is_empty() {
            let cat = clean_name(category)?;
            dirs.push(r.join(&cat));
            dirs.push(r.join(DOWNLOADS).join(&cat));
        }
        dirs.push(r.join(DOWNLOADS));
        dirs.push(r.clone());
        for c in IMAGE_CATEGORIES {
            dirs.push(r.join(DOWNLOADS).join(c));
        }
        // Map pictures named by a `.gmap` (`MAPIMG`).
        dirs.push(r.join("maps"));
        for c in IMAGE_CATEGORIES {
            dirs.push(r.join(c));
        }
        first_in(&dirs, &candidates)
    }

    fn image_names(&self, rel: &str) -> Vec<String> {
        let mut v = Vec::new();
        match ext_of(rel) {
            None => {
                v.push(format!("{rel}{}", self.profile.files.image_ext));
                for e in [".png", ".gif", ".mng"] {
                    let s = format!("{rel}{e}");
                    if !v.contains(&s) {
                        v.push(s);
                    }
                }
            }
            Some(e) => {
                v.push(rel.to_string());
                let stem = &rel[..rel.len() - e.len()];
                match e.to_ascii_lowercase().as_str() {
                    ".gif" => v.push(format!("{stem}.png")),
                    ".png" => v.push(format!("{stem}.gif")),
                    _ => {}
                }
            }
        }
        v
    }

    // ---- sounds and music ------------------------------------------------

    /// The folder holding sound effects (`sounds/`, else `levels/sounds/`).
    pub fn sounds_dir(&self) -> Option<PathBuf> {
        self.root.sounds_dir()
    }

    /// Find a sound effect. `.wav` is added to a bare name.
    pub fn sound(&self, name: &str) -> Option<PathBuf> {
        let rel = clean_name(name.trim_start_matches('+'))?;
        let file = if has_ext(&rel) { rel } else { format!("{rel}.wav") };
        let r = &self.root.root;
        let dirs = [
            r.join("sounds"),
            r.join(DOWNLOADS).join("sounds"),
            r.join(DOWNLOADS),
            r.clone(),
        ];
        first_in(&dirs, &[file])
    }

    /// Find a music file. `.mid` is added to a bare name (the common case).
    pub fn music(&self, name: &str) -> Option<PathBuf> {
        let rel = clean_name(name)?;
        let file = if has_ext(&rel) { rel } else { format!("{rel}.mid") };
        let r = &self.root.root;
        let dirs = [
            r.join(DOWNLOADS).join("midis"),
            r.join("midis"),
            r.join(DOWNLOADS),
            r.join("sounds"),
            r.clone(),
        ];
        first_in(&dirs, &[file])
    }

    // ---- tile tables -----------------------------------------------------

    /// The tile type table: `tiledefs.txt` (text) or `arrays.dat`.
    pub fn tile_types(&self) -> Option<(PathBuf, TileTypesFile)> {
        let r = &self.root.root;
        if let Some(p) = first_in(&[r.clone(), r.join(DOWNLOADS)], &["tiledefs.txt".to_string()]) {
            return Some((p, TileTypesFile::Text));
        }
        first_in(&[r.clone(), r.join(DOWNLOADS)], &["arrays.dat".to_string()]).map(|p| (p, TileTypesFile::ArraysDat))
    }

    /// Find one tiledefs file by name in `levels/tiledefs/` (or `tiledefs/`).
    pub fn tiledef(&self, name: &str) -> Option<PathBuf> {
        let rel = clean_name(name)?;
        let r = &self.root.root;
        first_in(&[r.join(DOWNLOADS).join("tiledefs"), r.join("tiledefs")], &[rel])
    }

    /// The `tiledefs*.txt` files in `levels/tiledefs/` (the offline and
    /// online tile-set lists), sorted by name.
    pub fn tiledef_files(&self) -> Vec<PathBuf> {
        let dir = self.root.root.join(DOWNLOADS).join("tiledefs");
        let mut v: Vec<PathBuf> = fs::read_dir(&dir)
            .map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.is_file()).collect())
            .unwrap_or_default();
        v.sort();
        v
    }
}

/// Which loader reads the tile type table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileTypesFile {
    Text,
    ArraysDat,
}

// ---- name helpers --------------------------------------------------------

/// Turn a server-supplied name into a safe relative path with `/`
/// separators, or `None` when it would escape the folder or is empty.
pub(crate) fn clean_name(name: &str) -> Option<String> {
    let name = name.trim().trim_matches('"');
    if name.is_empty() || name.contains('\0') {
        return None;
    }
    let norm = name.replace('\\', "/");
    if norm.starts_with('/') {
        return None;
    }
    let mut parts = Vec::new();
    for p in norm.split('/') {
        match p {
            "" | "." => {}
            ".." => return None,
            // A drive letter ("c:") or any other colon.
            p if p.contains(':') => return None,
            p => parts.push(p),
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("/"))
    }
}

/// `".nw"`, `".png"`: the extension of the last component, with the dot.
fn ext_of(rel: &str) -> Option<&str> {
    let last = rel.rsplit('/').next().unwrap_or(rel);
    last.rfind('.').filter(|&i| i > 0 && i + 1 < last.len()).map(|i| &last[i..])
}

fn has_ext(rel: &str) -> bool {
    ext_of(rel).is_some()
}

/// A level extension if the name has one of the four Seihai knows.
fn level_ext(rel: &str) -> Option<&str> {
    ext_of(rel).filter(|e| matches!(e.to_ascii_lowercase().as_str(), ".nw" | ".graal" | ".zelda" | ".gmap"))
}

/// The first existing file, trying each directory in order and, inside
/// it, each candidate name in order.
fn first_in(dirs: &[PathBuf], names: &[String]) -> Option<PathBuf> {
    for d in dirs {
        for n in names {
            if let Some(p) = lookup(d, n) {
                return Some(p);
            }
        }
    }
    None
}

/// Find `rel` under `dir`, matching every component ignoring ASCII case.
pub(crate) fn lookup(dir: &Path, rel: &str) -> Option<PathBuf> {
    // Exact case first: one stat, no directory read.
    let direct = dir.join(rel);
    if direct.is_file() {
        return Some(direct);
    }
    let mut cur = dir.to_path_buf();
    let mut comps = rel.split('/').peekable();
    while let Some(c) = comps.next() {
        let last = comps.peek().is_none();
        let found = fs::read_dir(&cur).ok()?.flatten().find(|e| {
            e.file_name().to_str().is_some_and(|n| n.eq_ignore_ascii_case(c))
                && e.path().is_dir() != last
        })?;
        cur = found.path();
    }
    cur.is_file().then_some(cur)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::ClientVersion;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("seihai-resolve-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn touch(root: &Path, rel: &str) {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, b"x").unwrap();
    }

    fn resolver(root: &Path, v: ClientVersion) -> AssetResolver {
        AssetResolver::new(AssetRoot::new(root), v.profile())
    }

    #[test]
    fn names_stay_inside_the_folder() {
        assert_eq!(clean_name("a/b.png").as_deref(), Some("a/b.png"));
        assert_eq!(clean_name("a\\b.png").as_deref(), Some("a/b.png"));
        assert_eq!(clean_name("./a.png").as_deref(), Some("a.png"));
        assert_eq!(clean_name("\"quoted.png\"").as_deref(), Some("quoted.png"));
        for bad in ["", "..", "../x", "a/../../x", "/etc/passwd", "c:/x", "c:\\x", "a\0b", "///"] {
            assert_eq!(clean_name(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn levels_come_from_root_levels_or_maps_and_ignore_case() {
        let d = tmp("levels");
        touch(&d, "new1.nw");
        touch(&d, "maps/Offline.gmap");
        touch(&d, "levels/saved.nw");
        let r = resolver(&d, ClientVersion::V222);
        assert_eq!(r.level("new1.nw"), Some(d.join("new1.nw")));
        assert_eq!(r.level("NEW1.NW"), Some(d.join("new1.nw")));
        assert_eq!(r.level("offline.gmap"), Some(d.join("maps/Offline.gmap")));
        assert_eq!(r.level("saved.nw"), Some(d.join("levels/saved.nw")));
        // A bare name takes the version's extensions in turn.
        assert_eq!(r.level("new1"), Some(d.join("new1.nw")));
        assert_eq!(r.level("missing.nw"), None);
        // The downloads folder wins over the root.
        touch(&d, "levels/new1.nw");
        assert_eq!(r.level("new1.nw"), Some(d.join("levels/new1.nw")));
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn level_formats_follow_the_profile() {
        let d = tmp("levelgate");
        touch(&d, "a.nw");
        touch(&d, "b.graal");
        touch(&d, "c.gmap");
        // 1.39: `.graal` only.
        let old = resolver(&d, ClientVersion::V139);
        assert_eq!(old.level("a.nw"), None);
        assert!(old.level("b.graal").is_some());
        assert_eq!(old.level("c.gmap"), None);
        assert_eq!(old.level("b"), Some(d.join("b.graal")));
        // 2.05 and later: all three.
        for v in [ClientVersion::V205, ClientVersion::V217, ClientVersion::V222, ClientVersion::V231] {
            let r = resolver(&d, v);
            assert!(r.level("a.nw").is_some() && r.level("b.graal").is_some() && r.level("c.gmap").is_some(), "{v:?}");
        }
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn zelda_levels_open_in_every_build_and_bare_names_try_graal_first() {
        let d = tmp("zelda");
        touch(&d, "levels/Level11.Zelda");
        touch(&d, "house1.graal");
        touch(&d, "house1.zelda");
        touch(&d, "only.zelda");
        for p in crate::protocol::PROFILES.iter() {
            let r = AssetResolver::new(AssetRoot::new(&d), p);
            // An explicit name is found whatever case the disk has.
            assert_eq!(r.level("level11.zelda"), Some(d.join("levels/Level11.Zelda")), "{}", p.label);
            assert!(r.opens(".zelda") && r.opens(".ZELDA"), "{}", p.label);
            // A bare name: the build's own order, `.zelda` after `.graal`.
            assert_eq!(r.level("only"), Some(d.join("only.zelda")), "{}", p.label);
            assert_eq!(r.level("house1"), Some(d.join("house1.graal")), "{}", p.label);
            // No swap for a named extension that is missing.
            assert_eq!(r.level("house2.zelda"), None);
        }
        // A build whose profile does not read Z3-V1 would not open it.
        let mut p = ClientVersion::V139.profile().clone();
        p.levels.zelda = false;
        let p: &'static VersionProfile = Box::leak(Box::new(p));
        let r = AssetResolver::new(AssetRoot::new(&d), p);
        assert_eq!(r.level("only.zelda"), None);
        assert_eq!(r.level("only"), None);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn ganis_only_where_the_build_has_them() {
        let d = tmp("gani");
        touch(&d, "levels/ganis/Walk.gani");
        touch(&d, "gani/idle.gani");
        let old = resolver(&d, ClientVersion::V139);
        assert_eq!(old.gani("walk.gani"), None);
        let new = resolver(&d, ClientVersion::V222);
        assert_eq!(new.gani("walk.gani"), Some(d.join("levels/ganis/Walk.gani")));
        assert_eq!(new.gani("walk"), Some(d.join("levels/ganis/Walk.gani")));
        assert_eq!(new.gani("idle.gani"), Some(d.join("gani/idle.gani")));
        assert_eq!(new.gani("../walk.gani"), None);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn images_search_the_category_folders_and_the_extension_rules() {
        let d = tmp("image");
        touch(&d, "pics1.png");
        touch(&d, "levels/bodies/body.png");
        touch(&d, "levels/heads/head0.png");
        touch(&d, "levels/images/door.png");
        touch(&d, "levels/webgifs/Alienpics.png");
        touch(&d, "swords/old.png");
        touch(&d, "levels/ganis/gralats.png");
        touch(&d, "levels/bomys/bomy_idle.gani");
        touch(&d, "maps/map.png");
        let r = resolver(&d, ClientVersion::V222);
        assert_eq!(r.image("pics1.png"), Some(d.join("pics1.png")));
        assert_eq!(r.image_in("bodies", "body.png"), Some(d.join("levels/bodies/body.png")));
        // No category hint: still found by name.
        assert_eq!(r.image("head0.png"), Some(d.join("levels/heads/head0.png")));
        assert_eq!(r.image("door.png"), Some(d.join("levels/images/door.png")));
        assert_eq!(r.image("alienpics.png"), Some(d.join("levels/webgifs/Alienpics.png")));
        // GANI sprite sheets sit beside the GANIs; map pictures in maps/.
        assert_eq!(r.image("gralats.png"), Some(d.join("levels/ganis/gralats.png")));
        assert_eq!(r.image("map.png"), Some(d.join("maps/map.png")));
        assert_eq!(r.gani("bomy_idle.gani"), Some(d.join("levels/bomys/bomy_idle.gani")));
        // Old layout: the category folder straight under the root.
        assert_eq!(r.image("old.png"), Some(d.join("swords/old.png")));
        // A bare name gets the profile's extension, then the others.
        assert_eq!(r.image("door"), Some(d.join("levels/images/door.png")));
        // `.gif` asked, `.png` shipped.
        assert_eq!(r.image("door.gif"), Some(d.join("levels/images/door.png")));
        assert_eq!(r.image("nothere.png"), None);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn bare_image_names_prefer_the_profile_extension() {
        let d = tmp("imgext");
        touch(&d, "pic.gif");
        touch(&d, "pic.png");
        let old = resolver(&d, ClientVersion::V205);
        assert_eq!(old.profile().files.image_ext, ".gif");
        assert_eq!(old.image("pic"), Some(d.join("pic.gif")));
        let new = resolver(&d, ClientVersion::V222);
        assert_eq!(new.profile().files.image_ext, ".png");
        assert_eq!(new.image("pic"), Some(d.join("pic.png")));
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn sounds_and_music() {
        let d = tmp("sound");
        touch(&d, "sounds/Arrow.wav");
        touch(&d, "levels/midis/INTRO.MID");
        let r = resolver(&d, ClientVersion::V222);
        assert_eq!(r.sound("arrow.wav"), Some(d.join("sounds/Arrow.wav")));
        assert_eq!(r.sound("arrow"), Some(d.join("sounds/Arrow.wav")));
        // The loop prefix of a play request is not part of the name.
        assert_eq!(r.sound("+arrow.wav"), Some(d.join("sounds/Arrow.wav")));
        assert_eq!(r.sound("nothere.wav"), None);
        assert_eq!(r.music("intro.mid"), Some(d.join("levels/midis/INTRO.MID")));
        assert_eq!(r.music("intro"), Some(d.join("levels/midis/INTRO.MID")));
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn tile_tables_and_tiledefs() {
        let d = tmp("tiles");
        touch(&d, "arrays.dat");
        touch(&d, "levels/tiledefs/b.txt");
        touch(&d, "levels/tiledefs/a.txt");
        let r = resolver(&d, ClientVersion::V222);
        assert_eq!(r.tile_types(), Some((d.join("arrays.dat"), TileTypesFile::ArraysDat)));
        touch(&d, "tiledefs.txt");
        assert_eq!(r.tile_types(), Some((d.join("tiledefs.txt"), TileTypesFile::Text)));
        let files = r.tiledef_files();
        assert_eq!(files, vec![d.join("levels/tiledefs/a.txt"), d.join("levels/tiledefs/b.txt")]);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn a_file_that_arrives_later_is_found() {
        let d = tmp("late");
        let r = resolver(&d, ClientVersion::V222);
        assert_eq!(r.image("late.png"), None);
        touch(&d, "levels/late.png");
        assert_eq!(r.image("late.png"), Some(d.join("levels/late.png")));
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn level_path_falls_back_to_the_downloads_folder() {
        let d = tmp("lp");
        let r = resolver(&d, ClientVersion::V222);
        assert_eq!(r.level_path("x.nw"), d.join("levels/x.nw"));
        // A name that tries to leave the folder keeps only its last part.
        assert_eq!(r.level_path("../../etc/x.nw"), d.join("levels/x.nw"));
        let _ = fs::remove_dir_all(&d);
    }
}
