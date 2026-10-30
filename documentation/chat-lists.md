# Chat bubbles, buddy, ignore and mute

What Seihai does with chat bubbles and the three name lists, where each
rule comes from, and what is not known. Sources: the lab check of a real
2.22 client (`lab-checks-2026-10-24.md`, section 4, capture `c222-buddy.pcap`),
the laundry specs `10-right-click-menu.md` and `14-friends-ignore.md`, and the
2.01 vendor text in `decomp-findings-2026-10-24-c/09-features-classes-vocabulary.md`.

## What is observed, what is derived, what is a guess

| Rule | Status |
|---|---|
| Buddy add/delete writes `buddyaccounts.txt` locally and sends nothing. The file is deleted when it becomes empty | observed (real 2.22 client) |
| Ignore writes `ignoredaccounts.txt`, one account name per line, and sends nothing | observed (real 2.22 client) |
| Mute sends `PLI_MUTEPLAYER` (43): `GSHORT player id`, `GCHAR 1` (mute) or `GCHAR 0` (unmute), one message per click | observed (real 2.22 client, player id 2) |
| The player popup lists Message, History, Buddy, Ignore, Mute, Profile (2.22) and Message, History, Buddy, Ignore, Profile (2.05) | observed (2.22), derived (2.05) |
| Ignored players' chat and PMs are dropped | derived (spec 10 and 14) |
| Muted players' chat and PMs are dropped locally too | **guess**: the notes say "client-local mute set, same filter as ignore"; what a real client hides on mute has not been seen |
| Mute file `mutedaccounts.txt` | Seihai's own: no real client file for mute has been seen |
| Chat bubble stays 3 s | the 2.05 client counts 60 frames of 50 ms, **measured live** (3.0 s). Own typed line is held until a direction key; another player's line waits for their empty CURCHAT as well (`chat-log.md`) |

## Where it lives

- `client/src/chatlists.rs`: `NameList` (parse, add, remove, load, atomic
  save), `ChatLists` (the three files in the settings directory),
  `ChatFilter` (what is hidden), `parse_command` and `apply_command` (the
  chat-bar commands), `mute_message` (the id 43 bytes), `menu_items` (the
  popup order by profile) and `BubbleTimer`.
- `settings::config_dir()`: the directory `seihai.conf` is in
  (`~/.config/seihai/`). The list files live there.
- `SessionState` (`protocol/dispatch.rs`): reads the account name (prop 34)
  of other players, keeps who each player id was (`identities`, kept after
  the player leaves, so a PM from someone who has gone is still matched),
  drops hidden bubbles, toalls and PMs, and runs the bubble clock
  (`tick_bubbles`).
- `ui::player_list_rows`: the Tab list marks `[buddy]`, `[ignored]` and
  `[muted]`.
- `bin/main.rs`: loads the lists on each connection, runs the commands,
  sends the mute message, ticks the bubbles.

## Files

`buddyaccounts.txt`, `ignoredaccounts.txt`, `mutedaccounts.txt`: one
account name per line.

- Reading is tolerant: CRLF or LF, a byte order mark, blank lines, `#`
  comment lines, spaces around a name, a name repeated in another case,
  and lines with control characters (skipped). A file that is not UTF-8 is
  read as Latin-1. A missing or unreadable file is an empty list.
- Comparison ignores case. The case a name was first written in is kept.
- Saving writes `.<file>.tmp<pid>` in the same directory, syncs it and
  renames it over the old file, so a crash leaves the old file or the new
  one. An empty list deletes the file (as the real client does with the buddy
  file).
- The files hold account names and nothing else. Seihai never writes a
  password to them.

## Who is hidden

A chat line, bubble, toall or PM is dropped when its sender is in the
ignore list, or in the mute list on a build that sends id 43. The sender is
matched by account name when the server has sent it (prop 34), and by
nickname before that (and always on a build without account names). A
known account is not overridden by a nickname that equals an ignored name.

A mute file does nothing on a build whose profile has no id 43 (1.33 to
2.16): the commands answer "this version has no mute". Ignore and buddy
work on every build.

## Commands

`/ignore NAME`, `/unignore NAME`, `/buddy NAME`, `/unbuddy NAME`,
`/mute NAME`, `/unmute NAME`, `/lists [buddy|ignore|mute]`, typed in the
chat bar. Anything else, including `/` followed by an unknown word, is
chat. `NAME` can be an account name or a nickname; for a player Seihai has
seen, the account name is the one stored.

`/mute` and `/unmute` send id 43 only for a player who is in the level now,
on a build that sends 43. For a name that is not in the level only the list
changes. Seihai sends at most 5 mute messages per connection: the stock
GServer-v2 has no handler for 43, counts each as an invalid packet and
disconnects after more than 5 (lab notes). Past the cap the list still
changes and a line says so.

## Chat bubbles

Another player's line (CURCHAT, prop 12) shows for at least 3 s (60 frames) from
its arrival and until their empty CURCHAT comes (`BubbleTimer::start_floor`,
measured on the real 2.05, see `chat-log.md`). The history line stays. The text
is drawn as the version's own client draws it (`render::chattext`).
Our own typed line is held until a direction key is pressed, then timed the
same way (`chat-log.md`). The timer is `BubbleTimer`
(`chatlists::BUBBLE_MS`), driven by the game loop's frame time.

## Not done, not known

- The popup menu itself (right-click or F7 list with Message, History,
  Buddy, Ignore, Mute, Profile). `menu_items` has the order by profile and
  the commands do the actions; there is no mouse menu, no History window
  and no Profile request.
- A real client's buddy list through the listserver for 2.17 and later
  (`verifybuddies`, `addbuddy`, `deletebuddy`): never sent by the real 2.22
  client in the lab, so the token layout is unseen. Seihai sends nothing.
- A login notice when a buddy appears, and the status list (id 180).
- PMs from a player the client has never seen in a level: the PM carries
  only the sender's id and the layout of `PLO_ADDPLAYER` (55) is not in
  the notes, so such a sender can be matched only if an earlier prop
  update named them.
- What a real client hides on Mute, and whether it re-sends 43 after a
  relogin. Seihai sends it only on the command.
- The 2.05 client restarts the 60 when the server re-sends prop 12, as
  Seihai does. (The 3 s itself was measured live on 2026-10-30.)
- No lab run: the id 43 bytes and the file behaviour are taken from the
  2.22 capture and tested against it, not re-captured from Seihai.
