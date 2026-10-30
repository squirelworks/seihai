//! Differential-testing tool.
//!
//! ```text
//! difftest trace <capture.pcap> --version 2.31 --out trace.tsv
//!                [--scenario NAME] [--client real|seihai] [--no-mask]
//! difftest diff <real.trace.tsv> <seihai.trace.tsv> --out PREFIX
//!                [--jitter-ms N] [--pos-tol TILES] [--fail-on high|medium|low]
//! ```
//!
//! `trace` decodes a pcap with Seihai's own codecs into a normalized,
//! masked per-message trace. `diff` compares a real-client trace with a
//! Seihai trace, prints a ranked report, and writes `PREFIX.txt` and
//! `PREFIX.tsv`. Capture files hold login frames: keep them in a private
//! directory, never in a repository.

use seihai::difftest::decode::{decode_pcap, DecodeOptions};
use seihai::difftest::diff::{diff, DiffOptions, Severity};
use seihai::difftest::trace::Trace;
use seihai::protocol::ClientVersion;
use std::process::exit;

fn usage() -> ! {
    eprintln!("usage: difftest trace <capture.pcap> --version V --out FILE [--scenario S] [--client K] [--no-mask]");
    eprintln!("       difftest diff <real.tsv> <seihai.tsv> --out PREFIX [--jitter-ms N] [--pos-tol T] [--fail-on SEV]");
    exit(2)
}

fn opt(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("trace") => cmd_trace(&args[1..]),
        Some("diff") => cmd_diff(&args[1..]),
        _ => usage(),
    }
}

fn cmd_trace(args: &[String]) {
    let Some(pcap) = args.first().filter(|a| !a.starts_with("--")) else { usage() };
    let Some(ver) = opt(args, "--version") else { usage() };
    let Some(out) = opt(args, "--out") else { usage() };
    let Some(version) = ClientVersion::from_label(&ver) else {
        eprintln!("unknown version {ver}");
        exit(2)
    };
    let data = std::fs::read(pcap).unwrap_or_else(|e| {
        eprintln!("cannot read {pcap}: {e}");
        exit(1)
    });
    let mut o = DecodeOptions::new(version);
    o.mask = !args.iter().any(|a| a == "--no-mask");
    o.client = opt(args, "--client").unwrap_or_default();
    o.scenario = opt(args, "--scenario").unwrap_or_default();
    let trace = decode_pcap(&data, &o).unwrap_or_else(|e| {
        eprintln!("{e}");
        exit(1)
    });
    if let Err(e) = std::fs::write(&out, trace.to_tsv()) {
        eprintln!("cannot write {out}: {e}");
        exit(1)
    }
    let c2s = trace.events.iter().filter(|e| e.dir == seihai::difftest::trace::Dir::C2S).count();
    println!(
        "trace {out}: {} connection(s), {} messages ({} C>S, {} S>C), {} bad frame(s)",
        trace.meta("connections").unwrap_or("0"),
        trace.events.len(),
        c2s,
        trace.events.len() - c2s,
        trace.meta("bad_frames").unwrap_or("0")
    );
}

fn read_trace(path: &str) -> Trace {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("cannot read {path}: {e}");
        exit(1)
    });
    Trace::from_tsv(&text).unwrap_or_else(|e| {
        eprintln!("{path}: {e}");
        exit(1)
    })
}

fn cmd_diff(args: &[String]) {
    let (Some(a), Some(b)) = (args.first(), args.get(1)) else { usage() };
    let Some(out) = opt(args, "--out") else { usage() };
    let mut o = DiffOptions::default();
    if let Some(j) = opt(args, "--jitter-ms").and_then(|s| s.parse().ok()) {
        o.jitter_ms = j;
    }
    if let Some(t) = opt(args, "--pos-tol").and_then(|s| s.parse().ok()) {
        o.pos_tol = t;
    }
    let report = diff(&read_trace(a), &read_trace(b), &o);
    let text = report.to_text();
    print!("{text}");
    for (ext, body) in [("txt", text), ("tsv", report.to_tsv())] {
        let path = format!("{out}.{ext}");
        if let Err(e) = std::fs::write(&path, body) {
            eprintln!("cannot write {path}: {e}");
            exit(1)
        }
    }
    if let Some(level) = opt(args, "--fail-on") {
        let worst = match level.as_str() {
            "high" => Severity::High,
            "medium" => Severity::Medium,
            "low" => Severity::Low,
            _ => usage(),
        };
        if report.ranked().any(|d| d.severity <= worst) {
            exit(3)
        }
    }
}
