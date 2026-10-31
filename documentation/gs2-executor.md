# GS2 executor: run-time behaviour (Stage C)

Date: 2026-10-30. Covers `crates/seihai-core/src/scripting/gs2/exec.rs`.
It runs the bytecode the reader produces (`gs2-opcodes.md`) for the part of
GS2 that the classic GS1 engine also has. Classic clients stay on GS1, and
nothing here is on their path.

Tags: **[bin]** read from the 3.153 Mac client (the existing decompile in
the project folder, `graal-decompile/v3x/mac315.c`; no new decompile) ·
**[src]** from gs2-parser · **[inf]** inferred · **[seihai]** Seihai's own
choice where the client is not followed.

## The 3.153 client runs this bytecode

- 3.153 handles id 131 (`PLO_NPCBYTECODE`), which 3.04 does not
  (`Graal_analysis/protocol/v3x-clients.md`). **[bin]**
- Its loader (`FUN_003c9a80`) reads segment 4 and decodes the same
  operand markers `F0`–`F6` with the same widths as `gs2-opcodes.md`. A
  short operand ends the read. The writer just before it (`FUN_003c9230`)
  emits a function table with `public.` names. **[bin]**
- Its VM is `FUN_003dc954`. Its switch has a case for every opcode the
  gs2-parser compiler emits except 11 (`OP_WAITFOR`). It also has cases for
  45–47 and 66–67, which the compiler never emits (they are left for Stage D).
  3.153 also contains a GS2 compiler of its own: a flex scanner in
  `FUN_003f590c`. **[bin]**

So the 3.153 receive path in Stage E can hand id 131's payload to the
reader. The payload's own header still needs reading.

## What the VM does

The stack holds up to 256 entries. Each entry is a type tag (0 the
`OP_TYPE_ARRAY` marker, 1 number, 2 string, 3 object, 4 variable name,
5 resolved variable) and its value. **[bin]**

| Opcode | Behaviour | Tag |
|---|---|---|
| `OP_SET_INDEX` | jump | bin |
| `OP_IF` | pop. Jump if the value is 0 | bin |
| `OP_SET_INDEX_TRUE` | pop. Jump if the value is not 0 | bin |
| `OP_AND` | value 0: jump and keep it. Otherwise pop and go on | bin |
| `OP_OR` | value not 0: jump and keep it. Otherwise pop and go on | bin |
| `OP_INLINE_CONDITIONAL` | a non-zero top becomes 1 | bin |
| `OP_CMD_CALL` | adds 1 to a counter kept in the instruction. Past the limit: "Script exceeded loop limit" and the script stops | bin |
| `OP_JMP` | records the stack height in the running function's frame (where `OP_RET` puts its value) | bin |
| `OP_CALL` | see below | bin |
| `OP_FUNC_PARAMS_END` | binds the names above the inner marker to the arguments below it, first to first. Missing ones are 0. Drops both lists and the call's marker | bin |
| `OP_RET` | the top value goes into the call's slot and the caller continues. Outside any function the script stops | bin |
| `OP_SLEEP` | pop `t`. If `t > 0` the script yields for `t` seconds | bin |
| pushes 20–27 | number, string, variable name, marker, 1, 0, null object, π | bin |
| `OP_THIS` … `OP_TEMP` | an object. `temp` is the running function's frame, or the global object at top level | bin |
| `OP_COPY_LAST_OP`, `OP_SWAP_LAST_OPS`, `OP_INDEX_DEC` | duplicate, swap the top two, pop | bin |
| `OP_CONV_TO_FLOAT`, `OP_CONV_TO_STRING`, `OP_CONV_TO_OBJECT` | resolve the variable and convert in place | bin |
| `OP_MEMBER_ACCESS` | object and name become a variable of that object. A null object gives an empty name | bin |
| `OP_ASSIGN` | pops value and target, then stores. Nothing is left on the stack | bin |
| `OP_INC`, `OP_DEC` | change the variable on top, which stays there | bin |
| arithmetic 60–65 | on the numbers in the two top entries. `x / 0` is 0 | bin |
| `OP_NOT`, `OP_UNARYSUB`, `OP_ABS`, `OP_SIN`, `OP_COS`, `OP_ARCTAN` | on the top number | bin |
| `OP_INT`, bitwise 76–79 | through a double-to-int conversion | bin |
| `OP_RANDOM` | `S1 + r·(S0 − S1)`, `r` from the client's generator | bin |
| `OP_MIN`, `OP_MAX` | the smaller or larger of the two by the comparison below | bin |
| comparisons 70–75 | a three-way compare: two strings as text, two numbers as numbers within a small epsilon, a string against a number as the number it reads as | bin |
| `OP_JOIN` | `S1` text followed by `S0` text | bin |

