# GS2-only commands (Stage D)

Date: 2026-10-31. The GS2 commands the classic GS1 engine does not have,
one group per PR: timers, strings, lists, then the rest. Each one is read
in GServer-v2 (the 2026-10-31 source tarball) and in the 3.153 client's
command table, then run in `scripting/gs2/exec.rs` or listed here as not
run.

Tags as in `gs2-executor.md`: **[bin]** the 3.153 Mac client (the existing
decompile, `graal-decompile/v3x/mac315.c`) · **[src]** GServer-v2 or
gs2-parser · **[inf]** inferred · **[seihai]** Seihai's own choice.

GServer-v2 has no GS2 VM. Its server-side scripts are JavaScript on V8, so
its bindings (`server/src/Scripting/v8/`) give the meaning of a command,
not how a GS2 client runs it.

The 3.153 client registers each built-in with a return type and a
parameter string: `d` number, `s` string, `X` any number of further
arguments; the return types seen are `v` (none) and `o` (an object).
**[bin]** Built-in names are lower case in that table and scripts write
them in any case (`setTimer`), so Seihai matches them without case.
**[inf]**

## Timers

| Command | 3.153 table | GServer-v2 | Seihai |
|---|---|---|---|
| `settimer(t)` | returns nothing, `d` | `npc.settimer(t)`: `timeout = (int)(t * 20)` frames | the same |
| `timeout` | a property of the object with a getter and a setter | reads `frames / 20`, a write sets `(int)(t * 20)` frames | `this.timeout`, `thiso.timeout` and an unqualified `timeout` read and write it |
| `scheduleevent(t, event, args…)` | returns an object, `dsX` | `npc.scheduleevent(t, fn, data)`: runs `fn` after `(int)(t * 20)` frames | runs `on<event>` with `args` after that many frames, and returns 0 |
| `cancelevents(event)` | returns nothing, `s` | none | drops the waiting `scheduleevent`s of that event |

**Frames.** GServer-v2 counts `timeout` and scheduled events in frames of
1/20 s (`TNPC::runScriptTimer`). Each frame takes 1 off `timeout`; when it
reaches 0 the NPC's `onTimeout` is queued, once. A `timeout` of 0 is off.
Then each scheduled event takes 1 off, and one that reaches 0 runs and is
removed. **[src]** The client's own frame length was not read.
`Machine::tick` is one frame of `TICK` (0.05 s) in the same order:
`onTimeout`, then the scheduled events in the order they were made.
**[seihai]**

**Event names.** The client's event dispatcher (`0x3e2c0c`) accepts a name
with or without its leading `on`, and the timer's own event is `timeout`.
So `onTimeout` is the function a timer runs, and `scheduleevent(1, "Ping")`
and `scheduleevent(1, "onPing")` both run `onPing`. **[inf]**
`cancelevents` compares names the same way, without case. **[inf]**

### Where Seihai differs

- `scheduleevent` returns an object in 3.153 (what the object holds was
  not read). Seihai returns 0. **[seihai]**
- A `scheduleevent` of less than one frame never runs on GServer-v2, as its
  count goes past 0. Seihai runs it on the next frame. **[seihai]**
- A machine runs one event at a time. An event that comes due while a run
  is asleep (`sleep`) waits until that run has ended and runs on a later
  frame; starting it would end the sleeping run. **[seihai]**
- `cancelevents` also drops an event that has come due but is still
  waiting for a sleeping run. **[seihai]**

### Not run

- `waitfor(obj, event, t)`: the compiler emits `OP_WAITFOR` (11), but the
  3.153 VM has no case for it (`gs2-executor.md`). Seihai logs it as an
  opcode it does not run, and the script goes on.
- `timershow` and `timereverywhere` are GS1 commands. They reach
  `EffectHost` as GS1 effects, as in Stage C.

### Timer tests

`tests/fixtures/gs2/timers.gs2`, run by `tests/gs2_exec.rs`:
`setTimer(0.1)` is two frames; `onTimeout` re-arms itself with
`this.timeout = 0.05` until it has run three times; `scheduleevent` with and
without `on` comes at frames 5 and 6; a cancelled event does not run; and a
timer that runs out during a `sleep` runs on the frame after the run ends.

## Strings

