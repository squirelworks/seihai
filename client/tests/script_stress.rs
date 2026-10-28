//! Parse every unique NPC script in the fixtures and report failures.
//!
//! Run with: cargo test -p seihai --test script_stress -- --nocapture

use seihai::scripting::engine::{parse, Era};
use std::fs;
use std::path::Path;

fn scripts_in(path: &Path) -> Vec<(String, String)> {
    let text = fs::read_to_string(path).unwrap_or_default();
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut name = String::new();
    for line in text.lines() {
        if line.starts_with("// ===== ") {
            if !cur.is_empty() {
                out.push((name.clone(), cur.clone()));
                cur.clear();
            }
            name = line.trim_start_matches("// ===== ").trim_end_matches(" =====").to_string();
        } else if line.starts_with("NPC ") {
            cur.clear();
        } else if line == "NPCEND" {
            if !cur.is_empty() {
                out.push((name.clone(), cur.clone()));
                cur.clear();
            }
        } else if !line.starts_with("//") {
            cur.push_str(line);
            cur.push('\n');
        }
    }
    if !cur.is_empty() {
        out.push((name, cur));
    }
    out
}

#[test]
fn parse_all_unique_npc_scripts() {
    let dir = Path::new("client/tests/fixtures/npc-scripts");
    let mut total = 0;
    let mut failed = 0;
    let mut diags: Vec<(String, String)> = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map_or(true, |e| e != "txt") {
            continue;
        }
        for (name, body) in scripts_in(&path) {
            total += 1;
            let s = parse(&body, Era::V205);
            if !s.diagnostics.is_empty() {
                failed += 1;
                if diags.len() < 40 {
                    let msg = s.diagnostics.iter().map(|d| format!("L{}: {}", d.line, d.msg)).collect::<Vec<_>>().join("; ");
                    diags.push((name, msg));
                }
            }
        }
    }
    eprintln!("parsed {total} scripts, {failed} with diagnostics");
    for (n, m) in &diags {
        eprintln!("  {n}: {m}");
    }
    // Informational: do not fail the build. The count is the point.
    assert!(total > 1000, "expected the full corpus");
}
