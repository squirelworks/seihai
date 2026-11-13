# Issho: 6.x window, status display and chat (2026-11-13)

A check of the change for #442, #444 and the chat part of #445, in a headless container (Xvfb, Mesa llvmpipe). No server and no real client: this shows Seihai's drawing only. The real-client measurements it follows are in `issho-6x-lab-run-2026-11-12.md` and on #444.

**Build:** `seihai-issho` from branch `claude/project-thread-51lv7j`, debug, against seihai-core with squirelworks/seihai-core#40.

**Run:** `seihai-issho --version 6.03r7 --level new1.nw --assets assets/default --no-sound` under `xvfb-run`, then Tab, typing a line and Enter with `xdotool`.

| Check | Result |
|---|---|
| Window size | 1024 x 768, the level fills it |
| Status display | three slots at x 15, 80, 145; sword icon beside the second; AP bar at (15, 65); gralat, bomb and arrow icons from x 274 with `0`, `05`, `10`; three hearts from x 826, 45 px apart, as the real 6.03r7 drew them |
| Status line and position | at the foot of the window, out of the status display's way |
| Tab | opens the chat line at the foot; the arrows stop walking while it is open |
| Enter | closes the line; the text shows over the player as a bubble for 3 s |

Not checked here: the bubble and chat lines going over the wire to a server and another client. That needs the lab server stack, so it waits for the #441 run.