A stack underflow or overflow stops the script.

**`OP_CALL`**: the name is on top, with an object if it was `obj.name`. The
arguments run from just under it back to the nearest marker, the first
argument nearest the name (the compiler pushes them in reverse). A name
with no object, or on the script's own object, is first looked up in the
function table. A found function gets a frame, and the call is refused
past 10 000 frames ("callstack overrun"). Otherwise the name goes to the
built-in command table. Each command there has a parameter signature, and
too few arguments gives "GraalScript: Not enough parameters for …". If the
name is in neither, the message is "GraalScript: function … not found in
script of …", and the call's slot becomes the number 0. **[bin]**

**Unqualified variables** are looked up in this order: the `with` objects,
the running function's locals, the script's object (`thiso`), the level,
and then the global object. `temp`, `params`, `this`, `thiso`, `player`,
`playero`, `level`, `client`, `clientr` and names starting with `$` are
handled before that search. **[bin]**

## Seihai's executor

- `Machine::run_top_level` runs from op 0 (the compiler jumps over every
  function body). `Machine::call_event("onCreated", args)` calls a
  function the way `OP_CALL` would. Both return `Done`, `Sleep(t)` (go on
  with `resume`), `Stopped(reason)` or `NoSuchEvent`.
- Every opcode in the table above is run. Anything else (arrays,
  objects, `with`, `for`-each, the string methods, `format`, shifts, `exp`,
  `log`, `in`, `params`) is logged as "not run yet (Stage D)" and skipped.
  A value the reader does not name is logged as an unknown opcode and
  skipped. Skipping can leave the stack uneven. The VM's own checks still
  stop the script if that happens.
- The timer commands (`settimer`, `timeout`, `scheduleevent`,
  `cancelevents`) are the machine's own and run on `Machine::tick`; see
  `gs2-commands.md`.
- A call that is not to a script function goes to `Host::command`.
  `EffectHost` maps the commands GS1 already has to the GS1 engine's
  `Effect`s, the same values `scripting/engine` produces:
  - `set`, `unset`: `SetFlag`, `UnsetFlag`, with GS1's `name=value` form
    and its `local.` rule
  - `message`, `say`, `say2`, `setlevel`, `setlevel2`, `setplayerprop`:
    their own effects
  - every other effect command in `vocab.rs` (`showimg`, `hideimg`,
    `play`, …): `Effect::Command` with the evaluated arguments, as GS1
    sends them

  Engine-side GS1 commands (arrays, timers, string lists) are not mapped,
  because GS2 has its own forms of them (Stage D). `this.x` and `this.y`
  are `EffectHost` properties and a write gives `SetPos`.
- Variables of `this` and `thiso` share one store. A dotted name under a
  variable (`this.a.b`) is kept as one name, `a.b`, under its root.
  **[seihai]** Real objects are Stage D.

### Not taken from the client

- The loop limit is 10 000 passes per `OP_CMD_CALL`, the figure in the
  gs2-parser enum comment **[src]**. The client's constant was not read
  from the binary. The client keeps the counter in the instruction itself.
  Seihai resets it on each run **[seihai]**.
- Numbers compare exactly. The client's epsilon was not read **[seihai]**.
- Strings compare byte for byte. Whether the client's string compare
  ignores case was not checked **[inf]**.
- `OP_RANDOM` uses Seihai's own generator. Numbers print as the GS1
  engine prints them (`num_to_string`). **[seihai]**
- Seihai stops a run after 2 000 000 steps without a yield, which the
  client does not do **[seihai]**.

## Tests

`crates/seihai-core/tests/gs2_exec.rs` runs four fixtures built with the
compiler:

- `shared.gs2`: a flag, `showimg`/`hideimg` with `x`/`y` from the host, then
  a sleep that resumes.
- `calls.gs2`: a function with parameters and a return value, `@`, `&&` in
  an `if`, `say2`, `setlevel2`, `unset`, and a `while (true)` that stops at
  the loop limit with the counter at exactly 10 000.
- `control.gs2`: `for`, `continue`, `while`, `break` and `switch` reach
  `case 1`. The skipped `in` is logged; `setTimer` sets the timer.
- A missing event is reported as `NoSuchEvent`.
