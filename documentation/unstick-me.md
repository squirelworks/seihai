# "unstick me"

What happens when a player types `unstick me`, who handles it, how a world
turns it off, and what Seihai does. Everything below is **lab-captured**
(real 2.22 and 2.31 clients under Wine against the lab GServer-v2, on a
private loopback network, no public server), **decompile/strings-derived**,
or **read from the public GServer-v2 source** (summarized, not copied). Each
claim says which. Open items are listed at the end.

## Short answer

* The client does **nothing special** with the words. `unstick me` goes out
  like any other chat line: `CURCHAT` (prop 12) alone inside
  `PLI_PLAYERPROPS`. There is no separate message and no flag.
* The **server** recognises the text and warps the player.
* A **jail** is switched off in two ways: the server's `jaillevels` option
  (the server ignores the line), and a **client-side level script** that
  rewrites the line before it is sent (the server never sees it).
* Seihai already sent the identical bytes. It lacked one thing the real
  client does: a `playerchats` script's rewrite of the line (`setplayerprop
  #c,...`). That is fixed here.

## The client

**Strings (derived).** Every Graal client and tool under the lab's archive
(about 100 executables from 1.1 to 2.31, scanned for ASCII and UTF-16 text)
has no `unstick`, `unstuck` or `stuck` string. The typed-line words in the
2.05, 2.22 and 2.31 clients are three literals next to each other: `toall:`,
`showname`, `showap`; the script event name `playerchats` is the only other
chat-line string. So there is no client-side `unstick me` command.

**2.05 chat-box handler (derived from the decompile).** It trims the text;
`toall:` at the start sends a toall; the exact words `showname` and `showap`
are local commands; anything else becomes the player's chat text with the
bubble counter set, is offered to nearby players over the UDP side channel,
and then the `playerchats` script event fires. The handler itself does not
write the line to the TCP connection: the chat text rides in the player's
next props update, **after** the scripts had their say. (This is why a
client-side script can change what is sent. The 2.22 lab run below shows it
on the wire; the 2.05 order is read from the code, not captured.)

**On the wire (lab capture, real 2.22 and real 2.31).** Typing `hello
difftest`, `Unstick Me` and `unstick me` gives three client-to-server
messages of the same shape:

```
C>S PLAYERPROPS (2)  props=CURCHAT  CURCHAT=hello difftest
C>S PLAYERPROPS (2)  props=CURCHAT  CURCHAT=Unstick Me
C>S PLAYERPROPS (2)  props=CURCHAT  CURCHAT=unstick me
```

No other message accompanies them. The capitalisation is sent as typed.

## The server

**GServer-v2 (read from the public source; lab-confirmed).** When a
`CURCHAT` arrives the server splits the text into words. If the first word is
exactly `unstick` or `unstuck` (lower case) and the second is exactly `me`:

1. If the player's level is in `jaillevels`, nothing happens and the line is
   just chat. *(Lab: no warp, no reply.)*
2. If the player has not moved for at least `unstickmetime` seconds (default
   30), the server warps them to `unstickmelevel` at `unstickmex`,
   `unstickmey` (defaults `onlinestartlocal.nw`, 30, 30.5) and sets the
   player's chat to `Warped!`. *(Lab: `PLAYERWARP` x=30, y=30.5,
   `level=onlinestartlocal.nw`, then `CURCHAT=Warped!`.)*
3. Otherwise the player's chat becomes `Don't move for N seconds before
   doing '<what was typed>'!`. *(Source only, not run.)*

`Unstick Me` (capital letters) did **not** warp in the lab: the match is
case-sensitive. The same source warps a player to the same place when their
level cannot be loaded, and uses the unstick level as the fallback for a bad
login level.

The unstick options (`unstickmelevel`, `unstickmex`, `unstickmey`) are listed
as options a world may set in a playerworld guideline text found in a 2001
world, and many of the corpus worlds set them. `jaillevels` is in many option
files too.
`unstickmetime` appears only in GServer-v2's own default options; the
original closed server's handling of the words was not available to read.

## Switching it off in a jail

1. **Server:** the level is listed in `jaillevels`. The default options
   describe it as "players in these levels can't warp out nor can they PM
   other players". Lab-confirmed for 2.22 and 2.31: the client sent
   `CURCHAT=unstick me`, the server did not warp, and sent nothing back.
2. **Client-side script:** a level NPC (or weapon) with a client script that
   reacts to `playerchats` and rewrites the chat text, so the server only
   ever receives the new text. Real worlds do exactly this, for example (the
   pattern, not a copy of any world's file):

   ```
   if (playerchats && strequals(#c,unstick me)) {
     setplayerprop #c,Cant use unstick me while jailed!;
   }
   ```

   Lab-confirmed with the real 2.22 client and a level NPC of this shape:
   the **only** chat message on the wire was `CURCHAT=Cant use unstick me
   here!`; `unstick me` was never sent and the server did not warp.

Many corpus worlds also handle `unstick me` themselves in scripts (`setlevel2
unstick.nw,30,30` and similar), with their own cool-downs and level checks.
Those are world content, not engine behaviour.

## Seihai

* **Bytes:** identical. In the lab, Seihai (as 2.22) and the real client sent
  the same `CURCHAT` values for `hello difftest`, `Unstick Me`, `unstick me`
  (and in the jail level); the difftest report has no chat divergence. The
  server warped Seihai exactly as it warped the real client.
* **Changed:** the chat line now goes through the `playerchats` scripts first
  and the text a script leaves in the chat property is what is sent and
  shown (`SessionScripts::chat` returns it). Before, Seihai ran the scripts
  but sent the original words, so a jail NPC's rewrite did nothing and
  `unstick me` reached the server. With the change the lab run of the NPC
  scenario matches the real client: `CURCHAT=Cant use unstick me here!`, no
  warp.
* **Also fixed:** in a script, the first word of `setplayerprop` (`#c`, `#n`,
  ...) names the property and is no longer expanded as a message code.
  Before, `setplayerprop #c,hi` while a chat line was being handled tried to
  set a property called after the line that was said.
* **Offline:** the real client has no `unstick me` of its own, so Seihai does
  not warp offline either. Level scripts that handle the words run as for any
  other chat.
* Scenarios for the lab runner: `tools/difftest/scenarios/unstick.steps`,
  `unstick_jail.steps`, `unstick_npc.steps` (setup below).

## Reproducing

The runner is `tools/difftest/lab.sh` (see `difftest.md`). In the private
lab's server copy: keep `unstickmetime` at 30, add a copy of `lv_a.nw` named
`lv_jail.nw` and list it in `jaillevels`, and add `lv_j.nw` (a copy of
`lv_a.nw` plus one NPC with the script above). Then
`lab.sh run 2.22 unstick real`, `... seihai-parity`, and `lab.sh diff`.
The first scenario waits 34 s before the two unstick lines because of the
server's 30 s stand-still time.

## Open

* The 2.05 client was **not** run in the lab (the runner has no 2.05 login
  path); its behaviour is from the decompile. 2.22 and 2.31 are captured.
* Whether 2.22 trims the typed line before sending (2.05 does). Seihai sends
  the line as typed. A trailing or leading space may or may not matter to the
  server's word split; not tested.
* The original (closed) server's rules for the words, its idle time and its
  jail handling: only GServer-v2 was read and run.
* The "Don't move for N seconds" reply was read from the source, not captured.
* After the server's `Warped!` the real client sends an empty `CURCHAT` about
  3 s later (its own text timer ending); Seihai does not send that. It does
  not affect the unstick behaviour.
