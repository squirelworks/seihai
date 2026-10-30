# Remaining fields: ids 24, 46 and 178

Overnight item 12. The three fields that earlier notes called unknown. Each
is settled here as far as the evidence goes, with what is still open and
which capture would settle it. Evidence is the 2.17.1, 2.22 and 2.31 client
binaries (read as code, 2026-10-25), the 2026-10-24 lab captures, and the
GServer-v2 and graal-serverlist sources. No lab run and no public server
were used for this item.

| id | verdict | in one line |
|---|---|---|
| 24 (client to server) and 32 (relay) | settled for 2.17.1 and later | the sender's weapon-script image list; the byte 9 is its "full list" marker with no images |
| 46 (client to server) | settled as far as a client can settle it | a content-free "I switched level" notice; the 1 is a literal |
| 178 (server to client) | the reading side is settled; the real service's wire text is not | comma text read by the clients' own list reader |

## Id 24: `PLI_SHOWIMGPLAYER`, relayed as `PLO_SHOWIMG` (32)

**What it is.** A text that lists the images the sender's weapon scripts show
(`showimg`, `changeimgpart`, `changeimgcolors`, `changeimgzoom`, `changeimgvis`,
`changeimgmode`). The sender keeps the list it last sent and sends only what
changed; every 101st call of its periodic handler it sends the full list instead,
led by the marker byte `0x29`. A client with no such images sends the marker
alone, which is the `GCHAR 9` that the lab saw from 2.22 and 2.31. The
receiver keeps the list per player and, after a marked (full) text, drops
every image of that player that the text did not name.

The format is in the header of `client/src/protocol/showimg.rs`:

```text
text    = [ 0x29 ] record*
record  = (0x2a + index) field*          index 0..=199
field   = (0x20 + tag) data
  tag 0  name        GCHAR length, then the name; an empty name removes the image
  tag 1  x           one GCHAR
  tag 2  y           one GCHAR
  tag 3  layer       one GCHAR, value - 1 (default 1; 10 and over = screen units)
  tag 4  part        0x20 = whole image, or 0x21, left and top (two 7-bit bytes
                     each), width and height (one byte each); clamped 0..16000
                     and 0..220
  tag 5  colours     four GCHARs, each over 200
  tag 6  zoom        one GCHAR, over 10
  tag 7  order       one byte, value - 0x52
  tag 8  mode        one GCHAR
```

A tag of `0x2a` or more starts the next record. A tag with no data byte ends
the text. An unknown tag skips its one data byte.

**Evidence.**

