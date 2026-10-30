//! What a 3.x build says on the Graal TCP wire that 2.31 does not.
//!
//! Nothing is restated here: the token, the listserver token and the id
//! sets are read from the build's row in `seihai-core`
//! (`protocol::chain`, from Graal_analysis `version-profiles/3.04-3.153`),
//! and the "3.x-only" ids are that row compared with 2.31's. The framing,
//! board, gmap and GS1 code is shared with `seihai-mae` through the core.

use seihai_core::protocol::{ClientVersion, IdSet};

/// The build Ato compares against: the last 2.x client, whose row the 3.x
/// rows are written as changes to.
pub const BASE: ClientVersion = ClientVersion::V231;

/// Ids in `a` and not in `b`.
fn minus(a: &IdSet, b: &IdSet) -> Vec<u8> {
    a.iter().filter(|id| !b.contains(*id)).collect()
}

/// A 3.x build's wire facts.
#[derive(Debug, Clone, PartialEq)]
pub struct Wire {
    pub version: ClientVersion,
    pub label: &'static str,
    /// Game-server login token (`G3D…`).
    pub token: &'static str,
    /// Listserver login token.
    pub list_token: &'static str,
    /// First GCHAR of the login line.
    pub client_type: u8,
    /// Server→client ids the build handles and 2.31 does not.
    pub plo_added: Vec<u8>,
    /// Server→client ids 2.31 handles and the build does not.
    pub plo_removed: Vec<u8>,
    /// Client→server ids the build sends and 2.31 does not.
    pub pli_added: Vec<u8>,
    /// Client→server ids 2.31 sends and the build does not.
    pub pli_removed: Vec<u8>,
}

impl Wire {
    pub fn of(v: ClientVersion) -> Wire {
        let p = v.profile();
        let base = BASE.profile();
        Wire {
            version: v,
            label: p.label,
            token: p.token,
            list_token: v.listserver_token(),
            client_type: p.protocol.client_type,
            plo_added: minus(&p.protocol.plo, &base.protocol.plo),
            plo_removed: minus(&base.protocol.plo, &p.protocol.plo),
            pli_added: minus(&p.protocol.pli, &base.protocol.pli),
            pli_removed: minus(&base.protocol.pli, &p.protocol.pli),
        }
    }

    /// The report `--print-profile` prints.
    pub fn report(&self) -> String {
        fn ids(v: &[u8]) -> String {
            if v.is_empty() {
                "none".into()
            } else {
                v.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", ")
            }
        }
        format!(
            "build {}\ngame token {} (client type {})\nlistserver token {}\nserver->client ids, added against {}: {}\nserver->client ids, dropped against {}: {}\nclient->server ids, added against {}: {}\nclient->server ids, dropped against {}: {}\n",
            self.label,
            self.token,
            self.client_type,
            self.list_token,
            BASE.label(),
            ids(&self.plo_added),
            BASE.label(),
            ids(&self.plo_removed),
            BASE.label(),
            ids(&self.pli_added),
            BASE.label(),
            ids(&self.pli_removed),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens() {
        let w = Wire::of(ClientVersion::V304);
        assert_eq!((w.token, w.list_token), ("G3D03014", "G3D30123"));
        let w = Wire::of(ClientVersion::V3153);
        assert_eq!((w.token, w.list_token), ("G3D28015", "G3D30123"));
    }

    #[test]
    fn three_oh_four_against_231() {
        // The 3.04 step in core's chain: PLO 44, 101, 102 in and 170, 177,
        // 182, 183, 189 out; PLI 21, 36, 81 in and 39, 44, 46 out.
        let w = Wire::of(ClientVersion::V304);
        assert_eq!(w.plo_added, vec![44, 101, 102]);
        assert_eq!(w.plo_removed, vec![170, 177, 182, 183, 189]);
        assert_eq!(w.pli_added, vec![21, 36, 81]);
        assert_eq!(w.pli_removed, vec![39, 44, 46]);
    }

    #[test]
    fn three_one_five_three_is_both_steps() {
        // 3.153 is written as a change to 3.04, so against 2.31 it carries
        // both steps: 189 (MOVE2) comes back, 44 goes again.
        let w = Wire::of(ClientVersion::V3153);
        for id in [67, 74, 82, 101, 102, 104, 131, 132, 133, 134, 187] {
            assert!(w.plo_added.contains(&id), "PLO {id}");
        }
        assert!(!w.plo_added.contains(&44) && !w.plo_removed.contains(&189));
        for id in [21, 36, 50, 81, 93, 96, 97, 98, 152, 154, 155, 156, 157, 158] {
            assert!(w.pli_added.contains(&id) || BASE.profile().protocol.pli.contains(id), "PLI {id}");
        }
        assert!(w.pli_removed.contains(&39) && w.pli_removed.contains(&44) && !w.pli_removed.contains(&46));
    }
}
