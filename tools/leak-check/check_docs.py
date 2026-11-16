"""Leak check for `documentation/` (seihai#537).

Fails on:
* hex code/data addresses (`0x` + 5..8 digits) not on the allowlist,
* disassembler names (`FUN_` / `DAT_` / `LAB_` / `PTR_DAT_` / …),
* public IPv4 literals outside the allowlist (loopback, RFC 5737 docs,
  RFC 2544 benchmarking, and known client-version dotted quads).

The community listserver hostname is out of scope for this check.
Pure Python. Run from the repo root: `python3 tools/leak-check/check_docs.py`.
"""
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

# Protocol / mask constants that appear in docs (same spirit as
# Graal_analysis/tools/leak-check/check_addresses.py).
ALLOWED_HEX = {
    0x04A80B38,  # gen-3/5 cipher LCG seed
    0x08088405,  # Borland Random LCG multiplier
    0x8088405,  # same multiplier without leading zero
    0x7FFFFFFF,  # 31-bit mask
    0x00000000,  # zero dword
    0xFFFFFFFF,  # all-ones mask
}

HEX_RE = re.compile(r'(?<![0-9A-Za-z_])0[xX]([0-9a-fA-F]{5,8})(?![0-9A-Za-z_])')
NAME_RE = re.compile(
    r'\b(?:(?:FUN|DAT|LAB|PTR_DAT|PTR_FUN|thunk_FUN)_[0-9a-fA-F]{4,}|sub_[0-9a-fA-F]{6,8})\b'
)
IP_RE = re.compile(r'\b([0-9]{1,3}\.){3}[0-9]{1,3}\b')

# Client / build version strings that look like dotted quads in the docs.
# Keep this an explicit list so neutral public DNS (8.8.8.8, 1.1.1.1) still fail.
ALLOWED_VERSION_QUADS = {
    '0.0.0.0',
    '2.2.2.0',
    '2.3.1.1',
    '4.0.2.11',
    '6.0.0.7',
    '6.0.1.5',
    '6.0.3.4',
}


def ip_allowed(ip: str) -> bool:
    if ip in ALLOWED_VERSION_QUADS:
        return True
    parts = ip.split('.')
    if len(parts) != 4 or not all(p.isdigit() and 0 <= int(p) <= 255 for p in parts):
        return True  # not a real IPv4; leave alone
    a, b, c, d = (int(p) for p in parts)
    if a == 127:
        return True  # loopback
    if a == 192 and b == 0 and c == 2:
        return True  # RFC 5737 TEST-NET-1
    if a == 198 and b == 51 and c == 100:
        return True  # RFC 5737 TEST-NET-2
    if a == 203 and b == 0 and c == 113:
        return True  # RFC 5737 TEST-NET-3
    if a == 198 and 18 <= b <= 19:
        return True  # RFC 2544 benchmarking (fake resolver answers)
    return False


def line_errors(line: str) -> list[str]:
    errs: list[str] = []
    for m in HEX_RE.finditer(line):
        if int(m.group(1), 16) not in ALLOWED_HEX:
            errs.append(f'code address {m.group(0)}')
    for m in NAME_RE.finditer(line):
        errs.append(f'disassembler name {m.group(0)}')
    for m in IP_RE.finditer(line):
        ip = m.group(0)
        if not ip_allowed(ip):
            errs.append(f'public IPv4 {ip}')
    return errs


def repo_root() -> Path:
    here = Path(__file__).resolve().parent
    try:
        out = subprocess.run(
            ['git', 'rev-parse', '--show-toplevel'],
            cwd=here,
            capture_output=True,
            check=True,
            text=True,
        ).stdout.strip()
        return Path(out)
    except (OSError, subprocess.CalledProcessError):
        return here.parent.parent


def tracked_docs(root: Path) -> list[Path]:
    try:
        out = subprocess.run(
            ['git', 'ls-files', '-z', 'documentation'],
            cwd=root,
            capture_output=True,
            check=True,
        ).stdout
        names = [n for n in out.decode('utf-8', 'surrogateescape').split('\0') if n]
    except (OSError, subprocess.CalledProcessError):
        names = [p.as_posix() for p in (root / 'documentation').rglob('*') if p.is_file()]
    return [root / n for n in names if Path(n).suffix.lower() in {'.md', '.txt', '.tsv', '.csv'}]


def scan(root: Path) -> list[str]:
    problems: list[str] = []
    for path in tracked_docs(root):
        if not path.is_file():
            continue
        text = path.read_bytes().decode('utf-8', 'replace')
        rel = path.relative_to(root).as_posix()
        for no, line in enumerate(text.splitlines(), 1):
            for err in line_errors(line):
                problems.append(f'{rel}:{no}: {err}')
    return problems


def main(argv: list[str]) -> int:
    root = Path(argv[1]).resolve() if len(argv) > 1 else repo_root()
    problems = scan(root)
    for p in problems:
        print(p)
    if problems:
        print(f'documentation leak check: {len(problems)} hit(s)', file=sys.stderr)
        return 1
    print('documentation leak check: clean')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
