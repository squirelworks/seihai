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
| `{a, b, c}` with scalar elements (2.1 and later) | array literal | `lists.gs2bc` |
| `a[i]`, `a[i, j]` read | `OP_ARRAY`, `OP_ARRAY_MULTIDIM` | `lists.gs2bc` |
| `a[i] = v`, `a[i, j] = v` | `OP_ARRAY_ASSIGN`, `OP_ARRAY_MULTIDIM_ASSIGN` | `lists.gs2bc` |
| `x in this.list` (2.1 and later) | `OP_IN_OBJ` | `lists.gs2bc` |
| `playerx`, `playery` | `player.x`, `player.y` | same bytes as the GS2 source |
| `sleep`, `setarray`, `sin`, `cos`, `arctan`, `abs`, `random`, `int`, `exp`, `log`, `min`, `max`, `pow`, `char`, `getangle`, `getdir`, `vecx`, `vecy`, `waitfor`, `format`, `makevar`, `arraylen`, `sarraylen` | their opcodes, with Seihai's argument-count check | `gs2-opcodes.md`, argument tests |
| `strlen(s)`, `startswith(s, t)`, `indexof(s, t)` | `OP_OBJ_LENGTH`, `OP_OBJ_STARTS`, `OP_OBJ_POS` | `gs2-opcodes.md` only |
| a GS1 command that GS2 also has | a call by name | `shared.gs2bc` (`showimg`, `hideimg`, `set`) |

A computed array index (`a[this.i + 1]`) is converted with
`OP_CONV_TO_FLOAT` like any numeric operand. The fixture only has constant
indexes, so that conversion is Seihai's choice.

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
- `with`, and a function inside another function.
- `x in {a, b, c}`.
- Array literals whose elements contain calls or nested lists: the GS1
  interpreter evaluates them left to right and the GS2 encoding pushes them
  right to left, so they need an order-preserving lowering first.
- A list stored into an array element (`a[0] = {1}`).
- `a[i] += v` and the other compound assignments, and `a[i]++` / `a[i]--`.
  There is no fixture for their encoding. Write `a[i] = a[i] + v`.
- A field after an index (`npcs[i].x`), and three or more indexes.
- `board[...]` and `tiles[...]`: no fixture shows how GS2 encodes them.
