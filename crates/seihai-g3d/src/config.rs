//! `graal3dconfig.txt`, the client's settings file (Graal_analysis
//! `graal3d/formats.md`): `key=value` lines, read at start and written back
//! when settings change.
//!
//! Seihai reads it from the asset folder for the player's look and the UDP
//! options. The server address in the shipped file is a 2000-era public
//! host: Seihai parses it but never connects to it on its own; the server
//! always comes from `--online`.

use crate::props::Player;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub server_addr: Option<String>,
    pub server_port: Option<u16>,
    /// Bind UDP to `fixed_udp_port` instead of any free port.
    pub fixed_port: bool,
    pub fixed_udp_port: u16,
    pub no_udp: bool,
    pub body_model: String,
    pub head_model: String,
    pub body_texture: String,
    pub arm_colour: u32,
    pub shoe_colour: u32,
    pub hand_colour: u32,
    pub account: Option<String>,
    /// Keys Seihai does not know, kept so a write-back loses nothing.
    pub other: Vec<(String, String)>,
}

impl Default for Config {
    /// The shipped values, without the server address.
    fn default() -> Self {
        Config {
            server_addr: None,
            server_port: None,
            fixed_port: false,
            fixed_udp_port: 14898,
            no_udp: false,
            body_model: "body.x".into(),
            head_model: "head.x".into(),
            body_texture: "body.bmp".into(),
            arm_colour: 0xFF0000,
            shoe_colour: 0x800000,
            hand_colour: 0xFDCC47,
            account: None,
            other: Vec::new(),
        }
    }
}

fn colour(v: &str) -> Option<u32> {
    let v = v.trim();
    let hex = v.strip_prefix("0x").or_else(|| v.strip_prefix("0X")).or_else(|| v.strip_prefix('$')).unwrap_or(v);
    u32::from_str_radix(hex, 16).ok().filter(|c| *c <= 0xFFFFFF)
}

fn flag(v: &str) -> bool {
    matches!(v.trim().to_ascii_lowercase().as_str(), "true" | "1" | "yes")
}

impl Config {
    /// Parse the file. Bad values keep the default; unknown keys are kept.
    pub fn parse(text: &str) -> Config {
        let mut c = Config::default();
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else { continue };
            let (k, v) = (k.trim(), v.trim());
            match k.to_ascii_lowercase().as_str() {
                "serveraddr" => c.server_addr = Some(v.to_string()).filter(|s| !s.is_empty()),
                "serverport" => c.server_port = v.parse().ok(),
                "fixedport" => c.fixed_port = flag(v),
                "fixedudpport" => c.fixed_udp_port = v.parse().unwrap_or(c.fixed_udp_port),
                "noudp" => c.no_udp = flag(v),
                "bodyx" => c.body_model = v.to_string(),
                "headx" => c.head_model = v.to_string(),
                "bodytex" => c.body_texture = v.to_string(),
                "armcolor" => c.arm_colour = colour(v).unwrap_or(c.arm_colour),
                "shoecolor" => c.shoe_colour = colour(v).unwrap_or(c.shoe_colour),
                "handcolor" => c.hand_colour = colour(v).unwrap_or(c.hand_colour),
                "accountname" => c.account = Some(v.to_string()).filter(|s| !s.is_empty()),
                // The password is never read from or written to this file
                // by Seihai; it comes from `--password-env`.
                "password" => {}
                _ => c.other.push((k.to_string(), v.to_string())),
            }
        }
        c
    }

    /// Put the look into a player's props 6–8, 10, 11 and 17.
    pub fn apply_look(&self, p: &mut Player) {
        p.body_model = self.body_model.clone();
        p.head_model = self.head_model.clone();
        p.body_texture = self.body_texture.clone();
        p.arm_colour = self.arm_colour;
        p.shoe_colour = self.shoe_colour;
        p.hand_colour = self.hand_colour;
    }

    /// The file text, in the shipped key order.
    // TODO(G3D-4): the F2 "Player look" window writes this back.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        if let Some(a) = &self.server_addr {
            out += &format!("serveraddr={a}\n");
        }
        if let Some(p) = self.server_port {
            out += &format!("serverport={p}\n");
        }
        out += &format!("fixedport={}\nfixedudpport={}\nnoudp={}\n", self.fixed_port, self.fixed_udp_port, self.no_udp);
        out += &format!("bodyx={}\nheadx={}\nbodytex={}\n", self.body_model, self.head_model, self.body_texture);
        out += &format!(
            "armcolor=0x{:06X}\nshoecolor=0x{:06X}\nhandcolor=0x{:06X}\n",
            self.arm_colour, self.shoe_colour, self.hand_colour
        );
        if let Some(a) = &self.account {
            out += &format!("accountname={a}\n");
        }
        for (k, v) in &self.other {
            out += &format!("{k}={v}\n");
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHIPPED: &str = "serveraddr=<host>\nserverport=14901\nfixedport=false\nfixedudpport=14898\nnoudp=false\nbodyx=body.x\nheadx=head.x\nbodytex=body.bmp\narmcolor=0xFF0000\nshoecolor=0x800000\nhandcolor=0xFDCC47\n";

    #[test]
    fn reads_the_shipped_file() {
        let c = Config::parse(SHIPPED);
        assert_eq!(c.server_port, Some(14901));
        assert_eq!((c.arm_colour, c.shoe_colour, c.hand_colour), (0xFF0000, 0x800000, 0xFDCC47));
        assert!(!c.no_udp && !c.fixed_port);
        assert_eq!(c.to_text(), SHIPPED);
    }

    #[test]
    fn look_and_bad_values() {
        let c = Config::parse("armcolor=0x00FF00\nshoecolor=nonsense\nbodytex=body1.bmp\npassword=secret\nfoo=bar\n");
        let mut p = Player::default();
        c.apply_look(&mut p);
        assert_eq!((p.arm_colour, p.shoe_colour, p.body_texture.as_str()), (0x00FF00, 0x800000, "body1.bmp"));
        assert!(!c.to_text().contains("secret"));
        assert!(c.to_text().ends_with("foo=bar\n"));
    }
}
