# Chat keys

* Closed chat line: **Enter** (the main key or the numeric pad's) or **T**
  opens it. The bar is always on screen; open, it shows `> text_`.
* Open: type, **Enter** sends, **Esc** cancels, Up/Down recall earlier lines
  (1.39 and later profiles). `T` is a letter while typing.
* Sending: a plain line goes out as the player's CURCHAT prop in
  `PLI_PLAYERPROPS` (prop 12, a GSTRING), the way the real client says
  something; it also shows in the chat log and as a bubble over the player
  for a few seconds. `/toall` text goes as `PLI_TOALL`; `/ignore`, `/buddy`
  and the like change the lists and are not sent.
* The client prints `chat: line opened (Enter)` and
  `chat: Enter pressed, N characters typed` on stderr, so a run log shows
  whether the key arrived.

Reported: Enter "does nothing" online on a public server with the 2.22
profile. The path was read end to end (key, bar, send) and has no gap that
would explain it, and it could not be reproduced here (no Wayland session).
Two things were changed in case the key never arrived as the main Enter: the
numeric pad's Enter now works, T opens the line too, and both the open and the
send are printed on stderr. If neither stderr line appears after Enter, the
key did not reach the program (window focus on the user's desktop); if the
second appears, the line was sent and the server's reaction is the thing to
look at.
