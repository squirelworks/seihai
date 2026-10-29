# Chat keys

* Closed chat line: **Enter** (the main key or the numeric pad's) or **T**
  opens it. The bar is always on screen; open, it shows `> text_` with the full
  history window above it. Closed, the history is a fading overlay.
* Open: type, **Enter** sends, **Esc** cancels, Up/Down recall earlier lines
  (1.39 and later profiles). `T` is a letter while typing.
* Sending: a plain line goes out as the player's CURCHAT prop in
  `PLI_PLAYERPROPS` (prop 12, a GSTRING), the way the real client says
  something; it also shows in the chat log overlay and as a bubble over the
  player that stays until you move, then goes after 3 s; when it goes, an
  empty CURCHAT is sent so other screens take it down too (see `chat-log.md`). `/toall` text goes as `PLI_TOALL`; `/ignore`, `/buddy`
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

`unstick me` is not special to the client: it is sent as an ordinary line and
the server handles it. A `playerchats` script can rewrite the line before it
goes out, as in the real client. See `unstick-me.md`.

## Holding a key repeats it (laundry list P5, 2026-10-29)

The real clients took typed text from Windows (`WM_CHAR`), which repeats a held
key by itself. Seihai's window library passes on whatever the system sends: on
Windows and X11 that includes the repeats, but a Wayland window gets none (the
client must repeat by itself there), and a Mac with press-and-hold shows its
accent menu instead. So a held key typed one letter.

`ui::CharRepeat` now repeats the held character key in the chat line itself:
after 500 ms, then 30 times a second, until the key is let go or another key
goes down. When the system's own repeats arrive (the same character again with
no new key press), it adds none, so Windows and X11 do not get double letters.
Modifiers (Shift, Ctrl, Alt, Super, Caps Lock) are not counted as the key that
typed. The delay and the rate are Seihai's choice (common system defaults), not
read from a client.
