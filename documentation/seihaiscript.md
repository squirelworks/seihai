# Seihaiscript

Seihaiscript is the scripting engine for the seihai client and tools. It aims for practical GS1 compatibility while adding selected GS2-quality improvements.

## Design Goals

- Run existing GS1 NPC and weapon scripts with minimal changes.
- Provide a cleaner variable model, functions, and basic object support inspired by GS2.
- Stay clean-room: driven by public documentation, OpenGS1, and observed server behavior rather than decompiled official VM code.
- Usable both inside the client (client-side scripts) and in tools (validation, generators, editors).

## Language Features (target)

### GS1 baseline (from OpenGS1 and public docs)
- Literals (numbers, strings, lists)
- Arithmetic, comparison, logical operators
- `if` / `else`, `for`, `while`
- Commands (`set`, `unset`, `setimg`, `move`, …)
- Built-in functions
- Dot-operator identifiers (`this.`, `player.`, `level.`, …)
- Basic arrays / string lists

### Planned extensions (GS2-inspired)
- Functions with parameters and return values
- Unified numeric/string variable access
- Multi-dimensional arrays
- Object/class joining (`join` / `leave`)
- Event handlers (`onCreated`, `onPlayerChats`, …)
- Timeout / scheduling primitives
- GUI control objects (later phase)

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

OpenGS1 (`../shanalysis/OpenGS1`) is the primary reference implementation for the GS1 subset. Seihaiscript may start by wrapping or reimplementing it, then diverge as GS2 features are added.

## Documentation Sources

- Graal Bible pages on GScript / GS1→GS2 migration (public)
- OpenGS1 source and tests
- Server-side script handling in GServer-v2

Detailed language semantics and edge-case notes go in `../shworkingpapers/scripting/`.