- Sender: the same routine is in 2.17.1, 2.22 and 2.31. It collects the
  images of the player's weapons (index 0 to 199), compares them with the
  list it sent last, and builds the text; at the counter wrap it starts the
  text with `)` (0x29) and sends every image in full. It sends nothing when
  the text is empty. This is why the lab saw only `GCHAR 9`, at the first
  tick after login and then every 5.1 s: no images, so the marker alone.
  The counter runs 100 down to 0 and wraps on the 101st call; the call after
  a send is skipped, so a wrap is 102 ticks. At 50 ms a tick that is 5.1 s,
  where the lab median was 5.13 s. (The 50 ms is inferred from the 5.1 s; the
  timer's interval was not read from the form resource.)
- Receiver: the 2.31 handler for id 32 reads the sender's id, finds that
  player in the active level, and hands the rest to the routine that applies
  the text: one branch per tag above, a marker flag that purges the
  unnamed images afterwards. Nothing else reads the byte.
- Server: GServer-v2 forwards the payload unchanged (`msgPLI_SHOWIMG`), so any
  bytes reach the other players.
- Why 2.22 sends only with another player present: the client sends the
  UDP copy (2.05's UDP id 15) when it can reach the players by UDP, and the
  TCP copy only when it is told UDP does not reach someone in the level; with
  nobody in the level the test is false. In 2.31 that test is a constant
  true, so the TCP copy always goes out. The wrap is not held back: a wrap
  that finds the test false is lost, the next one is a period later.
  `ShowImgSender` follows this now (it used to hold the clock while alone and
  send the moment someone arrived).
- 2.17.1 has the same sender as 2.22, and the binary reading gave it the
  same test. The 90 s solo 2.17 capture of 2026-10-25 saw no id 24, which
  that test predicts. The 2026-10-30 captures disagree: in all five solo runs,
  on both lab servers, 2.17.1 sent id 24 in its post-login frame and then
  every 5.07 to 5.11 s while idle (`difftest-217.md` item 3). Seihai follows
  the newer captures (`SHOWIMG_217`: 5.1 s, no other player needed). Why the
  two captures differ is not known.

**What Seihai does now.**

- `parse_showimg_update` and `apply_showimg_update` decode and apply the text;
  `SessionState::player_images` holds the result per player of the level
  (builds from 2.17 on; `showimg_records`). Nothing draws it.
- The capture viewer shows `full` and an `images` line for ids 24 and 32,
  and `packet-layouts.tsv` has a `showimg` encoding for them.
- The sender is unchanged in what it sends: the marker alone, only with
  `--send-showimg`, since Seihai has no weapon-script images to list.

**Open.**

- The sender's own field formatting was not decoded; the layout above is the
  receiver's reading. A capture of a real client whose weapon shows an image
  would confirm the units of x, y, colours and zoom. The lab cannot make one
  yet: the real client never ran NPC scripts in the lab (see
  `script-warps.md`).
- What the +0x124 field in the 2.22 test (the "UDP does reach this player"
  flag) is, and how Graal 2.22 decides it. Two real clients in the lab did
  not move it.
- The receiver applies a text only for a player in the client's active level;
  Seihai applies it for any player in its list.
- Drawing: layers, the screen-unit mode (layer 10 and over), colour and
  zoom. No renderer consumes the list.
- 2.05 and earlier send a four-field emote form (index, x, y, name) and
  relay it by UDP too. It is not decoded; their relays stay opaque.

## Id 46: `PLI_ENTERLEVEL`

**What it is.** A one-byte notice, `GCHAR 1`, that carries nothing. The 2.31
client sends it at the start of the routine that switches its current level,
whenever it is connected, before it does anything else. That fits the lab:
once at login, and again on every level change, including gmap edge
crossings, always just before the props frame that carries the new cell prop.
The 1 is a literal in the code. Nothing is read from the level, a player
count or the connection, so GServer-v2's guess that it "might be a player
count for the gmap level" is not what the client does.

**Evidence.** One send site in 2.31 and none in 2.17.1 or 2.22. GServer-v2
ignores the packet (its handler is a stub). The lab sent captured values only
as 1.

**What Seihai does now.** No change: it already sends `GCHAR 1` at login and
on each level change for profiles with id 46 (2.3 and later in the chain), in
that place. The table row is now `known` and limited to 2.3 and 2.31.

**Open.** What a server could do with it. Stock GServer-v2 does nothing. A
real service may use it; that cannot be told from a client capture. A lab
capture could show whether any server reaction follows, but the lab server
is GServer-v2 itself.

## Id 178: `PLO_SERVERWARP`

**The reading side (settled).** The 2.17.1, 2.22 and 2.31 handlers are the
same: build a list from the text with the client's own comma-text reader,
require at least 4 items, and call the "connect to another server" routine
with items 1, 2 and 3. Item 0 is read and not used.

The comma-text reader is the common Delphi one, and it matters because it
splits on white space:

- leading white space (bytes 1 to 0x20) is skipped before each item;
- an item that starts with `"` is quoted, ends at the closing `"`, and `""`
  is one quote; an unclosed quote runs to the end;
- any other item ends at a byte of 0x20 or less, or at `,`;
- after an item, white space, one comma and more white space are skipped;
  a comma at the end adds no empty item, `a,,b` keeps the empty one;
- a NUL ends the text.

So `x,Main Server,h,1` is five items (`Main` and `Server`), and the name must
be quoted when it has a space. Seihai now reads 178 this way
(`delphi_comma_text`); before it kept spaces inside an item.

Item 1 is stored as the server's name. The routine drops the first two
characters when the second is a space (the listserver's type prefix, as in
`P Name`), and 2.31 shows the result in the window title (as `Graal: Name`,
unless the name already contains "graal"). It does not use the name to
connect, which matches the lab (a made-up name connected normally). Seihai
logs the name without the prefix and puts it in the window title.

**The writing side (open).** Where the text comes from:

- GServer-v2 relays whatever the listserver answers to `PLI_SERVERWARP` (41)
  and only to clients from 2.1 on (`msgSVI_SERVERINFO`: "a hack to allow v5
  clients to serverwarp").
- The open-source listserver (graal-serverlist) answers with four items:
  the server name twice, the host and the port, each quoted by its
  tokenizer. An older, commented-out line in the same source shows a
  different shape: `playerworld<number>,"<name>",host,port`. Its item 0 is
  the "unused" one, which suggests the real service's item 0 is a tag of
  that kind. That is a reading of source comments, not an observation.
- What the real service sends has never been captured. Everything above
  would still parse with the item order Seihai uses, whatever the real
  service puts in item 0.

**What would settle it.** A capture of a 2.17-or-later client doing a
serverwarp on a real list service. That needs the public-server checks
(item 14) and a server that warps; nothing in this item contacted one.

**Open.** How the port text becomes a number in the clients (it is kept as
text first); Seihai accepts 1 to 65535. A host name instead of an address
and a port over 65535 were not tried in the lab.
