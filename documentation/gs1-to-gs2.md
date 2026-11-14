# GS1 to GS2 lowering: what is lowered and what is not

Issue #371. `seihai_core::scripting::gs2::compile::compile_gs1` parses GS1
with the engine grammar and emits GS2 bytecode. This page lists the
constructs it lowers and the ones it rejects. Anything not on the first list
is a compile error; nothing is dropped silently.

"Fixture" below means an op-for-op comparison with bytecode in
`crates/seihai-core/tests/fixtures/gs2/` (see the README there). A construct
without a fixture has a Seihai test only. No original client has run any of
this output, so this list is not a compatibility claim for any client.

## Lowered

| GS1 | GS2 form | Checked against |
|---|---|---|
| `if (created)`, `if (playerenters)` and the other events | `function onCreated()` and so on | `shared.gs2bc`, name tests |
| `if (ev1 \|\| ev2)` | one function per event | function-name test |
| `if (ev && guard)` | the event's function with `if (guard)` inside | function-name test |
| `function name(a, b)` | same | `calls.gs2bc` prefix |
| `if` / `else`, `while`, `for`, `break`, `continue`, `return` | same | op-shape tests (GS2 side only) |
| `x = v`, `x += v`, `-=`, `*=`, `/=`, `x++`, `x--` on a plain variable | same | `shared.gs2bc` (`=`), op-shape tests |
| arithmetic, comparisons, `&&`, `\|\|`, `!` | same | `shared.gs2bc`, logic-chain op test |
| `@`, `SPC`, `NL`, `TAB` | `OP_JOIN` | runs on Seihai's GS2 machine |
| `c ? a : b` | same | runs on Seihai's GS2 machine |
| `x in \|lo, hi\|` | `OP_IN_RANGE` | runs on Seihai's GS2 machine |
| `{a, b, c}` (2.1 and later), where the element order cannot matter (see below) | array literal | `lists.gs2bc`; with calls, same bytes as the GS2 source |
| `a[i]`, `a[i, j]` read | `OP_ARRAY`, `OP_ARRAY_MULTIDIM` | `lists.gs2bc` |
| `a[i] = v`, `a[i, j] = v` | `OP_ARRAY_ASSIGN`, `OP_ARRAY_MULTIDIM_ASSIGN` | `lists.gs2bc` |
| `x in this.list` (2.1 and later) | `OP_IN_OBJ` | `lists.gs2bc` |
| `a[i] += v`, `-=`, `*=`, `/=`, `a[i]++`, `a[i]--` (and with `a[i, j]`) | the bytes of `a[i] = a[i] + v` | element read and write from `lists.gs2bc`; same bytes as the long form; runs on Seihai's GS2 machine |
| `x in {a, b, c}` | the bytes of `x == a \|\| x == b \|\| x == c`; `in {}` is 0 | same bytes as the long form; runs on Seihai's GS2 machine |
| `playerx`, `playery` | `player.x`, `player.y` | same bytes as the GS2 source |
| `with (target) body` (2.1 and later), with the limits below | target, `OP_CONV_TO_OBJECT`, `OP_WITH`, body, `OP_WITHEND` | `rest.gs2bc` (`with (player)`, `with (null)`), both front ends |
| `sleep`, `setarray`, `sin`, `cos`, `arctan`, `abs`, `random`, `int`, `exp`, `log`, `min`, `max`, `pow`, `char`, `getangle`, `getdir`, `vecx`, `vecy`, `waitfor`, `format`, `makevar`, `arraylen`, `sarraylen` | their opcodes, with Seihai's argument-count check | `gs2-opcodes.md`, argument tests |
| `strlen(s)`, `startswith(s, t)`, `indexof(s, t)` | `OP_OBJ_LENGTH`, `OP_OBJ_STARTS`, `OP_OBJ_POS` | `gs2-opcodes.md` only |
| a GS1 command that GS2 also has | a call by name | `shared.gs2bc` (`showimg`, `hideimg`, `set`) |

