#!/usr/bin/env python3
"""Copy the Seihai suite Cursor rule into every sibling suite clone.

Usage: python3 tools/handoff/install_cursor_rules.py [FOLDER_WITH_CLONES]

FOLDER_WITH_CLONES defaults to this clone's parent folder. Each clone of a
suite repo found there (by its `origin` URL) gets `.cursor/rules/seihai-suite.mdc`,
and the path is added to its `.git/info/exclude` so it is never committed by
accident. Run it again after the rule changes.
"""
import pathlib
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parents[2]
RULE = HERE / ".cursor" / "rules" / "seihai-suite.mdc"
REL = ".cursor/rules/seihai-suite.mdc"
SUITE = {
    "seihai", "seihai-core", "seihai-gserver", "seihai-listserver",
    "seihai-npcserver", "seihai-rc", "seihai-servertools", "seihai-gserver-v2",
    "seihai-gs2lib", "graal-client-repro", "Graal_analysis",
}


def origin_repo(path):
    try:
        url = subprocess.run(
            ["git", "-C", str(path), "remote", "get-url", "origin"],
            check=True, capture_output=True, text=True,
        ).stdout.strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return None
    name = url.rstrip("/").rsplit("/", 1)[-1].removesuffix(".git")
    return name if "squirelworks" in url and name in SUITE else None


def main():
    root = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else HERE.parent).resolve()
    text = RULE.read_text(encoding="utf-8")
    done = 0
    for clone in sorted(p for p in root.iterdir() if (p / ".git").is_dir()):
        repo = origin_repo(clone)
        if not repo or clone.resolve() == HERE:
            continue
        target = clone / REL
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text, encoding="utf-8", newline="\n")
        exclude = clone / ".git" / "info" / "exclude"
        exclude.parent.mkdir(parents=True, exist_ok=True)
        lines = exclude.read_text(encoding="utf-8").splitlines() if exclude.exists() else []
        if REL not in lines:
            with exclude.open("a", encoding="utf-8", newline="\n") as f:
                f.write(("\n" if lines and lines[-1] else "") + REL + "\n")
        print(f"{repo}: {target}")
        done += 1
    print(f"Installed the rule in {done} clone(s) under {root}.")


if __name__ == "__main__":
    main()
