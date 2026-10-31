#!/usr/bin/env bash
# Headless salvage analysis wrapper for recovered seihai-servertools binaries.
# Usage: analyze_tool.sh <binary_path> <output_dir> [max_decomp_funcs]
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
Usage: $(basename "$0") <binary_path> <output_dir> [max_decomp_funcs]

Runs Ghidra analyzeHeadless with auto-analysis, then exports:
  functions.txt / functions.csv
  imports.txt / exports.txt
  strings_interesting.txt
  decompiled/*.c (+ INDEX.txt)
  SUMMARY.md
  prelim/ (file, strings, readelf/objdump/rizin snapshots)

Environment:
  GHIDRA_HOME              default: $SCRIPT_DIR/current
  JDK_HOME                 default: /workspace/tools/jdk/current
  GHIDRA_HEADLESS_MAXMEM   default: 4G
  GHIDRA_ANALYSIS_TIMEOUT  seconds per file (default 1200)
  GHIDRA_PROJECTS_DIR      default: $SCRIPT_DIR/projects
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" || $# -lt 2 ]]; then
  usage
  exit $([[ $# -lt 2 ]] && echo 2 || echo 0)
fi

BINARY="$(readlink -f "$1")"
OUT_DIR="$(mkdir -p "$2" && cd "$2" && pwd)"
MAX_DECOMP="${3:-40}"

if [[ ! -f "$BINARY" ]]; then
  echo "ERROR: binary not found: $BINARY" >&2
  exit 1
fi
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
SAFE_NAME="$(echo "$TOOL_NAME" | tr -c 'A-Za-z0-9._+-' '_')"
PROJECT_NAME="salvage_${SAFE_NAME}"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
PRELIM="$OUT_DIR/prelim"
LOG_DIR="$OUT_DIR/logs"
mkdir -p "$PRELIM" "$LOG_DIR" "$PROJECTS_DIR" "$OUT_DIR/decompiled"

echo "==> Salvage analysis"
echo "    binary : $BINARY"
echo "    output : $OUT_DIR"
echo "    ghidra : $GHIDRA_HOME"
echo "    java   : $($JAVA_HOME/bin/java -version 2>&1 | head -1)"

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
# interesting-ish filter for summary (paths, flags, urls, keywords)
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
  -postScript ExportSalvageArtifacts.java "$OUT_DIR" "$MAX_DECOMP" \
  -scriptlog "$SCRIPT_LOG" \
  -log "$HEADLESS_LOG"
GHIDRA_RC=$?
set -e

if [[ $GHIDRA_RC -ne 0 ]]; then
  echo "WARNING: analyzeHeadless exited with code $GHIDRA_RC (see $HEADLESS_LOG)" >&2
fi

# --- SUMMARY.md ---
SUMMARY="$OUT_DIR/SUMMARY.md"
STATS="$OUT_DIR/summary_stats.txt"

get_stat() {
  local key="$1"
  if [[ -f "$STATS" ]]; then
    grep -E "^${key}=" "$STATS" | head -1 | cut -d= -f2-
  fi
}

{
  echo "# Salvage analysis: ${TOOL_NAME}"
  echo
  echo "- **Binary:** \`$BINARY\`"
  echo "- **Analyzed (UTC):** ${STAMP}"
  echo "- **Ghidra:** $(grep -E '^application.version=' "$GHIDRA_HOME/Ghidra/application.properties" 2>/dev/null | cut -d= -f2 || echo unknown)"
  echo "- **Purpose focus:** recover CLI, protocols, config paths, and rebuild/rehost notes for seihai-servertools (ops/server tooling — not malware inventing)."
  echo

  echo "## Identity"
  echo
  echo '```'
  cat "$PRELIM/file.txt" 2>/dev/null || true
  echo '```'
  echo

  # Seihai binding rules: no FUN_/binary offsets/public hosts in committed docs.
  # Local artifacts under this out dir may still carry addresses; SUMMARY.md must not.
  scrub_host() {
    sed -E \
      -e 's#[Hh][Tt][Tt][Pp][Ss]?://[^[:space:]"]*[Gg][Rr][Aa][Aa][Ll][^[:space:]"]*#<public-url>#g' \
      -e 's#\b[[:alnum:]._-]*graalonline\.com\b#<public-host>#gI'
  }
  names_only_from_stat() {
    # "name@addr(sz=N) | ..." -> backtick-quoted unique names, drop FUN_/entry
    local raw="$1"
    [[ -z "$raw" ]] && return 0
    echo "$raw" | tr '|' '\n' | sed -E 's/@[0-9A-Fa-f]+.*//; s/^[[:space:]]+//; s/[[:space:]]+$//' \
      | grep -Ev '^(FUN_|FID_conflict:|entry$|thunk_)' | grep -E '^[A-Za-z_~]' \
      | awk '!seen[$0]++' | head -n 25 \
      | sed 's/.*/`&`/' | paste -sd ', ' -
  }

  if [[ -f "$STATS" ]]; then
    echo "## Ghidra program"
    echo
    echo "| Field | Value |"
    echo "| --- | --- |"
    echo "| Format | $(get_stat executable_format) |"
    echo "| Language | $(get_stat language) |"
    echo "| Compiler | $(get_stat compiler) |"
    echo "| Functions | $(get_stat function_count) (internal $(get_stat internal_function_count)) |"
    echo "| Imports | $(get_stat import_count) |"
    echo "| Exports / entries | $(get_stat export_count) |"
    echo "| Interesting strings | $(get_stat interesting_string_count) |"
    echo "| Decompiled funcs | $(get_stat decompiled_count) |"
    echo
  else
    echo "## Ghidra program"
    echo
    echo "_Ghidra export stats missing — headless may have failed. Check logs under \`logs/\`._"
    echo
  fi

  echo "## Notable symbols (names only)"
  echo
  _names="$(names_only_from_stat "$(get_stat key_functions)")"
  if [[ -z "$_names" ]]; then
    _names="$(names_only_from_stat "$(get_stat top_functions)")"
  fi
  if [[ -n "$_names" ]]; then
    echo "Observed application/helper symbols (no addresses): ${_names}."
    echo
  else
    echo "Most symbols were anonymous compiler stubs; prefer imports and strings for behaviour."
    echo
  fi
  echo "Full local function lists stay in untracked \`functions.txt\` / \`functions.csv\` under the analysis dir."
  echo

  echo "## Imports"
  echo
  if [[ -f "$OUT_DIR/imports.txt" && -s "$OUT_DIR/imports.txt" ]]; then
    echo '```'
    # Strip Ghidra external addresses / ordinals from committed SUMMARY lines.
    head -n 80 "$OUT_DIR/imports.txt" | sed -E 's/ @ EXTERNAL:[0-9A-Fa-f]+//; s/ @[0-9A-Fa-f]{4,}//g'
    echo '```'
    echo
    local_count=$(wc -l <"$OUT_DIR/imports.txt" | tr -d ' ')
    if [[ "$local_count" -gt 80 ]]; then
      echo "_… $((local_count - 80)) more in \`imports.txt\`_"
      echo
    fi
  else
    echo "_None exported (static binary, failed analysis, or no externals)._"
    echo
  fi

  echo "## Exports / entry points"
  echo
  if [[ -f "$OUT_DIR/exports.txt" && -s "$OUT_DIR/exports.txt" ]]; then
    echo '```'
    head -n 40 "$OUT_DIR/exports.txt" | sed -E 's/ @[0-9A-Fa-f]+ / /g; s/\bFUN_[0-9A-Fa-f]+\b//g'
    echo '```'
    echo
  else
    echo "_None listed._"
    echo
  fi

  echo "## Interesting strings"
  echo
  echo "Useful for CLI flags, config paths, URLs, protocol tokens."
  echo
  if [[ -f "$OUT_DIR/strings_interesting.txt" && -s "$OUT_DIR/strings_interesting.txt" ]]; then
    echo '```'
    head -n 60 "$OUT_DIR/strings_interesting.txt" | sed -E 's/^[0-9A-Fa-f]{4,}[[:space:]]+//' | scrub_host
    echo '```'
    echo
  elif [[ -f "$PRELIM/strings_interesting_raw.txt" && -s "$PRELIM/strings_interesting_raw.txt" ]]; then
    echo "_Ghidra string pass empty; raw \`strings\` filter:_"
    echo
    echo '```'
    head -n 60 "$PRELIM/strings_interesting_raw.txt" | scrub_host
    echo '```'
    echo
  else
    echo "_No interesting strings matched filters._"
    echo
  fi

  echo "## Decompile note"
  echo
  echo "Decompiled C may be produced locally for salvage reading only. Per Seihai binding rules, this repo does not commit decompiled source, anonymous decompiler names, or binary offsets. Describe findings as plain behaviour."
  echo
  if [[ -f "$OUT_DIR/decompiled/INDEX.txt" && -s "$OUT_DIR/decompiled/INDEX.txt" ]]; then
    echo "_Local \`decompiled/\` present (gitignored) — do not paste into MIT crates or this SUMMARY._"
    echo
  fi

  echo "## Rebuild / rehost checklist"
  echo
  echo "- [ ] Identify language/runtime from imports (libc, libstdc++, Go runtime, .NET, Python, JVM)."
  echo "- [ ] Extract CLI surface from \`usage\`/\`help\` strings and decompiled arg parsers."
  echo "- [ ] Note config file paths and env vars from interesting strings."
  echo "- [ ] Map network protocols (ports, URLs, RPC names) before rehosting."
  echo "- [ ] Prefer reimplementation from observed behavior over blind binary reuse when licensing/safety unclear."
  echo

  echo "## Artifacts"
  echo
  echo "- \`prelim/\` — file(1), strings, readelf/objdump/rizin/radare2 snapshots"
  echo "- \`functions.txt\`, \`functions.csv\`"
  echo "- \`imports.txt\`, \`exports.txt\`"
  echo "- \`strings_interesting.txt\`"
  echo "- \`decompiled/\`"
  echo "- \`logs/\` — analyzeHeadless + script logs"
  echo "- \`summary_stats.txt\` — machine-readable stats"
  echo
  if [[ $GHIDRA_RC -ne 0 ]]; then
    echo "## Status"
    echo
    echo "**analyzeHeadless exit code:** $GHIDRA_RC — review \`$HEADLESS_LOG\`."
    echo
  fi
} >"$SUMMARY"

echo "==> Wrote $SUMMARY"
echo "==> Done (ghidra_rc=$GHIDRA_RC)"
exit "$GHIDRA_RC"
