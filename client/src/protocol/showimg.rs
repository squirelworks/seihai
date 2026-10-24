//! `PLI_SHOWIMGPLAYER` (24) as the 2.22 and 2.31 clients send it.
//!
//! Both clients send one `GCHAR 9` at login and then again on a fixed
//! period (lab, 2026-10-24, real clients, nothing altered):
//!
//! | build | period | condition |
//! |---|---|---|
//! | 2.22 | about 5.1 s | only while another player is in the level (0 sends in 3 solo sessions) |
//! | 2.31 | median 5.13 s (5.11 to 5.17) | always, idle or walking, in every level used |
//!
//! The 9 never changed and level changes neither trigger nor reset it. What
//! it means is **unresolved**: GServer-v2 relays any payload unchanged to the
//! other players in the level as `PLO_SHOWIMG` (32, `GSHORT sender id` plus
//! the client's bytes), and two real clients showed no reaction to payloads
//! from 0 to 95 or to an empty one. It is not the four-field emote form of
//! 2.05.
//!
//! Seihai therefore sends it only when asked (`--send-showimg`). The period
//! and condition come from the profile, never from the version.

use super::version::ClientVersion;
use std::time::Duration;

/// Client to server id.
pub const PLI_SHOWIMGPLAYER: u8 = 24;
/// The one-byte payload every real client sent.
pub const SHOWIMG_VALUE: u8 = 9;

/// What a profile records about id 24.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShowImgInfo {
    /// Time between sends.
    pub period: Duration,
    /// True when the client sends only while another player is in its level.
    pub needs_others: bool,
}

/// The wire message: `GCHAR 24`, `GCHAR 9`, newline.
pub fn showimg_message() -> Vec<u8> {
    vec![PLI_SHOWIMGPLAYER + 0x20, SHOWIMG_VALUE + 0x20, b'\n']
}

/// Decides when id 24 goes out. The first send is due at once (the real
/// clients send it with their first frame after login), so long as the
/// profile's condition holds.
#[derive(Debug, Clone)]
pub struct ShowImgSender {
    info: ShowImgInfo,
    /// Time since the last send, capped at one period.
    since: Duration,
}

impl ShowImgSender {
    /// `None` for a profile whose build has no recorded id 24 behaviour.
    pub fn for_version(version: ClientVersion) -> Option<Self> {
        let info = version.profile().protocol.showimg?;
        Some(Self { info, since: info.period })
    }

    pub fn info(&self) -> ShowImgInfo {
        self.info
    }

    /// `dt` has passed; `others_present` is true when another player is in
    /// the level. True when a send is due. At most one per call, and a
    /// stalled caller does not get a burst. While the condition does not
    /// hold nothing is due and the clock keeps its value, so the send comes
    /// as soon as another player shows up.
    pub fn advance(&mut self, dt: Duration, others_present: bool) -> bool {
        self.since = (self.since + dt).min(self.info.period);
        if self.info.needs_others && !others_present {
            return false;
        }
        if self.since >= self.info.period {
            self.since = Duration::ZERO;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::PROFILES;

    const MS: fn(u64) -> Duration = Duration::from_millis;

    #[test]
    fn message_is_one_gchar_9() {
        assert_eq!(showimg_message(), [24 + 0x20, 9 + 0x20, b'\n']);
    }

    /// Only profiles that can send id 24 carry the record; the two lab
    /// builds are the ones that have it.
    #[test]
    fn profile_records_follow_the_send_set_and_the_lab() {
        for p in PROFILES {
            if p.protocol.showimg.is_some() {
                assert!(p.protocol.pli.contains(PLI_SHOWIMGPLAYER), "{}", p.label);
            }
        }
        let with: Vec<&str> =
            PROFILES.iter().filter(|p| p.protocol.showimg.is_some()).map(|p| p.label).collect();
        assert_eq!(with, ["2.22", "2.31"]);
        let i222 = ClientVersion::V222.profile().protocol.showimg.unwrap();
        let i231 = ClientVersion::V231.profile().protocol.showimg.unwrap();
        assert!(i222.needs_others && !i231.needs_others);
        assert_eq!(i222.period, MS(5100));
        assert_eq!(i231.period, MS(5130));
        assert!(ShowImgSender::for_version(ClientVersion::V217).is_none());
        assert!(ShowImgSender::for_version(ClientVersion::V205).is_none());
    }

    /// 2.31: first send at once, then one per period, alone or not.
    #[test]
    fn v231_sends_at_once_then_every_period() {
        let mut s = ShowImgSender::for_version(ClientVersion::V231).unwrap();
        assert!(s.advance(MS(0), false));
        let mut sends = 0;
        // 20 s in 50 ms steps: sends at 5.13, 10.26 and 15.39 s.
        for _ in 0..400 {
            sends += usize::from(s.advance(MS(50), false));
        }
        assert_eq!(sends, 3);
        // One send per call even after a long stall.
        assert!(s.advance(Duration::from_secs(60), false));
        assert!(!s.advance(MS(50), false));
    }

    /// 2.22: nothing while alone, then a send as soon as someone is there,
    /// then one per period while someone stays.
    #[test]
    fn v222_sends_only_with_other_players() {
        let mut s = ShowImgSender::for_version(ClientVersion::V222).unwrap();
        for _ in 0..4000 {
            assert!(!s.advance(MS(50), false));
        }
        assert!(s.advance(MS(50), true));
        let mut sends = 0;
        for _ in 0..400 {
            sends += usize::from(s.advance(MS(50), true));
        }
        assert_eq!(sends, 3);
        // The other player leaves: it stops again.
        for _ in 0..400 {
            assert!(!s.advance(MS(50), false));
        }
    }
}
