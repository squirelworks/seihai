# Lab captures: derived data

This folder holds the numbers, timings and packet orders taken out of raw lab captures, so the issues and READMEs that cite them no longer depend on lab branches.

The raw files (pcaps, screenshots, full gserver logs and packet dumps, harness scripts) are **not** in the repo. That follows the boundary set in #531 and #543. They are kept privately in the project's shared files under `lab-captures/seihai-lab-458-466/`, as a git bundle of both branch tips and a tarball of each tree.

| Folder | Lab | Source (deleted branch, tip commit) | Write-ups |
|---|---|---|---|
| [`lab458-graal3d/`](lab458-graal3d/FINDINGS.md) | #458: Ato 5.12 Graal3D link (runs 1 to 3), walk and chat (pass 2) | `claude/lab-458-graal3d` at `aab03f9` | [run 1/2](../lab/2026-11-13-lab458-graal3d/README.md), [run 3](../lab/2026-11-13-lab458-graal3d-run3/README.md), [pass 2](../lab/2026-11-14-lab458-pass2/README.md) |
| [`lab466-water-sound/`](lab466-water-sound/FINDINGS.md) | #466: Ato 3.153 water sound, `--no-sound`, water to dry level change | `claude/lab-466-water-sound` at `af24544` | [water sound](../lab/2026-11-13-lab466-water-sound/README.md), [level change](../lab/2026-11-14-lab466-level-change/README.md) |

`claude/lab-464-p19-trace` (#464) held decoded client scripts only. Its write-up is in [`docs/lab/2026-11-13-lab464-p19-trace/`](../lab/2026-11-13-lab464-p19-trace/README.md), and #543 recorded that the decoded dumps stay offline.

## What is in each table

Every `.tsv` is tab-separated with a header row. Host paths are replaced with `<lab>/file`. The lab account name `labato` is a throwaway loopback account. It is already in the write-ups.

`extract.py` rebuilds every table from the private trees:

```
python3 -I docs/captures/extract.py <lab-458 tree> <lab-466 tree> docs/captures
```
