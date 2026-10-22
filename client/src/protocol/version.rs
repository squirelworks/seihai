//! Client version profiles.
//!
//! A single switch that selects the version token, login client type,
//! encryption generation, and a few other version-specific behaviours.
//! Values come from cleaned observations (build catalogue, cross-version
//! notes). No decompile material.

/// Encryption generation used on the game socket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionGen {
    /// No compression (through ~1.1.9).
    Gen1,
    /// zlib only (~1.21–1.39).
    Gen2,
    /// zlib + one LCG-inserted byte per client→server message (~1.41–2.05/2.17).
    Gen3,
    /// Full-frame partial XOR (2.22+).
    Gen5,
}

/// A client version Seihai can present as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientVersion {
    /// Graal 1.33 (`GNW20020`). Gen 2.
    V133,
    /// Graal 1.34 (`GNW22030`). Gen 2.
    V134,
    /// Graal 1.35 (`GNW13040`). Gen 2.
    V135,
    /// Graal 1.36 (`GNW10050`). Gen 2.
    V136,
    /// Graal 1.37 (`GNW21050`). Gen 2.
    V137,
    /// Graal 1.38 (`GNW22060`). Gen 2.
    V138,
    /// Graal 1.39 (`GNW07080`). Gen 2.
    V139,
    /// Graal 1.41 (`GNW13110`). First gen 3.
    V141,
    /// Graal 1.42 NewMain (`GNW08011`). Gen 3.
    V142,
    /// Graal 2.00 (`GNW24011`). Gen 3.
    V200,
    /// Graal 2.01 (`GNW23021`). Gen 3.
    V201,
    /// Graal 2.02 (`GNW15041`). Gen 3.
    V202,
    /// Graal 2.05 rev 2 (`GNW03081`). Gen 3.
    V205,
    /// Graal 2.17 (`GNW22122`). Gen 3.
    V217,
    /// Graal 2.22 (`GNW03014`). Gen 5.
    V222,
    /// Graal 2.31 (`GNW28015`). Gen 5.
    V231,
}

impl ClientVersion {
    /// Eight-character version token sent at login.
    pub fn token(self) -> &'static str {
        match self {
            Self::V133 => "GNW20020",
            Self::V134 => "GNW22030",
            Self::V135 => "GNW13040",
            Self::V136 => "GNW10050",
            Self::V137 => "GNW21050",
            Self::V138 => "GNW22060",
            Self::V139 => "GNW07080",
            Self::V141 => "GNW13110",
            Self::V142 => "GNW08011",
            Self::V200 => "GNW24011",
            Self::V201 => "GNW23021",
            Self::V202 => "GNW15041",
            Self::V205 => "GNW03081",
            Self::V217 => "GNW22122",
            Self::V222 => "GNW03014",
            Self::V231 => "GNW28015",
        }
    }

    pub fn client_type(self) -> u8 {
        match self {
            Self::V222 | Self::V231 => 5,
            _ => 0,
        }
    }

    pub fn encryption(self) -> EncryptionGen {
        match self {
            Self::V133 | Self::V134 | Self::V135 | Self::V136 | Self::V137 | Self::V138
            | Self::V139 => EncryptionGen::Gen2,
            Self::V222 | Self::V231 => EncryptionGen::Gen5,
            _ => EncryptionGen::Gen3,
        }
    }

    pub fn sends_tampercheck(self) -> bool {
        matches!(self, Self::V217 | Self::V222 | Self::V231)
    }

    pub fn listserver_port(self) -> u16 {
        match self {
            Self::V133 | Self::V134 | Self::V135 | Self::V136 | Self::V137 | Self::V138
            | Self::V139 => 14900,
            Self::V141 => 14923,
            Self::V222 | Self::V231 => 14922,
            _ => 14921,
        }
    }

    pub fn listserver_token(self) -> &'static str {
        match self {
            Self::V222 | Self::V231 => "GNW30123",
            _ => self.token(),
        }
    }

    /// Display name for the GUI.
    pub fn label(self) -> &'static str {
        match self {
            Self::V133 => "1.33",
            Self::V134 => "1.34",
            Self::V135 => "1.35",
            Self::V136 => "1.36",
            Self::V137 => "1.37",
            Self::V138 => "1.38",
            Self::V139 => "1.39",
            Self::V141 => "1.41",
            Self::V142 => "1.42",
            Self::V200 => "2.00",
            Self::V201 => "2.01",
            Self::V202 => "2.02",
            Self::V205 => "2.05",
            Self::V217 => "2.17",
            Self::V222 => "2.22",
            Self::V231 => "2.31",
        }
    }

    /// Main versions shown by default.
    pub fn main_versions() -> &'static [ClientVersion] {
        &[Self::V139, Self::V205, Self::V217, Self::V222, Self::V231]
    }

    /// All analysed versions.
    pub fn all_versions() -> &'static [ClientVersion] {
        &[
            Self::V133, Self::V134, Self::V135, Self::V136, Self::V137, Self::V138, Self::V139,
            Self::V141, Self::V142, Self::V200, Self::V201, Self::V202, Self::V205, Self::V217,
            Self::V222, Self::V231,
        ]
    }
}

impl Default for ClientVersion {
    fn default() -> Self {
        Self::V205
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_eight_chars() {
        for v in [
            ClientVersion::V139,
            ClientVersion::V205,
            ClientVersion::V217,
            ClientVersion::V222,
            ClientVersion::V231,
        ] {
            assert_eq!(v.token().len(), 8);
        }
    }

    #[test]
    fn gen3_versions() {
        assert_eq!(ClientVersion::V205.encryption(), EncryptionGen::Gen3);
        assert_eq!(ClientVersion::V217.encryption(), EncryptionGen::Gen3);
        assert_eq!(ClientVersion::V222.encryption(), EncryptionGen::Gen5);
    }
}
