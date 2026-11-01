#!/usr/bin/env bash
# Headless salvage analysis wrapper for recovered seihai-servertools binaries.
# Usage: analyze_tool.sh <binary_path> <summary_dir> [max_decomp_funcs]
#
# Raw Ghidra output (imports, strings, decompilation, disassembly) goes to
# tools/ghidra/local-out/<tool>/ which is gitignored. <summary_dir> receives
# SUMMARY.md only, and only when that file does not already exist. Fill the
# summary in by hand. Do not paste string or import dumps into it.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GHIDRA_HOME="${GHIDRA_HOME:-$SCRIPT_DIR/current}"
JDK_HOME="${JDK_HOME:-/workspace/tools/jdk/current}"
RIZIN_BIN="${RIZIN_BIN:-/workspace/tools/rizin/bin}"
PROJECTS_DIR="${GHIDRA_PROJECTS_DIR:-$SCRIPT_DIR/projects}"
SCRIPTS_DIR="${GHIDRA_SCRIPTS_DIR:-$SCRIPT_DIR/ghidra_scripts}"
MAXMEM="${GHIDRA_HEADLESS_MAXMEM:-4G}"
TIMEOUT_PER_FILE="${GHIDRA_ANALYSIS_TIMEOUT:-1200}"

usage() {
  cat <<EOF
Usage: $(basename "$0") <binary_path> <summary_dir> [max_decomp_funcs]

Runs Ghidra analyzeHeadless with auto-analysis.

Raw artifacts (gitignored, under GHIDRA_LOCAL_OUT):
  functions.txt / functions.csv
  imports.txt / exports.txt
  strings_interesting.txt
  decompiled/*.c (+ INDEX.txt)
  prelim/ (file, strings, readelf/objdump/rizin snapshots)
  logs/ and summary_stats.txt (counts only; no string bodies)

<summary_dir>/SUMMARY.md is created only if missing, as a hand-edit stub
with program counts. An existing summary is left alone. Never paste raw
strings, imports, or decompiled code into that file.

Environment:
  GHIDRA_HOME              default: $SCRIPT_DIR/current
  JDK_HOME                 default: /workspace/tools/jdk/current
  GHIDRA_HEADLESS_MAXMEM   default: 4G
  GHIDRA_ANALYSIS_TIMEOUT  seconds per file (default 1200)
  GHIDRA_PROJECTS_DIR      default: $SCRIPT_DIR/projects
  GHIDRA_LOCAL_OUT         default: $SCRIPT_DIR/local-out
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi
if [[ $# -lt 2 ]]; then
  usage >&2
  exit 2
fi

if [[ ! -f "$1" ]]; then
  echo "ERROR: binary not found: $1" >&2
  exit 1
fi
BINARY="$(readlink -f "$1")"
SUMMARY_DIR="$(mkdir -p "$2" && cd "$2" && pwd)"
MAX_DECOMP="${3:-40}"
if [[ ! -x "$GHIDRA_HOME/support/analyzeHeadless" ]]; then
  echo "ERROR: analyzeHeadless not found under $GHIDRA_HOME" >&2
  echo "Install Ghidra and symlink it to $SCRIPT_DIR/current" >&2
  exit 1
fi
if [[ ! -x "$JDK_HOME/bin/java" ]]; then
  echo "ERROR: JDK not found at $JDK_HOME (Ghidra 12.2 needs JDK 25+)" >&2
  exit 1
fi

export JAVA_HOME="$JDK_HOME"
export PATH="$JAVA_HOME/bin:${RIZIN_BIN}:$PATH"
export GHIDRA_HEADLESS_MAXMEM="$MAXMEM"

TOOL_NAME="$(basename "$BINARY")"
TOOL_NAME="${TOOL_NAME%.*}"
# printf, not echo: echo adds a newline and tr turns that newline into a trailing underscore.
SAFE_NAME="$(printf '%s' "$TOOL_NAME" | tr -c 'A-Za-z0-9._+-' '_')"
PROJECT_NAME="salvage_${SAFE_NAME}"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
RAW_ROOT="${GHIDRA_LOCAL_OUT:-$SCRIPT_DIR/local-out}"
RAW_DIR="$RAW_ROOT/$SAFE_NAME"
PRELIM="$RAW_DIR/prelim"
LOG_DIR="$RAW_DIR/logs"
mkdir -p "$PRELIM" "$LOG_DIR" "$PROJECTS_DIR" "$RAW_DIR/decompiled"

cat >"$RAW_DIR/DO_NOT_COMMIT.txt" <<EOF
Gitignored local salvage output for ${TOOL_NAME}.
Do not copy imports, strings, disassembly, decompiled C, or addresses
into a committed SUMMARY.md. Write that summary by hand.
EOF

echo "==> Salvage analysis"
echo "    binary  : $BINARY"
echo "    raw out : $RAW_DIR"
echo "    summary : $SUMMARY_DIR"
echo "    ghidra  : $GHIDRA_HOME"
echo "    java    : $($JAVA_HOME/bin/java -version 2>&1 | head -1)"

# --- Preliminary RE snapshots (fast, no Ghidra) ---
{
  echo "# file"
  file -L "$BINARY" || true
  echo
  echo "# size"
  ls -lh "$BINARY" || true
  echo
  echo "# sha256"
  sha256sum "$BINARY" || true
} >"$PRELIM/file.txt" 2>&1

strings -a -n 4 "$BINARY" >"$PRELIM/strings_all.txt" 2>/dev/null || true
# Local filter only. Hits stay under local-out and are not copied into SUMMARY.md.
grep -E -i \
  '(^/[A-Za-z]|[A-Za-z]:\\|[.](conf|cfg|ini|json|xml|ya?ml|toml|properties|log)\b|https?://|--[a-zA-Z]|usage|help|version|config|listen|bindtextdomain|socket|passwd|password|gserver|graal|seihai|\bnpc\b)' \
  "$PRELIM/strings_all.txt" 2>/dev/null | head -n 500 >"$PRELIM/strings_interesting_raw.txt" || true

if command -v readelf >/dev/null 2>&1; then
  readelf -h -l -d -s "$BINARY" >"$PRELIM/readelf.txt" 2>&1 || \
    echo "(not ELF or readelf failed)" >"$PRELIM/readelf.txt"
fi
if command -v objdump >/dev/null 2>&1; then
  objdump -T "$BINARY" >"$PRELIM/objdump_dynamic_symbols.txt" 2>&1 || true
  objdump -d "$BINARY" 2>/dev/null | head -n 400 >"$PRELIM/objdump_disasm_head.txt" || true
fi
if command -v rz-bin >/dev/null 2>&1; then
  rz-bin -I "$BINARY" >"$PRELIM/rz_bin_info.txt" 2>&1 || true
  rz-bin -i "$BINARY" >"$PRELIM/rz_bin_imports.txt" 2>&1 || true
  rz-bin -E "$BINARY" >"$PRELIM/rz_bin_exports.txt" 2>&1 || true
  rz-bin -z "$BINARY" 2>/dev/null | head -n 300 >"$PRELIM/rz_bin_strings_head.txt" || true
fi
if command -v r2 >/dev/null 2>&1; then
  # Older distro radare2 builds differ; keep flags minimal and never fail the pipeline.
  r2 -qq -c 'iI; ii; iE' "$BINARY" >"$PRELIM/radare2_info.txt" 2>&1 || true
fi

# --- Ghidra headless ---
HEADLESS_LOG="$LOG_DIR/analyzeHeadless_${STAMP}.log"
SCRIPT_LOG="$LOG_DIR/ghidra_script_${STAMP}.log"

echo "==> Running analyzeHeadless (this can take a while)..."
set +e
"$GHIDRA_HOME/support/analyzeHeadless" \
  "$PROJECTS_DIR" "$PROJECT_NAME" \
  -import "$BINARY" \
  -overwrite \
  -deleteProject \
  -analysisTimeoutPerFile "$TIMEOUT_PER_FILE" \
  -max-cpu "${GHIDRA_MAX_CPU:-2}" \
  -scriptPath "$SCRIPTS_DIR" \
  -postScript ExportSalvageArtifacts.java "$RAW_DIR" "$MAX_DECOMP" \
  -scriptlog "$SCRIPT_LOG" \
  -log "$HEADLESS_LOG"
GHIDRA_RC=$?
set -e

if [[ $GHIDRA_RC -ne 0 ]]; then
  echo "WARNING: analyzeHeadless exited with code $GHIDRA_RC (see $HEADLESS_LOG)" >&2
fi

# --- SUMMARY.md (hand-written; stub only if missing) ---
SUMMARY="$SUMMARY_DIR/SUMMARY.md"
STATS="$RAW_DIR/summary_stats.txt"

get_stat() {
  local key="$1" val=""
  if [[ -f "$STATS" ]]; then
    val="$(grep -E "^${key}=" "$STATS" | head -1 | cut -d= -f2- || true)"
  fi
  printf '%s' "$val"
}

if [[ -f "$SUMMARY" ]]; then
  echo "==> Left existing SUMMARY.md untouched (hand-written): $SUMMARY"
else
  GHIDRA_VER="$(grep -E '^application.version=' "$GHIDRA_HOME/Ghidra/application.properties" 2>/dev/null | cut -d= -f2 || echo unknown)"
  {
    echo "# Salvage analysis: ${TOOL_NAME}"
    echo
    echo "Hand-written summary. Replace the placeholders below."
    echo "Do not paste strings, import lists, symbol dumps, addresses, or decompiled code."
    echo "Raw exporter output is gitignored under \`tools/ghidra/local-out/${SAFE_NAME}/\`."
    echo
    echo "- **Binary:** \`${TOOL_NAME}\`"
    echo "- **Analyzed (UTC):** ${STAMP}"
    echo "- **Ghidra:** ${GHIDRA_VER}"
    echo
    echo "## Purpose"
    echo
    echo "_What the tool does, which screens or commands it has, which file formats or protocol roles it touches, and what Seihai could learn._"
    echo
    echo "## Placement"
    echo
    echo "_analysis-only / sibling repo / out-of-scope. See SALVAGE_PLACEMENT.md._"
    echo
    echo "## Program (counts only)"
    echo
    if [[ -f "$STATS" ]]; then
      echo "| Field | Value |"
      echo "| --- | --- |"
      echo "| Format | $(get_stat executable_format) |"
      echo "| Language | $(get_stat language) |"
      echo "| Compiler | $(get_stat compiler) |"
      echo "| Functions | $(get_stat function_count) (internal $(get_stat internal_function_count)) |"
      echo "| Import count | $(get_stat import_count) |"
      echo "| Export count | $(get_stat export_count) |"
      echo "| Interesting-string count | $(get_stat interesting_string_count) |"
      echo "| Decompiled locally | $(get_stat decompiled_count) — not committed |"
      echo
    else
      echo "_Ghidra export stats missing — headless may have failed. Read the log in the local-out folder. Do not paste it here._"
      echo
    fi
    if [[ $GHIDRA_RC -ne 0 ]]; then
      echo "Headless exit code: ${GHIDRA_RC}. See the local log; do not paste it into this file."
      echo
    fi
  } >"$SUMMARY"
  echo "==> Wrote summary stub (edit by hand before committing): $SUMMARY"
fi

echo "==> Raw artifacts: $RAW_DIR"
echo "==> Done (ghidra_rc=$GHIDRA_RC)"
exit "$GHIDRA_RC"
