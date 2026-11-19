//! The 1.x registration key: `reg.code` in the player's own install
//! folder, a 48-byte `.code` file (seihai-core `graaldes::code_read`).
//! 15th Graal to 1.1.9 show it to their relay peers as type `8` and leave
//! when a peer shows the same key (Graal_analysis `formats/graalsav.md`,
//! `protocol/relay-1998.md`).

use std::path::Path;

/// What the original says before it leaves the relay.
pub const DUPLICATE_KEY: &str =
    "Another player uses the same registration key like you. You will be disconnected.";

/// The key in `dir/reg.code`. `None` when the file is missing or does not
/// decode (the original reads that as `###`), or when it is empty: no key
/// is shown then **[inf]**.
pub fn registration_key(dir: &Path) -> Option<String> {
    let bytes = std::fs::read(dir.join("reg.code")).ok()?;
    let text = seihai_core::graaldes::code_read(&bytes).ok()?;
    let key = String::from_utf8_lossy(&text).into_owned();
    (!key.is_empty() && key.chars().all(|c| !c.is_control())).then_some(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_generated_reg_code_and_refuses_a_damaged_one() {
        let dir = std::env::temp_dir().join(format!("seihai-regcode-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(registration_key(&dir), None, "no file");
        let mut n = 0u8;
        let file = seihai_core::graaldes::code_write(b"SQRL-1999", || {
            n = n.wrapping_add(37);
            n
        });
        std::fs::write(dir.join("reg.code"), file).unwrap();
        assert_eq!(registration_key(&dir).as_deref(), Some("SQRL-1999"));
        let mut bad = file;
        bad[30] ^= 1;
        std::fs::write(dir.join("reg.code"), bad).unwrap();
        assert_eq!(registration_key(&dir), None, "halves differ");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
