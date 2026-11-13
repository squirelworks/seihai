//! Reading signs in the shared loop (#445), with the 2.05 rules Mae uses
//! (`documentation/signs.md`): core opens a sign when the player walks or
//! pushes up into it; the sign shows three lines a page; Up goes back a
//! page, the other game keys go on and close it on the last page, and
//! Escape (the map key's place here) closes it at once. While a sign is
//! open the player does not walk, swing, lift or use an item.

use super::Game;

/// Lines on one page of a sign.
pub const LINES_PER_PAGE: usize = 3;

/// What a key does to an open sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignKey {
    /// Up: the page before.
    Back,
    /// Escape: stop reading at once.
    Close,
    /// Any other game key: the next page, or stop on the last.
    Next,
}

/// The first line of the page after the one starting at `first`, or None
/// when the sign ends there.
pub fn next_page(text: &str, first: usize) -> Option<usize> {
    let lines = text.split('\n').count();
    (lines >= first + LINES_PER_PAGE).then_some(first + LINES_PER_PAGE)
}

/// The first line of the page before, or None on the first page.
pub fn previous_page(first: usize) -> Option<usize> {
    (first >= LINES_PER_PAGE).then(|| first - LINES_PER_PAGE)
}

impl Game {
    /// The lines of the sign page on screen, while a sign is open.
    pub fn sign_page(&self) -> Option<Vec<String>> {
        let text = self.session.reading.as_deref()?;
        Some(
            text.split('\n')
                .skip(self.sign_first)
                .take(LINES_PER_PAGE)
                .map(str::to_string)
                .collect(),
        )
    }

    /// True while a sign is open.
    pub fn reading(&self) -> bool {
        self.session.reading.is_some()
    }

    /// A key on the open sign.
    pub fn sign_key(&mut self, key: SignKey) {
        let Some(text) = self.session.reading.as_deref() else {
            return;
        };
        match key {
            SignKey::Close => self.session.reading = None,
            SignKey::Back => {
                self.sign_first = previous_page(self.sign_first).unwrap_or(self.sign_first)
            }
            SignKey::Next => match next_page(text, self.sign_first) {
                Some(p) => self.sign_first = p,
                None => self.session.reading = None,
            },
        }
        if self.session.reading.is_none() {
            self.sign_first = 0;
        }
        self.dirty = true;
    }

    /// A sign opened since the last frame starts on its first page.
    pub(super) fn notice_sign(&mut self) {
        let open = self.session.reading.is_some();
        if open && !self.sign_was_open {
            self.sign_first = 0;
            self.dirty = true;
        }
        self.sign_was_open = open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_lines_a_page() {
        let text = "a\nb\nc\nd\ne";
        assert_eq!(next_page(text, 0), Some(3));
        assert_eq!(next_page(text, 3), None);
        assert_eq!(next_page("a\nb\nc", 0), Some(3));
        assert_eq!(previous_page(3), Some(0));
        assert_eq!(previous_page(0), None);
    }

    /// A sign the walk opened pages on with the keys and closes on the
    /// last page; Escape closes it at once.
    #[test]
    fn the_keys_page_and_close_the_sign() {
        let (tx, rx) = std::sync::mpsc::channel();
        drop(tx);
        let mut g = Game::new(rx, None, true).unwrap();
        g.session.reading = Some("one\ntwo\nthree\nfour".into());
        g.notice_sign();
        assert_eq!(g.sign_page().unwrap(), vec!["one", "two", "three"]);
        g.sign_key(SignKey::Next);
        assert_eq!(g.sign_page().unwrap(), vec!["four"]);
        g.sign_key(SignKey::Back);
        assert_eq!(g.sign_page().unwrap()[0], "one");
        g.sign_key(SignKey::Next);
        g.sign_key(SignKey::Next);
        assert!(!g.reading());
        g.session.reading = Some("x".into());
        g.notice_sign();
        g.sign_key(SignKey::Close);
        assert!(!g.reading());
    }
}
