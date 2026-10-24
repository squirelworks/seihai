# Handoff for Grok: public-server checks (blocked), written by Chiaki, 2026-10-25 JST

## Where things stand
- The overnight queue (items 1-14) is done. Item 15 (differential-testing divergences) was still running when this was written; its PRs will appear as chiaki/fix-* branches.
- Merged into Seihai main by the maintainer: #27-#36, #38, #39 (item 12 ids 24/46/178 plus the item-14 docs). Open: #40 (.zelda Z3-V1 reader). Graal_analysis: #30-#41 merged; #42 (Z3-V1 notes) open.
- Item 13 (early builds / a 2.00-era listserver reply) was skipped only because no real reply has ever been observed. It depends on the public checks below.

## Why the public checks did not run
Chiaki's computer cannot reach the public listserver. TCP connects to listserver.graal.in on 14921, 14922 and 14923 all time out with no reply (6 attempts on 2026-10-25 03:06-03:09 JST, plus 2 more at 03:49). The host name resolves to a private-range address (198.18.0.1), not the real host. Ordinary HTTPS (443) works. This looks like the box's network policy, not the listserver and not Seihai. No protocol bytes were sent, no login, no chat. The password was never used.
User confirmed (t113u): port 14923 is the legacy support port on listserver.graal.in. This resolves the earlier open question for pre-2.17 clients (14921 vs 14923): 14923 is the one to try for them; 14922 serves 2.17 and later.

## Approval the user gave
Read-only checks first, then a log-in to one public game server with the Seji account, at most about five ordinary chat lines, one or two level warps through normal links, the normal WANTFILE/UPDATEFILE flow, and following a server warp if one arrives. No uploads, no account changes, no scanning, nothing abnormal. Stop at any ban, rate limit, captcha or error that looks like a stop sign.

## To run, from a machine that can reach the ports (the user's own machine)
Keep the password in an environment variable, never on a command line. Account name Seji.
1. Capture only the target traffic to a private folder (dir mode 700, files 600). The pcaps are never committed or uploaded.
2. seihai --online listserver.graal.in:14922 --account Seji --password-env VAR --version 2.22 --list-only, then 2.17 and 2.31. For a pre-2.17 profile (2.00/2.05) use port 14923, one connection each.
3. seihai capture show FILE --version V (login frames are redacted, the account is masked).
4. One game-server login per profile, a few minutes at most.
5. Hand back the capture (or just the --list-only text) to Chiaki to decode.
Full checklist: documentation/public-checks.md in squirelworks/seihai.

## What each result would settle
| Question | Settled by |
|---|---|
| Real server-list text: item 0, names with spaces, "X " prefixes; id 178 field order | the --list-only text, or a serverwarp seen on the wire |
| Which port answers for clients older than 2.17 | 14923 vs 14921 on 2.00/2.05 profiles |
| What 2.17/2.22/2.31 logins get from the public service | one login per profile |
| Whether the tamper value (id 39) is enforced | the login frames and server reaction |
| A real 2.00-era listserver reply (item 13) | the 2.00 profile on 14923 |
| Chat, warps, file requests on a public server | the game-server session |
