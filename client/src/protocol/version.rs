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
    /// Graal 1.39 (`GNW07080`). Gen 2, client type varies.
    V139,
    /// Graal 2.05 rev 2 (`GNW03081`). Gen 3, client type 0.
    V205,
    /// Graal 2.17 (`GNW22122`). Still gen 3, client type 0.
    V217,
    /// Graal 2.22 (`GNW03014`). Gen 5, client type 5.
    V222,
    /// Graal 2.31 (`GNW28015`). Gen 5, client type 5.
    V231,
}

impl ClientVersion {
    /// Eight-character version token sent at login.
    pub fn token(self) -> &'static str {
        match self {
            Self::V139 => "GNW07080",
            Self::V205 => "GNW03081",
            Self::V217 => "GNW22122",
            Self::V222 => "GNW03014",
            Self::V231 => "GNW28015",
        }
    }

    /// Client type byte sent as the first byte of the game login.
    pub fn client_type(self) -> u8 {
        match self {
            Self::V139 => 0,
            Self::V205 => 0,
            Self::V217 => 0,
            Self::V222 => 5,
            Self::V231 => 5,
        }
    }

    /// Encryption generation used after the login frame.
    pub fn encryption(self) -> EncryptionGen {
        match self {
            Self::V139 => EncryptionGen::Gen2,
            Self::V205 => EncryptionGen::Gen3,
            Self::V217 => EncryptionGen::Gen3,
            Self::V222 => EncryptionGen::Gen5,
            Self::V231 => EncryptionGen::Gen5,
        }
    }

    /// True if the client sends `PLI_TAMPERCHECK` (id 39) after login.
    pub fn sends_tampercheck(self) -> bool {
        matches!(self, Self::V217 | Self::V222 | Self::V231)
    }

    /// Default listserver port for this version.
    pub fn listserver_port(self) -> u16 {
        match self {
            Self::V139 => 14900,
            Self::V205 => 14921,
            Self::V217 => 14922,
            Self::V222 => 14922,
            Self::V231 => 14923,
        }
    }

    /// Listserver token (when distinct from the game token).
    pub fn listserver_token(self) -> &'static str {
        match self {
            Self::V222 | Self::V231 => "GNW30123",
            _ => self.token(),
        }
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