These are opcodes, not calls by name (`gs2-opcodes.md`, "Built-in
functions"), so GServer-v2, which has no GS2 VM, has nothing on them. All
of this is from the 3.153 VM (`0x3dc954`) and its helpers. **[bin]** The
compiler converts the object of a method to a string first
(`OP_CONV_TO_STRING`).

| Built-in | Opcode | 3.153 |
|---|---|---|
| `char(n)` | `OP_CHAR` | the byte `(char)(int)n` as a one-byte string |
| `s.trim()` | `OP_OBJ_TRIM` | drops leading and trailing bytes whose low 7 bits are 0x20 (space and 0xA0); tabs stay (`0x3d0588`) |
| `s.length()` | `OP_OBJ_LENGTH` | the byte count |
| `s.pos(t)` | `OP_OBJ_POS` | the first index of `t`, or -1; an empty `t` or one longer than `s` is -1 (`0x3cff90`) |
| `s.charat(i)` | `OP_OBJ_CHARAT` | the byte at `int(i)`, or "" outside the string |
| `s.substring(a, n)` | `OP_OBJ_SUBSTR` | `a` below 0 is 0, past the end is the end; `n` below 0 runs to the end; the end stops at the length (`0x3da32c`) |
| `s.starts(t)`, `s.ends(t)` | `OP_OBJ_STARTS`, `OP_OBJ_ENDS` | 1 or 0 |
| `_(s)` | `OP_TRANSLATE` | leaves the value as it is |
| `format(f, …)` | `OP_FORMAT` | see below |

`int(i)` above is the client's double-to-int conversion (`0x3d3ab8`). It
rounds down and adds a small constant first, whose value was not read.
Seihai rounds down. The same conversion is used by `OP_INT` and the
bitwise operators. Stage C truncated instead, so `int(-1.5)` was -1; it
is -2 now, as in the client.

**`format`** (`0x3dbb0c`): the pattern is on top of the stack and its
arguments are below it, back to the marker, the first nearest. The whole
list becomes the result. With no pattern the result is "". The pattern is
read as `%[-][+][width][.precision][l]` and one letter. Width and
precision stop at 120.

- `d`, `i`: the argument through `int()`, with at least `precision` digits,
  a `+` with the flag, padded with spaces to `width` (on the right with
  `-`). `u` is the same, unsigned.
- `s`: the argument's text cut to `precision`, padded with spaces on the
  left to `width`. The `-` flag is read but not used.
- `f`: the number as the client writes numbers. The writer is not given the
  width or the precision. **[bin]** How it rounds was not checked; Seihai
  writes it as the GS1 engine does (`num_to_string`). **[seihai]**
- `%%` is `%`. Any other letter is dropped with its `%`. A `%` at the end of
  the pattern ends it.

The padding byte for `s` is a one-byte string constant in the client
(`PTR_DAT_007ad4b8`) that was not read. Seihai uses a space. **[inf]**

### Where Seihai differs

- When the pattern asks for more arguments than were given, the client
  goes on reading down its stack. Seihai uses 0 or "". **[seihai]**
- `s.substring(a)` with one argument: gs2-parser pushes only `a`, but the
  3.153 op always takes the string, the start and the length. Seihai does
  the same, so the call takes one value too many off the stack (here the
  target of the assignment), as it would in the client. Write
  `s.substring(a, -1)` instead. **[src]/[bin]**
- `starts` and `ends` with an empty `t` give 1. What the client's helpers
  (`0x3d2a88`, `0x3d2b4c`) do with an empty string was not checked. **[inf]**

### String tests

`tests/fixtures/gs2/strings.gs2`, run by `tests/gs2_exec.rs`, and unit
tests in `scripting/gs2/strings.rs` for `format`, `substring`, `trim` and
the integer conversion.

## Lists

Opcodes again, so the source is the 3.153 VM. **[bin]** The client keeps
a list as a variable object (`TGraalVar` in the class names) with a kind:
numbers, a string, or objects. Its methods (size, element at, insert at,
set at, delete at) are reached through a method table whose entries are
not in the decompile. So the VM shows which values each opcode takes off
the stack, in which order, and what it does with them; what happens past
the end of a list, inside those methods, was not read. **[bin]/[inf]**

Seihai keeps a list as a value of the variable (`lists::Value`) and
copies it on assignment. Whether the client copies or shares a list on
`b = a` was not read. **[seihai]** A list read as a plain value, or handed
to a host command, is its elements joined with commas, the form of a GS1
string list. **[seihai]**

| Built-in | Opcode | 3.153 | Seihai |
|---|---|---|---|
| `{a, b, …}` | `OP_ARRAY_END` | takes everything down to the marker, the first element nearest the top (`0x3d69c4`) | the same |
| `new[n]` | `OP_ARRAY_NEW` | `n` zeros, `n` kept to 0..10 000 (`0x3d6b08`) | the same |
| `new[n][m]` | `OP_ARRAY_NEW_MULTIDIM` | each element becomes `m` zeros, recursively (`0x3d6bcc`) | the same |
| `a.size()` | `OP_OBJ_SIZE` | the list's size; 0 for no object | 0 for anything that is not a list |
| `a[i]` | `OP_ARRAY` | by kind: the number at `i`, the variable's string (no index), or the object at `i`; 0 for no object (`0x3da5d4`) | the element at `i`; a string or number variable gives itself; past the end is 0 **[inf]** |
| `a[i] = v` | `OP_ARRAY_ASSIGN` | sets the element at `i` by the value's kind and leaves nothing on the stack (`0x3da744`) | past the end, nothing happens **[inf]** |
| `a[i, j]`, `a[i, j] = v` | `OP_ARRAY_MULTIDIM`, `…_ASSIGN` | needs a two-dimensional variable (`0x3da904`, `0x3daaf8`) | element `j` of element `i` **[inf]** |
| `a.add(v)` | `OP_OBJ_ADDSTRING` | inserts at -1, the end (`0x3dae0c`) | the same |
| `a.insert(i, v)` | `OP_OBJ_INSERTSTRING` | the index is on top, the value under it; inserts at `i` (`0x3db220`) | `i` kept to 0..size **[inf]** |
| `a.replace(i, v)` | `OP_OBJ_REPLACESTRING` | the index on top; sets at `i`, or adds at the end when `i` is below 0 or past the end (`0x3db064`) | the same |
| `a.delete(i)` | `OP_OBJ_DELETESTRING` | deletes at `i` | out of range, nothing **[inf]** |
| `a.remove(v)` | `OP_OBJ_REMOVESTRING` | finds `v` and deletes the first match (`0x3daf74`) | the same |
| `a.clear()` | `OP_OBJ_CLEAR` | clears the list | the same |
| `a.index(v)` | `OP_OBJ_INDEX` | the first index of `v`, or -1; -1 for an empty list (`0x3db388`) | the same |
| `v in a` | `OP_IN_OBJ` | 1 if `v` is in `a`; a list `v` is in when each of its elements is (`0x3d9f14`) | the same |
| `s.tokenize(seps)` | `OP_OBJ_TOKENIZE` | splits on the bytes in `seps` (`" ,"` when none is given); `"…"` is one token with `""` for `"`; tokens trimmed, empty unquoted ones dropped; no tokens gives no object (`0x3db984`, `0x3c1884`) | the same |
| `s.positions(t)` | `OP_OBJ_POSITIONS` | every index where `t` starts, overlapping ones too; an empty list when there is none (`0x3db7a8`) | the same |
| `a.subarray(start, n)` | `OP_OBJ_SUBARRAY` | takes the list, the start and the count, the count on top; `start` kept to the list, `n` below 0 runs to the end (`0x3da47c`) | the same |
| `for (v : a)` | `OP_FOREACH` | with the loop variable, the list and the index on the stack: sets the variable to element `index`, or jumps out past the end | the same |

Searches (`index`, `remove`, `in`) compare numbers with numbers and
strings with strings (`0x3ef660`, `0x3ef704`). Seihai uses the executor's
comparison (`gs2-executor.md`), and lists compare element by element.
**[inf]**

`tokenize` here is the GS2 one. It is not the GS1 engine's `tokenize`,
which always splits on spaces and commas and does not trim.

### Where gs2-parser and the 3.153 VM do not agree

- `a.subarray(start, n)`: gs2-parser pushes `n`, then `start`, then the
  list, and does not convert it to an object. The VM wants the list
  first and the count on top. Seihai follows the VM, so a script built by
  gs2-parser gets a wrong result from `subarray`. **[src]/[bin]**
- `arraylen(a)` and `sarraylen(a)`: gs2-parser converts `a` to a number
  (`OP_CONV_TO_FLOAT`) before `OP_OBJ_SIZE`. In Seihai that makes it 0.
  Whether the client's conversion keeps the list was not read. Use
  `a.size()`. **[src]/[inf]**
- After a `for`-each loop, gs2-parser pops only the index. The VM leaves
  the loop variable and the list on the stack too, and so does Seihai;
  they go when the function returns. **[src]/[bin]**

### Not run

- `setarray(var, n)` (`OP_SETARRAY`): its helper (`0x3d6d7c`) was not
  read. It is logged and skipped.
- `a.indices(…)`, `a.link(…)`, `a.type()`: these are part of "the rest".

### List tests

`tests/fixtures/gs2/lists.gs2`, run by `tests/gs2_exec.rs`, and unit tests
in `scripting/gs2/lists.rs` for `tokenize`, `positions` and `subarray`.
