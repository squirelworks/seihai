# Chat log overlay and chat bubble timing

Question: does the real client have an on-screen chat log that fades away?

**Answer: no.** Neither the 2.05 client (read from a decompile) nor the 2.22
notes and vendor texts have a chat feed on the screen, and nothing about chat
fades. What the real client draws is a line of text over the speaker's head
that disappears after a while. Seihai's fading overlay is therefore **a
Seihai feature, requested by the user, with assumed timings**, kept apart
from the bubble, which follows the evidence.

## What was found (2.05 client, derived from the decompile)

| Fact | Status |
|---|---|
| Chat is drawn by the same routine that draws nicknames and the HUD numbers: plain text, word-wrapped on spaces at the view width, stacked upward from the head, no balloon, no alpha | derived |
| Callers of that routine are the players (own and others), the computer, HUD numbers and the editor NPC. No caller draws a screen-space feed | derived |
| Chat text is white, with a one-pixel (+1, +1) shadow in a contrast colour: blue (0, 0, 255) for white text. The font is per build (Dialog 28, Helvetica 24, Tempus Sans ITC 24; see `render::chattext` and Graal_analysis `client-internals/chat-text.md`) | derived, and **seen live** on 2.05 rev 2 (2026-10-30) |
| Another player's chat line (prop 12 in `PLO_OTHERPLPROPS`) shows for at least 60 frames from its arrival **and** until an *empty* prop 12 comes. An empty line that comes after the 60 frames clears the text at once; one that comes earlier lets the rest of the 60 run. With no empty line the text stays. (The earlier reading here, "an empty line starts 60 more frames", was wrong.) (Your own prop 12 sent by the server, in `PLO_PLAYERPROPS`, sets 60 directly.) | **live**, 2.05 rev 2 under Wine, 2026-10-30: empty after 20 s cleared it in under 0.15 s; empty after 0.54 s and after 2.0 s both cleared it 3.0 to 3.2 s after the text |
| The counter drops by one per drawn frame; at 0 the text is cleared. No fade, the text just goes | derived |
| Your own typed line (Enter in the hidden edit box) sets the counter to **61**, one above the range that counts down, so it does **not** count down until one of the four direction keys is pressed; then it becomes 60 and runs out. When it runs out with text up, the client sends an empty CURCHAT, which is what ends the line on everyone else's screen | derived, and **live**: up for 7+ minutes standing still, gone 2.9 to 3.1 s after a key, with the empty CURCHAT sent then |
| The text is trimmed; `toall:` at the start sends a toall instead; exactly `showname` and `showap` are commands (they start a 60-frame name or ap display) | derived |
| Typed lines are kept in a 100-entry history (Up/Down recall) | derived, and in the 1.41 vendor text |
| A PM or toall is appended to a per-player message history with a time stamp, and a message window is opened or flashed. It does not go to an overlay | derived |
| 60 frames = 3 s (50 ms frame) | the vendor text says chat text "is always displayed at least 3 seconds" (2.01 and later); **confirmed live** on 2.05 rev 2 (3.0 s) |
| Toall bubbles keep up to 50 messages per player and message windows have a history of the last 50 | vendor text |
| The Alt+0 "log window" (2.01 and later) shows system events such as DirectX start, login/logout and explosions, **not chat** | vendor text |

So for the real client: no log lines, no line count, no fade steps, no push-up.
Those do not exist and are not invented as "real" anywhere in Seihai.

## What Seihai does

### Chat bubble (evidence-backed)

* `chatlists::BUBBLE_MS` is 60 frames x 50 ms = 3000 ms.
* **Your own typed line is held**: it stays over your head until you press a
  direction key (with the chat line closed), then goes down after the same
  3 s (`BubbleTimer::start_held`, `release`). When it goes, Seihai sends an
  empty CURCHAT, as 2.05 does. Before this, Seihai did not, so real clients
  kept showing a Seihai player's line.
* **Another player's line is held too**, until their client sends the empty
  CURCHAT; then it shows 3 s more and goes. Before this, Seihai took it down
  after 3 s whatever the speaker did.
* Not done: your own prop 12 coming back from the server (`PLO_PLAYERPROPS`)
  is not shown as your bubble.

### Chat overlay (Seihai's own, assumed timings)

`ui::draw_chat_overlay` replaces the permanently shaded 6-line window.

* The input bar is always shown at the bottom.
* The history appears above it as an overlay of the newest 6 lines
  (`CHAT_LINES`) that have not faded. A new line is drawn lowest and pushes
  the older ones up.
* Everything that goes to the history goes to the overlay: other players'
  chat, your own typed lines, toalls, PMs, and Seihai's own notes
  (`ChatKind` colours as before).
* A line is fully visible for `OVERLAY_VISIBLE_MS` = **8000 ms**, then fades
  in `OVERLAY_FADE_STEPS` = **4** equal alpha steps over `OVERLAY_FADE_MS` =
  **2000 ms** (opacity 204, 153, 102, 51 of 255; the line's shade fades with
  it), then is gone. Older lines expire first.
* While the chat line is open, or the history is scrolled back with PageUp,
  the full window is drawn as before and **nothing fades**, so older lines
  (the last 200 are kept) can be read.
* Line age is measured on a chat clock advanced by the frame time
  (`SessionState::chat_clock_ms`).

**8 s, 2 s and 4 steps are assumptions** (the 8 s is a readable-time guess;
the real client has no equivalent). Change the three constants in `ui.rs` to
retune; the tests derive from them.

### Typing a line (the "nothing visible" report)

Enter sends through `ui::own_say`, which does three things: pushes the line to
the history (so it is in the overlay), makes it your own bubble, and starts
the held bubble clock. A test covers all three. The earlier report that Enter
"did nothing" was never reproduced; one new explanation that fits the old
behaviour is that the line was visible only as a 3 s bubble that ran out while
standing still. It now stays until you move, and the line is in the overlay
for 8 s. This is a guess about the cause, not a confirmed one.

## Settled, open, assumed

* **Settled (derived from the 2.05 client):** no on-screen log, no fade; chat
  is a white outlined head label; every line held (own until a direction
  key, others until their empty CURCHAT), then 60 frames; PM/toall go to message windows with a history.
* **Assumed:** 50 ms frame (so 3 s); the overlay's 6 lines, 8 s, 2 s, 4 steps.
* **Open:** the shadow colour; the layout of the real message window and
  whether a real PM also flashes something on the main screen; whether newer
  clients (2.22) keep the 60-frame rule or the held own line (the 2.22 notes
  only say "at least 3 seconds"); whether the real chat label is clipped or
  wrapped the same in 2.22; the cause of the user's "no visible result".
