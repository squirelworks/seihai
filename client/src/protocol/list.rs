//! Listserver reply parser.
//!
//! Provisional. The login line is written; the reply format is not yet
//! locked against a capture. This parser accepts the shapes seen in
//! public notes: a version echo, then server lines, then a terminator.

/// One server entry from a listserver reply.
#[derive(Debug, Clone)]
pub struct ServerEntry {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub players: Option<u16>,
}

/// Parse a listserver reply body into server entries.
///
/// Accepts lines of the form:
/// - `name,host,port` or `name,host,port,players`
/// - lines starting with the version token are ignored
/// - empty lines are ignored
pub fn parse_list_reply(body: &str) -> Vec<ServerEntry> {
    let mut out = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("GNW") || line.starts_with("G3D") {
            continue;
        }
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 3 {
            if let Ok(port) = parts[2].parse::<u16>() {
                let players = parts.get(3).and_then(|p| p.parse().ok());
                out.push(ServerEntry {
                    name: parts[0].to_string(),
                    host: parts[1].to_string(),
                    port,
                    players,
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic() {
        let body = "GNW03081\nClassic,example.com,14912,3\n";
        let entries = parse_list_reply(body);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Classic");
        assert_eq!(entries[0].port, 14912);
        assert_eq!(entries[0].players, Some(3));
    }
}
