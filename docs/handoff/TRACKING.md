# Issues and roadmaps: how they are kept

GitHub issues are the one source of truth for open work. Each repo has one
roadmap issue; the umbrella roadmap is
[squirelworks/seihai#353](https://github.com/squirelworks/seihai/issues/353).
A local Markdown snapshot of all of it is generated, never hand-edited.

## The roadmaps

| Repo | Roadmap issue |
|---|---|
| suite (umbrella) | [seihai#353](https://github.com/squirelworks/seihai/issues/353) |
| seihai: Issho (6.x) | [seihai#440](https://github.com/squirelworks/seihai/issues/440) (sub-issues #441 to #448) |
| seihai: Ato (3.x to 5.x) | no roadmap issue yet; the plan is in [#380](https://github.com/squirelworks/seihai/issues/380), the lab run in [#400](https://github.com/squirelworks/seihai/issues/400), and `crates/seihai-ato/README.md` |
| seihai: Graal3D | `documentation/plannedmilestones.md` and the Graal3D part of #353 |
| seihai-core | [seihai-core#7](https://github.com/squirelworks/seihai-core/issues/7) |
| seihai-gserver | [seihai-gserver#56](https://github.com/squirelworks/seihai-gserver/issues/56) |
| seihai-listserver | [seihai-listserver#20](https://github.com/squirelworks/seihai-listserver/issues/20) |
| seihai-npcserver | [seihai-npcserver#22](https://github.com/squirelworks/seihai-npcserver/issues/22) |
| seihai-rc | [seihai-rc#27](https://github.com/squirelworks/seihai-rc/issues/27) |
| seihai-servertools | [seihai-servertools#50](https://github.com/squirelworks/seihai-servertools/issues/50) |
| seihai-gserver-v2 | [seihai-gserver-v2#35](https://github.com/squirelworks/seihai-gserver-v2/issues/35) |
| seihai-gs2lib | [seihai-gs2lib#11](https://github.com/squirelworks/seihai-gs2lib/issues/11) |
| graal-client-repro | [#174](https://github.com/squirelworks/graal-client-repro/issues/174) v0 to v2, [#175](https://github.com/squirelworks/graal-client-repro/issues/175) GraalX, [#176](https://github.com/squirelworks/graal-client-repro/issues/176) 39ster, [#178](https://github.com/squirelworks/graal-client-repro/issues/178) implementation |
| Graal_analysis | [Graal_analysis#370](https://github.com/squirelworks/Graal_analysis/issues/370) research track, [#191](https://github.com/squirelworks/Graal_analysis/issues/191) TRM 1.0 |

GitHub milestones are not used; roadmap issues with checkboxes or sub-issues
stand in for them.

## Labels

Use the labels a repo already has. The common ones:

- `lab`: needs a run on the lab machines with real clients.
- `research`: needs reading or capturing before code.
- `decision`: waits on Squirrel. Say what the choice is in the first lines.
- `on hold`: parked on purpose; the issue says what unblocks it.
- `fidelity`, `torque`, version labels (`0.0.5`, `2.22`): graal-client-repro areas.

## When you start a piece of work

1. Find or open its issue. Search first; don't open a duplicate.
2. If it is not on the repo's roadmap, add a line there that links the issue.
3. Comment on the issue with the branch name, so parallel agents see it is taken.

## When a piece of work finishes

Do all of these in the same sitting:

1. **The PR** carries `Fixes #n`, the test or lab record, and the repo's
   `docs/NEXT.md` update.
2. **On merge**, check the issue closed. If the PR only did part of it, leave
   it open and comment what is left.
3. **Tick the roadmap line** in the repo's roadmap issue, and in #353 if it is
   listed there, with the PR number (`- [x] ... (#123)`).
4. **Refresh the local snapshot** from a seihai clone:

   ```sh
   python3 tools/handoff/tracker.py -o ~/Claude/HANDOFF/TRACKER.md
   ```

   On Windows: `py tools\handoff\tracker.py -o %USERPROFILE%\Claude\HANDOFF\TRACKER.md`.
5. **Add one line** to `~/Claude/HANDOFF/LOG.md`: date, repo, PR, what changed,
   what is next.

To catch boxes that were missed, run `python3 tools/handoff/tracker.py --check`.
It lists unticked roadmap lines whose linked issues and PRs are all closed.
It is a hint: a line can cite a merged PR as background while its own work is
still open, so read each line before ticking it.

## The tracker script

`tools/handoff/tracker.py` only reads from GitHub. It uses the GitHub CLI
(`gh`) when it is installed and signed in, otherwise `GITHUB_TOKEN`. It lists
every roadmap with its done/open counts (checkboxes plus sub-issues), then
every open issue by repo. Add a repo to `REPOS` at the top when the suite
gains one.

## Cursor rules in every clone

`.cursor/rules/seihai-suite.mdc` in this repo is the suite rule for Cursor.
To use it in the other repos' clones without committing it there, run from a
seihai clone:

```sh
python3 tools/handoff/install_cursor_rules.py ..   # the folder that holds the clones
```

It copies the rule into each sibling suite clone's `.cursor/rules/` and adds
it to that clone's `.git/info/exclude`, so it never shows up as a change.
Repos with their own `AGENTS.md` keep it; Cursor reads both.
