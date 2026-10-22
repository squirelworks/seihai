//! Offline asset root.
//!
//! Expected layout for a local asset tree used by the offline client:
//! ```text
//! <root>/
//!   pics1.png          # 2048×512 tile sheet
//!   tiledefs.txt       # or arrays.dat
//!   levels/            # .nw files
//!   gani/              # .gani files
//! ```

use std::path::{Path, PathBuf};

/// Paths into a local asset tree.
#[derive(Debug, Clone)]
pub struct AssetRoot {
    pub root: PathBuf,
}

impl AssetRoot {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn pics1(&self) -> PathBuf {
        self.root.join("pics1.png")
    }

    pub fn tiledefs(&self) -> PathBuf {
        self.root.join("tiledefs.txt")
    }

    pub fn arrays_dat(&self) -> PathBuf {
        self.root.join("arrays.dat")
    }

    pub fn levels_dir(&self) -> PathBuf {
        self.root.join("levels")
    }

    pub fn level(&self, name: &str) -> PathBuf {
        self.levels_dir().join(name)
    }

    pub fn gani_dir(&self) -> PathBuf {
        self.root.join("gani")
    }

    pub fn gani(&self, name: &str) -> PathBuf {
        self.gani_dir().join(name)
    }

    /// True if the root contains at least a `pics1.png`.
    pub fn is_valid(&self) -> bool {
        self.pics1().is_file()
    }
}

impl AsRef<Path> for AssetRoot {
    fn as_ref(&self) -> &Path {
        &self.root
    }
}
