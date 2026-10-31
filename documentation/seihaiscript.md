# Seihaiscript

Seihaiscript is the scripting engine for the seihai client and tools. It aims for practical GS1 compatibility while adding selected GS2-quality improvements.

## Design Goals

- Run existing GS1 NPC and weapon scripts with minimal changes.
- Provide a cleaner variable model, functions, and basic object support inspired by GS2.
- Stay clean-room: driven by public documentation, OpenGS1, observed server behaviour, and cleaned client observations rather than decompiled official VM code.
- Usable both inside the client (client-side scripts) and in tools (validation, generators, editors).

## Language Features (target)

### GS1 baseline (from OpenGS1, public docs, and cleaned 2.05 observations)
- Literals (numbers, strings, lists)
- Arithmetic, comparison, logical operators
- `if` / `else`, `for`, `while`
- Commands (see below)
- Built-in functions
- Dot-operator identifiers (`this.`, `player.`, `level.`, …)
- Basic arrays / string lists

### 2.05 command set (cleaned)
The 2.05 client uses a **parse-tree interpreter**, not bytecode. Command opcode = 600 + table index. The table contains **161** commands (opcodes 600..760). `show`/`hide` are 759/760. A small set of commands compile but have no executor case in 2.05 (`seturllevel`, `setfocus`, `resetfocus`, `setbacktile`, `setbacktile2`, `canwarp`, `canwarp2`, `drawovertrees`, `join`, `freezeplayer2`, `unfreezeplayer`, `setz`, `updateterrain`). (For 2.1+ builds Seihai runs `setfocus` / `resetfocus` as a view move: 2gta's `splashscreen.nw`, written for those clients, holds its title on `setfocus 30,30` with the player at y 65. Inferred from that script, not read in a 2.1+ binary.) Script-facing directions are 0 up / 1 left / 2 down / 3 right. `keydown(i)`: 0..3 movement, 4..6 A/S/D, 7 map, 8 tab, 9 Q, 10 P (`newfeatures2001.txt` in 2.05 rev 2), always false while paused. Full cleaned list is in the Graal_analysis notes; Seihai should implement the subset needed for offline play first.

### Planned extensions (GS2-inspired)
- Functions with parameters and return values
- Unified numeric/string variable access
- Multi-dimensional arrays
- Object/class joining (`join` / `leave`)
- Event handlers (`onCreated`, `onPlayerChats`, …)
- Timeout / scheduling primitives
- GUI control objects (later phase)

## Implemented GS1 engine

`client/src/scripting/engine/` is a first working GS1 interpreter. It is written from observed behaviour and its own design.

**Pipeline.** `parse(src, era)` never fails: it returns a `Script` plus diagnostics, and an unknown or not-yet-introduced command becomes an `Unknown` statement that is logged and ignored when run. The tree compiles to a flat node list with jump targets. Every event runs that list from the top; `if (created)`-style blocks test the event flag. `sleep` remembers the next node and the next `timeout` event resumes there.

**Language.** `if`/`else`, `while`, `for`, `break`/`continue`/`return`, assignment and `++`/`--`, the usual numeric, comparison and logical operators (a single `=` in a condition compares), numeric and string variables, arrays (`setarray`), message codes (`#c #n #a #L #F #v #s`, and from 2.02 `#t #e #T`, from 2.03 `#p`), and the built-in functions `abs int sin cos arctan random strequals strlen startswith indexof ascii arraylen onwall playersays playersays2`. Scopes: `this.` is per object, `level.` clears on a level change, `server.` is global, `client.`/`local.`/plain names are flags.

**Spellings the NPC corpus uses (2026-10-28).** The stress test (`client/tests/script_stress.rs`) parses every unique NPC script in `client/tests/fixtures/npc-scripts/`. These forms parse:

- `if(` with no space, and `a&&b` with no spaces.
- `elseif` as one word, also glued to the `}` before it. The client matches command names by prefix (as with `hurt1`), so `elseif` reads as `else` then `if`.
- `in |lo,hi>`: the high bound is read above the comparisons, so the `>` closes the range.
- A field after an index: `players[i].x`, `npcs[i].save[n]`, in conditions, expressions and assignments. The 2.05 client's tables list these with signatures `[R]N` and `[R]N[R]`.
- `in {list}` inside a larger condition.
- `<>`, `=<` and `=>`: the 2.05 client's comparison table holds them beside `!=`, `<=` and `>=`.

With these, 444 of the corpus's 3,038 scripts still had diagnostics (498 before).

**2.1 forms (2026-10-29).** Read on the 2.1 era (2.1 test builds and later) only; a 2.05 client reads none of them:

- `{a,b,c}` as a value: `this.a = {1,2,3}` fills `this.a` as an array (length and items, as `setarray` makes), and a command's number argument can be one (`showpoly 300,{0,0,32,0}`).
- `x in this.list`: true when `x` equals one of the array's values.
- `cond ? a : b`, the loosest operator.
- `with (target) body`. The NPC server notes of 2001 describe it (the body runs as `players[i]`, `getplayer(acct)`, `npcs[i]` or `getnpc(name)`), and the word is in the 2.17 client and later. Seihai does not switch objects, so the body is skipped and the skip logged; it is never run as the wrong object.
- `#P1` to `#P30` read a gani attribute (the 2.2 command list). With no index or with -1 it is the NPC's own, as set by `setcharprop #P1,value`; a player's is not known and reads empty. `#C0` to `#C4` keep their digits too, but are not answered yet.

In every era:

- An empty number argument is 0 (`seteffect ,,,;`, `callnpc,this.i,call3,;`). Inferred: that the client reads an empty field as 0 was not read from the program.
- `for (i=0;i<3;i++;)`: a `;` after the step is let through.
- `{...}` inside the raw text of a command (`setshape2 4,5,{...}`) or of an unknown command is kept whole, so its `}` no longer closes the block around it.

The corpus now has 365 scripts with diagnostics read as 2.05, and 160 read as 2.1 and later. Of the rest, many are broken scripts (`if(strequals(#g,Red Team)||)`, a sign's text run into the script), and many call functions Seihai does not have yet (`strcontains`, `lindexof`).

**Stress set, 2026-10-30.** The stress test now reads the 91 fixture blocks marked `recovered-from-binary` the way the level loader reads a binary level: each block is a whole `.graal` level dumped as text, so the test takes its NPC records out (`x+32, y+32, image#script`) and parses each as its own script. Before, the tiles, links and every NPC of such a level were parsed as one script, which made up half of the 2.1 failures. A sign or `say2` text line that starts `NPC ` (`NPC made by Ace2896 #b`) no longer starts a new block either. `STRESS_ALL=1` lists every failing script with the line of its first diagnostic. These forms now parse:

- `hideimgs();`, `move();`: a command with empty parentheses is the command, unless the script defines a function of that name, which is then called (the baddy scripts define their own `move` and `hurt`). Every era.
- `x in <lo,hi>` (2.1 and later): the GS2 grammar of the public GServer (gs2-parser) reads it as the same range as `|lo,hi|`. Whether the original treats the ends as open was not checked.
- `{0x2, ,0x2A5}`: an empty list entry is 0 and keeps its place (the boomerang's tile table counts on it). A list's `}` ends an assignment with no `;` after it (`tiles = {...}` then `function`). 2.1 and later, as lists are.
- An empty argument of a call is 0 (`textwidth(1,arial,,text)`), as an empty command argument is. Every era.
- The last text argument of a built-in runs to its `)`, commas included (`playersays(0,buy tank1)`), as a command's last text argument does. Inferred.
- `getnpc(name)` and `getplayer(account)` read their argument as text (`with (getnpc(House-Control#v(h)))`). 2.1 and later; they read 0, as `with` bodies are not run.
- `300++this.i` between two values is `300 + this.i`.
- A comma inside plain parentheses in a text argument stays in the text (`@font@#s(l) (#v(x), #v(y))`). Inferred.
- `name (text),more;` for a name that is not a command is an unknown command with that text, not a call.

Corpus: 3,306 scripts (2,947 plus 359 records from the binary levels): 352 with diagnostics as 2.05 and 95 as 2.1 and later. Every script that fails only as 2.05 uses a 2.1 form (`{...}`, `with`, `?:`). Of the 95, 53 are binary-level records (some still carry level data the dump ran into them) and 6 are Flash ActionScript files in the pack; the rest are broken scripts, such as `if (this.timeLeft > ){`, `playerx+,`, an extra `}`, a missing `;`, or a file that starts partway through a script.

**Text join and the chat attribute (2.1 and later).** `a @ b` joins two texts; `SPC`, `NL` and `TAB` join with a space, a line break or a tab between (the four operators of the GS2 grammar of the public GServer, where `@` binds looser than the comparisons and tighter than `&&`). `chat` (an object's bubble text, the same as the `message` command) and `player.chat` (the player's chat text, like `setplayerprop #c`) can be read and written; `player.account` and `player.nick` can be read. A level NPC script of a server that runs NPCs (`chat = player.account SPC "touched me";`) parsed with `expected ;` before. Open: scripts that are really GS2 (`temp.` variables, `this.x` methods, `{}` object literals) are still not run; and when the server runs the NPCs (`PLO_HASNPCSERVER`) a level file's NPC script is still run locally.

**Commands.** About 150 are recognised, each with a signature and the version that introduced it. Commands that change the world are returned as `Effect` values for the host to apply (messages, warps, sounds, images, weapon grants, player movement, flags, `sendtoserver`); the offline client applies the ones it can and prints the rest to stderr. The visual-effect commands do nothing when online without an NPC server, as the original does.

**Version gating.** `Era` follows the client version: 2.00 baseline, then 2.01 (`hitobjects`, `playerdies`...), 2.02 (string functions, `local.`), 2.03 (`triggeraction`, `action*` events, `move` with time), 2.04 (`replaceani`, tile defs, `attachplayertoobj`...), 2.05 (`loadmap`, `setbacktile2`, `updateterrain`) and 2.1 and later (`ascii`, `playerhp`...). A command or event from a later era is parsed as unknown on an earlier one.

**Scheduler.** Ticks are 50 ms. Timeouts round up to a tick and fire in order of tick, then creation. The same script and the same seed produce the same run. Limits: 10,000 loop passes, 200,000 steps per run, call depth 64, expression depth 400, parser depth 200. `callnpc`, `callweapon`, `triggeraction` and `movementfinished` are queued and run after the caller finishes (at most 1,000 per drain).

**The weapon list (2.04 and later).** `selectedweapon`, `weaponscount`, `#w(i)` (name), `#W(i)` (image) and `callweapon i,event` all index the whole weapon list the client holds, in the order it arrived, built-in and hidden (`-`) weapons included (2.04 `newfeatures2001.txt`; 2gta's `-System3` and `_Bomb_Bag` call `callweapon selectedweapon,...`). A script can set `selectedweapon`: the client's own select rule applies, so the first weapon from that index on that is not hidden is taken, and an index past the end changes nothing. The front end feeds the list in (`PlayerView::weapons`) and takes a script's pick back (`OfflineSession::weapon_select`). Offline there is no list, so `callweapon i` counts the world's weapon objects as before. Before 2.04 `selectedweapon` is an ordinary variable.

**Touch.** `playertouchsme` is edge-triggered against the NPC's rectangle, tested with the player's foot point. The rectangle is `setshape 1,w,h` (pixels) if the script set one, else the NPC's picture (its image part, or the whole image once it is loaded), else 2 by 2 tiles. The picture size came in on 2026-10-31: 2gta's north exit on `onlinestartlocal.nw` is a wide image NPC at 29,0 that could not be touched as 2 by 2.

**Host.** `scripting::engine::host::SessionScripts` wires an offline session to the engine. Scripts run by default; `--no-scripts` turns them off.

**Not done.** (`board[]` and `tiles[x,y]` reads now return the level's tiles: `board-reads.md`.) No string lists or `with`/`join`; online NPC script props are not decoded; no weapon-key input for `weaponfired`; `move` is instant; script warps online use the link request (see `documentation/script-warps.md`); sound and network effects are only logged; image effects (`setimg`, `setimgpart`, `showimg`, `hideimg`, `changeimgpart`, `setcharani`, `setani`, `showcharacter`) change the NPC and are drawn (`documentation/npc-images.md`). Assumptions that still need the lab: case-insensitive `==`/`strequals`, quote stripping, the touch geometry, and `playersays` matching rules.

## Architecture

```
scripting/
├── lexer
├── parser
├── compiler (optional bytecode)
├── vm / interpreter
├── builtins/          # Commands and functions registered by the host
└── host/              # Client or tool bindings
```

The engine exposes a host API so the client can register movement, drawing, networking, and flag commands, while a headless tool can register a reduced set.

## Relationship to OpenGS1

OpenGS1 (`../shanalysis/OpenGS1`) is the primary reference implementation for the GS1 subset. Seihaiscript may start by wrapping or reimplementing it, then diverge as GS2 features are added. Note that OpenGS1 is a reimplementation; the 2.05 client itself does not use its bytecode.

## Documentation Sources

- Graal Bible pages on GScript / GS1→GS2 migration (public)
- OpenGS1 source and tests
- Server-side script handling in GServer-v2
- Cleaned 2.05 command observations (Graal_analysis notes)
- Archived Classic scripting corpus (see `documentation/legacy-scripts.md` for patterns useful for host effects and open items)

Detailed language semantics and edge-case notes go in `../shworkingpapers/scripting/`.

## String lists and the 2.2 builtins (2026-10-29)

From the 2.2 command list (`commands.rtf`), with GServer-v2 beta4's GS1 code
as the reference where the list says little. All are 2.1 and later.

* **String lists.** A list is a flag holding comma-separated entries; an entry
  with a comma or a quote is written in double quotes (`""` is a quote).
  `addstring list,text` adds an entry (before 2.1 `addstring` still adds text
  to the string, as Seihai did), `insertstring list,index,text`,
  `replacestring list,index,text`, `removestring list,text` (every equal
  entry), `deletestring list,index`. An index past the end does nothing.
  `#I(list,index)` reads an entry, `sarraylen(list)` counts them (0 for an
  empty list; GServer counts 1) and `lindexof(text,list)` gives the first
  equal entry's index or -1. Entries compare with spaces trimmed.
* `strcontains(text,part)`: true when `part` is in `text`, ignoring case
  (GServer's case-blind find).
* `onmapx(level)` / `onmapy(level)`: the level's cell on the current map, the
  gmap the player is on, else the big map. -1 with no map, 0 for a level not
  on it (as GServer).
* `play2 file,x,y,volume` is `play file`: Seihai has no positional sound or
  volume yet (and plays no script sounds at all so far, as before).
* `sendrpgmessage text`: GServer's GS1 sends it to the player's F2 message
  window (`PLO_RPGWINDOW`); Seihai puts the line in its chat log.

Corpus: scripts with diagnostics read as 2.1 and later went from 160 to 148.