The compound element forms and `in {list}` are rewritten before lowering,
to what the GS1 interpreter does: it reads the element (or `x`) once and
stores once, and it compares `x` with each item left to right with its
`==`, stopping at the first match. The rewrite pushes the index (or `x`)
more than once, so it is refused when that expression calls a function;
put it in a variable first. The GS2 front end (`compile_gs2`) still
refuses these forms: gs2-parser's own encoding of them has no fixture.

A computed array index (`a[this.i + 1]`) is converted with
`OP_CONV_TO_FLOAT` like any numeric operand. The fixture only has constant
indexes, so that conversion is Seihai's choice.

## `with`

No script in the client archives (1.x to 2.x, the NW betas) or the NPC
library uses `with`. It appears only in the NPC server's notes
(`NPCControl/npcserverdoc.txt` in the NW beta 1 and 2 archives), so the
lowering follows those notes and the fixture's encoding. The notes say that
inside `with (players[i])` or `with (npcs[i])` the target's properties and
`this.` variables are used, plain script variables (`i`, `j`) still work,
script functions still run as the original NPC, the block is skipped when
the target does not exist, and `sleep`, `break` and `continue` inside the
block leave the old object switched.

In Seihai's GS2 machine a plain name resolves on the `with` object first and
`player` stays the script's player; no fixture shows how the client runs a
bare call inside `with`. So a GS1 body is lowered only when its meaning
cannot change: no calls or commands, no write to
a plain name that is not a built-in property (`x`, `y`, `timeout`, `chat`),
and no `player…`, `client.` or `level.` name. Reading a plain script
variable is allowed; it would read the target's variable of the same name
first if the target had one. `getplayer` and `getnpc` are still refused, so
the target is an expression such as `players[i]`, `npcs[i]` or `player`.
Seihai's GS2 machine runs `with` only on `player`, `this` and the other root
objects; it logs and skips any other target.

## Seen in the archives, still refused

- `players[i].x`, `.y` and `.dir` (about 1,350 occurrences across the
  client builds we hold and the NPC library) and the same reads on `npcs[i]` are
  lowered since seihai-core 0.2.1, from `members.gs2bc`: the element, then
  the member, with no conversion between. Writes to them and other fields
  after an index stay errors.
- `tiles[x, y]`, read and written, and `updateboard` are lowered since
  0.2.1 (`tiles.gs2bc`). `board[...]`, read and written in level scripts,
  stays an error: GS2 has no `board`.
- Lists inside array literals (when nothing in the literal calls a
  function) and a list stored into an array element are lowered since
  0.2.1 (`nested.gs2bc`, `elemlist.gs2bc`).

## Rejected with a compile error

- A top-level statement that is not an event test or a function, an event
  test with `else`, and event tests mixed into larger conditions.
- Reading an event flag as a value (`x = playerenters;`).
- GS1 commands with no GS2 command or opcode: the list is
  `gs1_commands_without_gs2()` (string-list commands, `tokenize`, `setgif`
  and others; the error names the GS2 replacement where there is one).
- GS1 functions with no GS2 form: `playersays`, `playersays2`, `ascii`,
  `strcontains`, `lindexof`, `onmapx`, `onmapy`, `strtofloat`, `hasweapon`,
  `getnpc`, `getplayer`.
- A function inside another function.
- A `with` body that GS2 would run differently (see `with` above), and a
  `return`, `sleep`, `waitfor` or a `break`/`continue` that leaves a `with`
  block (in either front end).
- `x in {a, b, c}`, `a[i] += v` and `a[i]++` when `x` or the index calls a
  function (the rewrite above would call it more than once).
- Array literals with nested lists where an element calls a function, and
  array literals where the order of
  the elements could matter: the GS1 interpreter evaluates them left to
  right and the GS2 encoding pushes them right to left. A literal is
  lowered when no element calls script or host code or `random`, or when
  exactly one does and the others are constants (`{next(), 1}`); any other
  mix (`{next(), next()}`, `{next(), this.v}`) needs the values in
  variables first.
- A field after an index other than a `players[i]` / `npcs[i]` read of
  `x`, `y` or `dir`, any write to one, and three or more indexes.
- `board[...]`: GS2 has no `board` (use `tiles[x, y]`).
