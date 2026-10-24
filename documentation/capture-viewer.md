# Capture viewer and layout explorer

`seihai capture` is a small command-line tool for reading captures. It is the
tooling item of the roadmap: it shows the decoded frames and the field layouts
from the repository tables. It is not a GUI and it does not capture anything.
It reads files and prints text, tsv or json.

```text
seihai capture show FILE [options]        decoded messages of a capture
seihai capture layout ID|NAME [options]   one packet's layout across versions
seihai capture tables [--tables DIR]      the tables in use, and a check of them
```

## Input

`FILE` is one of two things:

* A **pcap** as the lab writes it (Ethernet, Linux cooked, raw IP or BSD
  loopback; IPv4 TCP). The viewer uses the difftest reader (`pcap.rs`), then
  the same codecs as `difftest trace` (gen-3 junk-byte cipher, gen-5 XOR with
  zlib or bz2, plain zlib), so what it decodes is what the client decodes.
* A **trace file** (`#seihai-trace v1`, what `difftest trace` writes). A trace
  already holds decoded and masked fields but no payload bytes, so there is no
  hex view for it. Everything else works the same.

The version comes from `--version`. Without it a pcap is decoded with every
version in turn and the one whose login token matches the capture's game
server login (or whose listserver token matches a listserver hello) wins.
When two versions tie, the first is used and the other is named in the header.
A trace file uses the `version` line of its header. If neither works the
command stops and asks for `--version`.

## Options of `show`

| Option | Meaning |
|--------|---------|
| `--version V` | Client version, as in `--version` of the client (`2.22`, `2.02 rev2`, ...) |
| `--tables DIR` | Use `packet-layouts.tsv` and `version-chain.tsv` from `DIR`. A file that is not there keeps the built-in table. |
| `--dir c2s\|s2c` | One direction |
| `--id N[,N..]`, `--name NAME` | Only these packets. A number is an id, a word is a name (case does not matter). `PLI_6` or `PLO_6` picks a direction too. Repeat or use commas. |
| `--conn NAME` | One connection: `gs0`, `gs1`, ... for game servers, `ls0`, ... for listservers, in order of first packet |
| `--time A..B` | Milliseconds since the connection's first packet. `A..`, `..B` and a single value work. |
| `--frame A..B` | Frame index within the connection and direction (the login frame is client frame 0) |
| `--limit N` | Print at most N messages. The header still says how many matched the capture in all. |
| `--layouts` | Print the layout from the tables under every message |
| `--raw` | Hex dump every payload (except login frames) |
| `--format text\|tsv\|json` | Output format |
| `--out FILE` | Write to a file (see below) |
| `--no-mask` | Do not mask the key, player ids, file times and account name |

## Output

Text, one block per message, in time order across both directions:

```text
# capture sample.pcap  version 2.31 (auto-detected from the login token)  mask on
# tables packet-layouts.tsv: built-in
# messages shown 10 of 10  (login frames redacted: password never decoded)
gs0 C>S f0 +2ms  -1 LOGIN  len 28
    type=0  key=<MASKED>  token=GNW28015  account=<ACCT>  password=<PASS>
gs0 C>S f1 +50ms  24 SHOWIMGPLAYER  len 1
    value=9  len=1
    layout: ?value:gchar
    ? `value` is unknown in the table
gs0 S>C f0 +60ms  6 LEVELNAME  len 7
    level=lv_a.nw
    layout: level:text
gs0 S>C f0 +60ms  120 PLO_120  len 5
    layout: (not in the tables for this version)
    ? no layout for this id and version in the tables
    raw 0000  01 02 61 62 63                                   |..abc|
gs0 S>C f1 +500ms  170 GHOSTMODE  len 3
    on=1  ?tail=4142
    layout: on:gchar
    ? 2 byte(s) after the last field in the layout
    raw 0000  21 41 42                                         |!AB|
```

(From a synthetic session; the `layout:` lines come from `--layouts`.)

The header line of each message is `connection direction frame time id name
length`. The next line holds the decoded fields.

**Where the fields come from.** The difftest decoder names the fields of the ids
it knows (props, warps, files and so on). For an id it leaves as hex or length
and checksum, the viewer reads the payload with the layout in
`packet-layouts.tsv` instead. If the table has no layout either, only the hex
dump is shown, with a note.

**Unknown stays unknown.** The viewer never fills a gap:

* `?name:enc` in a layout line, and `?name=` on a field, mean the packet has
  this field but the table says its meaning is unknown.
* `~name:enc` means the bytes of the field are known but what is in it is only
  partly decoded (for example an NPC prop list).
* `?tail=` holds the bytes left after the last field of the layout.
* A payload that ends inside a field is reported, not padded.
* An id with no row for the version says so.

**Raw hex.** An undecoded payload is dumped with offsets and an ASCII column
(16 bytes per line). A message that a table read completely gets no dump; one
with leftover bytes or a short payload does. `--raw` dumps every payload.

**tsv** has a `#seihai-capture-view v1` line, `#meta` lines and the columns
`conn dir t_ms frame id name len source fields layout unknown raw_hex`.
`source` is `decoder`, `table` or `raw`. Fields are `key=value` pairs joined by
`|` with the same percent escapes as a trace file.

**json** is one document: `capture`, `version`, `version_note`, `masked`,
`tables`, `total_messages`, `has_login` and a `messages` array. Each message has
`conn`, `dir`, `t_ms`, `frame`, `id`, `name`, `len`, `source`, `fields` (an
array of `{name, value}`), `layout`, `unknown` and `raw_hex`.

Text from the capture is written with control characters (including ESC) as
`\xNN` in the text format, so a hostile payload cannot drive the terminal.

## Redaction and private files

