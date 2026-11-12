#!/usr/bin/env python3
"""Snapshot the Seihai suite's open issues and roadmaps into one Markdown file.

GitHub is the source of truth. This writes a local, read-only copy so an agent
(Cursor, Claude, Codex) or Squirrel can see the whole suite's open work at once,
offline, and diff it against the last snapshot.

Usage:
    python3 tools/handoff/tracker.py                 # print to stdout
    python3 tools/handoff/tracker.py -o TRACKER.md   # write a file
    python3 tools/handoff/tracker.py --check         # list roadmap items whose
                                                     # linked issue is closed but
                                                     # the box is still unticked

It uses `gh api` when the GitHub CLI is installed and signed in, otherwise the
GITHUB_TOKEN environment variable. It only reads; it never writes to GitHub.
"""
import argparse
import datetime as dt
import json
import os
import re
import shutil
import subprocess
import sys
import urllib.parse
import urllib.request

OWNER = "squirelworks"
# Keep in the order of docs/handoff/README.md, "The repos".
REPOS = [
    "seihai",
    "seihai-core",
    "seihai-gserver",
    "seihai-listserver",
    "seihai-npcserver",
    "seihai-rc",
    "seihai-servertools",
    "seihai-gserver-v2",
    "seihai-gs2lib",
    "graal-client-repro",
    "Graal_analysis",
]
ROADMAP_RE = re.compile(r"\broadmap\b|research track", re.I)


def api(path, params=None):
    """GET a GitHub REST path and return parsed JSON."""
    query = "?" + urllib.parse.urlencode(params) if params else ""
    if shutil.which("gh"):
        out = subprocess.run(
            ["gh", "api", "-X", "GET", path + query],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
        return json.loads(out)
    token = os.environ.get("GITHUB_TOKEN")
    if not token:
        sys.exit("Install and sign in to the GitHub CLI (gh), or set GITHUB_TOKEN.")
    req = urllib.request.Request(
        "https://api.github.com/" + path.lstrip("/") + query,
        headers={
            "Authorization": "Bearer " + token,
            "Accept": "application/vnd.github+json",
            "User-Agent": "seihai-tracker",
        },
    )
    with urllib.request.urlopen(req) as resp:
        return json.load(resp)


def search(query):
    """All issues matching a search query (paged, up to 1000)."""
    items, page = [], 1
    while True:
        data = api("search/issues", {"q": query, "per_page": 100, "page": page})
        items += data["items"]
        if len(items) >= data["total_count"] or not data["items"]:
            return items
        page += 1


def repo_of(issue):
    return issue["repository_url"].rsplit("/", 1)[-1]


def open_issues():
    q = "is:issue is:open " + " ".join(f"repo:{OWNER}/{r}" for r in REPOS)
    return search(q)


def render(issues):
    now = dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    by_repo = {r: [] for r in REPOS}
    for i in issues:
        by_repo.setdefault(repo_of(i), []).append(i)
    roadmaps = [i for i in issues if ROADMAP_RE.search(i["title"])]

    out = [
        "# Seihai suite tracker snapshot",
        "",
        f"Generated {now} by `tools/handoff/tracker.py` from GitHub. Do not edit by hand:",
        "change the issue on GitHub, then run the script again.",
        "",
        f"{len(issues)} open issues in {len(REPOS)} repos.",
        "",
        "## Roadmaps",
        "",
    ]
    for i in sorted(roadmaps, key=lambda i: (REPOS.index(repo_of(i)) if repo_of(i) in REPOS else 99, i["number"])):
        body = i.get("body") or ""
        done = len(re.findall(r"^\s*[-*] \[[xX]\]", body, re.M))
        todo = len(re.findall(r"^\s*[-*] \[ \]", body, re.M))
        subs = i.get("sub_issues_summary") or {}
        if subs.get("total"):
            done += subs.get("completed", 0)
            todo += subs["total"] - subs.get("completed", 0)
        out.append(f"- {repo_of(i)}#{i['number']} [{i['title']}]({i['html_url']}): {done} done, {todo} open")
    out.append("")
    for repo in REPOS:
        items = sorted(by_repo.get(repo, []), key=lambda i: i["number"])
        out += [f"## {repo} ({len(items)} open)", ""]
        for i in items:
            labels = ", ".join(l["name"] for l in i["labels"])
            tag = f" `{labels}`" if labels else ""
            out.append(f"- [#{i['number']}]({i['html_url']}) {i['title']}{tag}")
        if not items:
            out.append("- none")
        out.append("")
    return "\n".join(out)


REF_RE = re.compile(r"(?:squirelworks/)?([A-Za-z_][A-Za-z0-9_.-]*)? ?#(\d+)")


def state(repo, number):
    """"open", "closed", or "unknown" when the lookup fails (a wrong or moved ref)."""
    try:
        return api(f"repos/{OWNER}/{repo}/issues/{number}")["state"]
    except Exception:  # noqa: BLE001 - one bad ref must not stop the check
        return "unknown"


def check(issues):
    """Unticked roadmap lines whose every referenced issue or PR is closed or merged.

    A hint, not a verdict: a line can cite a merged PR as background ("see #217's
    notes") while its own work is still open. Read each line before ticking it.
    """
    stale = []
    for road in [i for i in issues if ROADMAP_RE.search(i["title"])]:
        home = repo_of(road)
        for line in (road.get("body") or "").splitlines():
            if not re.match(r"^\s*[-*] \[ \]", line):
                continue
            refs = [(r if r in REPOS else home, int(n)) for r, n in REF_RE.findall(line)]
            if not refs:
                continue
            states = [state(r, n) for r, n in refs]
            if all(s == "closed" for s in states):
                stale.append(f"{home}#{road['number']}: {line.strip()}")
    return stale


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("-o", "--output", help="write the snapshot here instead of stdout")
    ap.add_argument("--check", action="store_true", help="list roadmap boxes that look ready to tick")
    args = ap.parse_args()
    issues = open_issues()
    if args.check:
        stale = check(issues)
        print("\n".join(stale) if stale else "No unticked roadmap item has all its links closed.")
        return
    text = render(issues)
    if args.output:
        with open(args.output, "w", encoding="utf-8", newline="\n") as f:
            f.write(text + "\n")
        print(f"Wrote {args.output} ({len(issues)} open issues).")
    else:
        print(text)


if __name__ == "__main__":
    main()
