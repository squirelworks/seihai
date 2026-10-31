//! The Graal3D admin protocol (Graal_analysis `graal3d/protocol.md`,
//! "Admin commands"), for a future admin window or command line.
//!
//! Scaffold: the ids and the encoders whose layouts were seen live are
//! here and tested. Nothing in the client uses them yet. TODO(G3D-5): an
//! admin session (`--admin`) that logs in with id 1, lists accounts (50)
//! and players (55), and can message, kick and reload levels.

use crate::codec::{c, message, put_sn, put_str};

/// Admin login: the player login layout under id 1. Needs an account with
/// admin level 1 or more.
pub const CL_ADMIN_LOGIN: u8 = 1;

/// Client → server admin ids.
pub mod cl {
    pub const GET_ACCOUNT: u8 = 51;
    pub const EDIT_ACCOUNT: u8 = 52;
    pub const ADD_ACCOUNT: u8 = 53;
    pub const DELETE_ACCOUNT: u8 = 54;
    pub const GET_PLAYER_ACCOUNT: u8 = 59;
    pub const EDIT_PLAYER_ACCOUNT: u8 = 60;
    pub const KICK: u8 = 61;
    pub const RELOAD_LEVELS: u8 = 62;
    pub const MESSAGE_ALL: u8 = 63;
    pub const MESSAGE_ONE: u8 = 64;
}

/// Server → admin ids.
pub mod sv {
    pub const ACCOUNT_ADDED: u8 = 50;
    pub const ACCOUNT_CHANGED: u8 = 51;
    pub const ACCOUNT_RENAMED: u8 = 52;
    pub const ACCOUNT_DELETED: u8 = 53;
    pub const ACCOUNT_RECORD: u8 = 54;
    pub const PLAYER_JOINED: u8 = 55;
    pub const PLAYER_LEFT: u8 = 56;
    pub const PLAYER_ACCOUNT: u8 = 57;
    pub const PLAYER_ACCOUNT_RENAMED: u8 = 58;
    pub const PLAYER_PROPS: u8 = 59;
}

/// The admin login line.
pub fn login(account: &str, password: &str) -> Vec<u8> {
    let mut b = crate::session::TOKEN.to_vec();
    put_str(&mut b, account.as_bytes());
    put_str(&mut b, password.as_bytes());
    message(CL_ADMIN_LOGIN, &b)
}

/// Account ids are 28 bits, sent as 4 × 7 bits.
pub fn put_account_id(out: &mut Vec<u8>, id: u32) {
    put_sn(out, id & 0x0fff_ffff, 4);
}

/// 61: disconnect a player. They get "The server administrator has
/// disconnected you."
pub fn kick(player: u8) -> Vec<u8> {
    message(cl::KICK, &[c(player)])
}

/// 63: a message to every player, shown as `Administrator <account>:` and
/// the text. Line breaks become `§`.
pub fn message_all(text: &str) -> Vec<u8> {
    message(cl::MESSAGE_ALL, &escape(text))
}

/// 64: a message to one player.
pub fn message_one(player: u8, text: &str) -> Vec<u8> {
    let mut b = vec![c(player)];
    b.extend(escape(text));
    message(cl::MESSAGE_ONE, &b)
}

/// 62: reload these levels from disk.
pub fn reload_levels(names: &[&str]) -> Vec<u8> {
    let mut b = Vec::new();
    put_sn(&mut b, names.len() as u32, 2);
    for n in names {
        put_str(&mut b, n.as_bytes());
    }
    message(cl::RELOAD_LEVELS, &b)
}

/// 51: ask for an account record (reply 54).
pub fn get_account(id: u32) -> Vec<u8> {
    let mut b = Vec::new();
    put_account_id(&mut b, id);
    message(cl::GET_ACCOUNT, &b)
}

// TODO(G3D-5): 52 edit, 53 add, 54 delete, 59 player's account and 60 edit
// an online player's account need the account record layout (account id,
// name, password, saved props, flags); only the 54 reply was seen live.

fn escape(text: &str) -> Vec<u8> {
    text.bytes().map(|b| if b == b'\n' { crate::codec::LINE_BREAK } else { b }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoders() {
        assert_eq!(login("alice", "pw1"), b"!GR3DV002%alice#pw1");
        assert_eq!(kick(2), b"]\"");
        assert_eq!(message_all("hi\nall"), b"_hi\xa7all");
        assert_eq!(message_one(1, "x"), b"`!x");
        assert_eq!(reload_levels(&["a.x"]), b"^ !#a.x");
        assert_eq!(get_account(1), b"S   !");
    }
}