* The password is never decoded or printed, with or without `--no-mask`. It
  shows as `<PASS>`. The same goes for the listserver request that carries the
  account and password.
* The account name is masked as `<ACCT>` in login fields and in text, and
  player ids, the cipher key and file times are masked (`--no-mask` keeps them).
* The raw bytes of a login frame, and of any client message on a listserver
  connection, are never kept, so no dump, tsv column or json field can hold them.
* `--out FILE` creates the file with mode 600. When the capture contains a
  login, an existing `FILE` that is readable or writable by group or others
  (mode looser than 600) is **refused**, and the file is left as it was.
  Without `--out` the output goes to the terminal.
* Keep captures in a mode-700 folder and never commit or upload them. The
  repository tests use synthetic captures built in memory.

## `layout`: one packet across versions

```text
$ seihai capture layout 49
S>C 49  PLAYERWARP2
  profiles: added in 2.1
  1.33 .. 2.05   not handled
  2.1 .. 2.16    handled, no layout in the tables (fields undecoded)
  2.17 .. 2.31   handled, layout below  [layout 1]
  layout 1:
      1  x            halftile     known
      2  y            halftile     known
      3  z            gchar        known  (read, not used by the client)
      4  gmap_x       gchar        known  (used only when the name ends in .gmap)
      5  gmap_y       gchar        known
      6  level        text         known
    one line: x:halftile, y:halftile, z:gchar, gmap_x:gchar, gmap_y:gchar, level:text
```

The argument is an id (reported for each direction that has the id) or a name
(`PLAYERWARP2`, `plo_toall`). `--dir` limits it to one direction and
`--version V` to the stretch of versions that contains V.

Each stretch of consecutive versions with the same state is one line. The state
is whether the build's profile handles the id (`plo` for server packets, `pli`
for client packets: `protocol/profile.rs`) and which layout rows apply. The
`profiles:` lines are the steps where the id enters or leaves those sets. If a
`version-chain.tsv` is given with `--tables`, the rows of its `plo` and `pli`
steps that name the id are printed as `chain file:` lines as well.

A handled id with no layout row is printed as such. The viewer does not guess a
layout for it. PLO 101 and 102 (the board and the file) arrive as the raw
packet after `PLO_RAWDATA` (100), so they count as handled where 100 is.

## The tables

`documentation/packet-layouts.tsv` is the layout table. It is compiled into the
binary, and `--tables DIR` replaces it with `DIR/packet-layouts.tsv`. One row
per field, tab separated, `#` lines and blank lines ignored:

| Column | Meaning |
|--------|---------|
| `dir` | `C>S` or `S>C` |
| `id`, `name` | Packet id and name (the name agrees with the built-in names; `seihai capture tables` warns if not) |
| `idx` | Position of the field. Only the order matters. |
| `field` | Field name |
| `encoding` | `gchar`, `gshort`, `gint3`, `gint4`, `gint5`, `halftile` (GCHAR in half tiles), `gstring` (GCHAR length plus bytes), `gstring_list`, `text` (rest of the payload), `comma` (comma text), `props` (a prop stream, left to the decoder), `raw` (rest, bytes), `none` |
| `from`, `to` | First and last version label the row covers, in chain order. `-` leaves that end open. A row with both open applies to every build that has the id. |
| `status` | `known`, `partial` (bytes known, contents only partly decoded) or `unknown` (the field exists, meaning not known) |
| `note` | Free text |

A malformed row stops the tool with its line number. `seihai capture tables`
prints the row counts and warns about name mismatches, two fields at one
position, and a rest-of-payload field that is not last.

The rows come from what Seihai already decodes (`layouts.rs`, the trace decoder),
the Graal_analysis layout tables for 2.17, 2.22 and 2.31, the 2.05 notes and the
lab captures. Where the sources are for one version range only, the row is
limited to it, so a version outside the range shows "no layout in the tables",
not a copy of a neighbour. An id the table does not list has no layout. Adding a
row is the way to teach the viewer a layout; no code change is needed for the
encodings above.

`version-chain.tsv` (`step label token basis kind area field detail`) is optional.
Without it the viewer reads the handled-id sets from the built-in profiles.

## Checks

Tests build synthetic captures in memory and cover: version detection, names and
fields, every filter, the table reader and the tail and unknown markers, raw
dumps, tsv and json validity, login redaction (text, tsv, json, hex and the
listserver request), file mode handling for `--out`, `--tables` overrides,
the layout report and name resolution, bad rows, and that the built-in table
agrees with the profiles (the `modtime` field of `PLO_FILE` follows the
profile's flag, and no limited row names a version whose profile lacks the id).

To re-run the examples, write the synthetic session and point the tool at it:

```text
SEIHAI_VIEWER_SAMPLE_OUT=/tmp/sample.pcap \
  cargo test -p seihai --lib write_sample_pcap -- --ignored
seihai capture show /tmp/sample.pcap --layouts
```

The tool was also run read-only on the private lab pcaps of 2.17, 2.22 and 2.31
(the lab runs and the labd and labsw captures). The version was detected
correctly for all captures named after a version. Output was only counted, never
stored; see the pull request for the counts.

## Open

* No GUI, no live capture, no follow mode.
* Layouts exist only for the ids in the table (14 client packets, 47 server
  packets). Props streams are decoded by the difftest decoder, not by a table
  row. Packets with `raw` fields (NPC props, move data, shot data, the process
  list) are shown as bytes.
* The viewer decodes one version per run. A capture that mixes builds (a
  reconnect with another client) needs `--conn` and `--version`.
* Listserver replies show the server list row only; the other listserver
  messages are raw.
* `--time` and `--frame` count per connection, as the trace does.
