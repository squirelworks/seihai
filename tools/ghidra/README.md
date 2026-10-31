# Ghidra headless salvage pipeline

Recover and understand ops/server binaries (seihai-servertools) with Ghidra headless analysis. Treat inputs as recovered tooling — focus on purpose, CLI, protocols, config paths, and how to rebuild or rehost them.

## Layout

| Path | Role |
| --- | --- |
| `current` → `ghidra_12.2_PUBLIC/` | Installed Ghidra release (gitignored) |
| `ghidra_scripts/ExportSalvageArtifacts.java` | Post-analysis export script |
| `analyze_tool.sh` | One-shot wrapper: prelim RE + headless + `SUMMARY.md` |
| `projects/` | Scratch Ghidra projects (deleted per run by default) |
| `/workspace/tools/jdk/current` | Temurin JDK 25 (required by Ghidra 12.2) |
| `/workspace/tools/rizin/bin` | Optional rizin static install for prelim snapshots |

Analysis outputs for recovered Drive binaries go under:

`/workspace/recovered-drive-tools/ghidra-analysis/<toolname>/`

## Versions

- **Ghidra:** 12.2 PUBLIC (`20261016`)
- **JDK:** Temurin 25.0.5+8 (Ghidra 12.2 requires Java 25+)
- **Helpers:** `file`, `strings`, `binutils` (`objdump`/`readelf`/`nm`), `radare2`/`r2`, `rizin`/`rz-bin`, `xxd`, `python3`

## Install / refresh Ghidra

If `current/` is missing (fresh clone):

```bash
mkdir -p /workspace/tools/ghidra /tmp/ghidra-dl
cd /tmp/ghidra-dl
curl -L -o ghidra.zip \
  "https://github.com/NationalSecurityAgency/ghidra/releases/download/Ghidra_12.2_build/ghidra_12.2_PUBLIC_20261016.zip"
unzip -q ghidra.zip
mv ghidra_12.2_PUBLIC /workspace/tools/ghidra/
ln -sfn ghidra_12.2_PUBLIC /workspace/tools/ghidra/current

# JDK 25
mkdir -p /workspace/tools/jdk /tmp/jdk-dl && cd /tmp/jdk-dl
curl -L -o jdk25.tar.gz \
  "https://github.com/adoptium/temurin25-binaries/releases/download/jdk-25.0.5%2B8/OpenJDK25U-jdk_x64_linux_hotspot_25.0.5_8.tar.gz"
tar -xzf jdk25.tar.gz -C /workspace/tools/jdk
ln -sfn jdk-25.0.5+8 /workspace/tools/jdk/current

# Point Ghidra at JDK 25
sed -i 's|^JAVA_HOME_OVERRIDE=.*|JAVA_HOME_OVERRIDE=/workspace/tools/jdk/jdk-25.0.5+8|' \
  /workspace/tools/ghidra/ghidra_12.2_PUBLIC/support/launch.properties
```

Verify:

```bash
export JAVA_HOME=/workspace/tools/jdk/current
/workspace/tools/ghidra/current/support/analyzeHeadless 2>&1 | head
```

## Analyze one binary

```bash
/workspace/tools/ghidra/analyze_tool.sh \
  /path/to/binary \
  /workspace/recovered-drive-tools/ghidra-analysis/toolname
```

Optional third arg: max functions to decompile (default `40`).

```bash
/workspace/tools/ghidra/analyze_tool.sh ./gscripts-old \
  /workspace/recovered-drive-tools/ghidra-analysis/gscripts-old 60
```

### Output tree

```
ghidra-analysis/<tool>/
  SUMMARY.md                 # human overview
  summary_stats.txt          # key=value stats
  functions.txt|.csv
  imports.txt
  exports.txt
  strings_interesting.txt
  decompiled/
    INDEX.txt
    <func>_<addr>.c
  prelim/                    # file/strings/readelf/rizin/radare2
  logs/                      # analyzeHeadless + script logs
```

## Batch from recovered Drive inventory

When a sibling recovery agent populates `/workspace/recovered-drive-tools/` (see `INVENTORY.md`):

```bash
INV=/workspace/recovered-drive-tools/INVENTORY.md
OUT=/workspace/recovered-drive-tools/ghidra-analysis
# Prefer ELF/Mach-O/PE executables and known servertool names first
find /workspace/recovered-drive-tools -type f \( -perm -111 -o -name '*.exe' -o -name '*.so' \) \
  ! -path '*/ghidra-analysis/*' -print
```

Then run `analyze_tool.sh` per high-value candidate into `$OUT/<name>/`.

## Raw analyzeHeadless

```bash
export JAVA_HOME=/workspace/tools/jdk/current
export GHIDRA_HEADLESS_MAXMEM=4G
GHIDRA=/workspace/tools/ghidra/current
$GHIDRA/support/analyzeHeadless \
  /workspace/tools/ghidra/projects myproj \
  -import /path/to/bin \
  -overwrite -deleteProject \
  -scriptPath /workspace/tools/ghidra/ghidra_scripts \
  -postScript ExportSalvageArtifacts.java /tmp/out 40
```

## Notes

- Headless deletes the scratch project by default (`-deleteProject`) so reruns stay clean.
- Decompilation targets entry points, name-matched helpers (`main`, `usage`, `config`, `listen`, …), then large/hot functions.
- Interesting-string filters bias toward paths, CLI flags, URLs, config extensions, and seihai/gserver-related tokens.
- Do not invent malware or exploit payloads; salvage for rebuild/rehost of legitimate server/ops tools only.
