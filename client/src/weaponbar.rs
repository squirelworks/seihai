//! The player's weapon list and the selected weapon, the one the status
//! display draws beside the D button.
//!
//! Rules from the 2.05 rev 2 client (`documentation/weapon-images.md`):
//!
//! * Server weapons (`PLO_NPCWEAPONADD`) and built-in weapons
//!   (`PLO_DEFAULTWEAPON`) go into one list, in the order they arrive. A
//!   built-in weapon is skipped when the list already has its name.
//! * Every add selects the new last weapon. Selecting index `i` takes the
//!   first weapon from `i` on, wrapping, whose name is not empty and does
//!   not start with `-`; when there is none, `i` itself stays selected.
//! * After a delete the selection stays where it was if that index still
//!   exists, otherwise it moves to the new last weapon.

/// A built-in weapon: its name in the player's list and its image (without
/// the extension, which is the build's image extension).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefaultWeapon {
    pub name: &'static str,
    pub image: &'static str,
}

/// The built-in weapon for a `PLO_DEFAULTWEAPON` id. The client keeps one
/// table of 25 item names; the ids with a weapon script are these seven. The
/// weapon's name is the item name with its first letter in upper case.
/// (2.05 rev 2 binary; the 2.22 client asked for the same images as `.png`
/// in the lab.)
pub fn default_weapon(id: u8) -> Option<DefaultWeapon> {
    let (name, image) = match id {
        7 => ("Bow", "wbowi1"),
        8 => ("Bomb", "wbomb1"),
        12 => ("Superbomb", "wbomb2"),
        20 => ("Fireball", "wbowi2"),
        21 => ("Fireblast", "wbowi3"),
        22 => ("Nukeshot", "wbowi4"),
        23 => ("Joltbomb", "wbomb3"),
        _ => return None,
    };
    Some(DefaultWeapon { name, image })
}

/// One weapon in the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    /// The image file name, if the weapon has one.
    pub image: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WeaponBar {
    pub entries: Vec<Entry>,
    pub selected: Option<usize>,
}

impl WeaponBar {
    /// Bring the list up to date with what the server has sent: drop the
    /// weapons that are gone, then add the new ones (built-in ones first, in
    /// their order, then server weapons). `image_ext` is the build's image
    /// extension (`.gif` or `.png`), for built-in weapons.
    /// `weapons` are the server weapons as (name, image) pairs, in order.
    pub fn sync<'a>(&mut self, defaults: &[u8], weapons: impl IntoIterator<Item = (&'a str, Option<&'a str>)>, image_ext: &str) {
        let wanted: Vec<Entry> = defaults
            .iter()
            .filter_map(|&id| default_weapon(id))
            .map(|d| Entry { name: d.name.to_string(), image: Some(format!("{}{image_ext}", d.image)) })
            .chain(weapons.into_iter().map(|(name, image)| Entry { name: name.to_string(), image: weapon_image(image) }))
            .collect();
        let before = self.entries.len();
        self.entries.retain(|e| wanted.iter().any(|w| w.name == e.name));
        if self.entries.len() != before {
            let n = self.entries.len();
            match self.selected {
                Some(i) if i < n => self.select(i),
                _ if n > 0 => self.select(n - 1),
                _ => self.selected = None,
            }
        }
        for w in wanted {
            match self.entries.iter_mut().find(|e| e.name == w.name) {
                // A server weapon sent again replaces its image.
                Some(e) => e.image = w.image,
                None => {
                    self.entries.push(w);
                    self.select(self.entries.len() - 1);
                }
            }
        }
    }

    /// Select from `start` on, skipping hidden weapons.
    pub fn select(&mut self, start: usize) {
        let n = self.entries.len();
        if n == 0 {
            self.selected = None;
            return;
        }
        let pick = (0..n)
            .map(|k| (start + k) % n)
            .find(|&i| !self.entries[i].name.is_empty() && !self.entries[i].name.starts_with('-'));
        self.selected = Some(pick.unwrap_or(start));
    }

    pub fn current(&self) -> Option<&Entry> {
        self.selected.and_then(|i| self.entries.get(i))
    }

    /// The image of the selected weapon.
    pub fn current_image(&self) -> Option<&str> {
        self.current().and_then(|e| e.image.as_deref())
    }
}

/// A server weapon's image: a name with no extension gets `.gif` (see
/// [`crate::protocol::weapon_images`]).
fn weapon_image(image: Option<&str>) -> Option<String> {
    let s = image.filter(|s| !s.is_empty())?;
    Some(if s.contains('.') { s.to_string() } else { format!("{s}.gif") })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w<'a>(name: &'a str, image: Option<&'a str>) -> (&'a str, Option<&'a str>) {
        (name, image)
    }

    #[test]
    fn the_latest_weapon_is_selected() {
        let mut bar = WeaponBar::default();
        bar.sync(&[8], [], ".png");
        assert_eq!(bar.current_image(), Some("wbomb1.png"));
        bar.sync(&[8, 7], [], ".png");
        assert_eq!(bar.current().unwrap().name, "Bow");
        // A weapon that arrives after login is selected too.
        bar.sync(&[8, 7], [w("Lantern", Some("lantern"))], ".png");
        assert_eq!(bar.current_image(), Some("lantern.gif"));
    }

    #[test]
    fn hidden_weapons_are_skipped_wrapping_round() {
        let mut bar = WeaponBar::default();
        bar.sync(&[8], [w("-System", None)], ".gif");
        assert_eq!(bar.current().unwrap().name, "Bomb");
        let mut only_hidden = WeaponBar::default();
        only_hidden.sync(&[], [w("-System", None)], ".gif");
        assert_eq!(only_hidden.selected, Some(0));
    }

    #[test]
    fn a_delete_keeps_the_index_or_moves_to_the_last() {
        let mut bar = WeaponBar::default();
        bar.sync(&[8, 7, 12], [], ".png");
        bar.select(0);
        bar.sync(&[7, 12], [], ".png");
        assert_eq!(bar.current().unwrap().name, "Bow");
        bar.select(1);
        bar.sync(&[7], [], ".png");
        assert_eq!(bar.current().unwrap().name, "Bow");
        bar.sync(&[], [], ".png");
        assert_eq!(bar.selected, None);
    }

    #[test]
    fn a_built_in_weapon_is_not_added_twice() {
        let mut bar = WeaponBar::default();
        bar.sync(&[8], [w("Bomb", Some("mybomb.png"))], ".png");
        assert_eq!(bar.entries.len(), 1);
        assert_eq!(default_weapon(3), None, "bombs (the counter) is not a weapon");
        assert_eq!(default_weapon(23).unwrap().image, "wbomb3");
    }
}
