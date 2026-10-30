# GS2 grammar

Date: 2026-10-30. GS2 is the script language of GServer-v2 and the later
clients. It is not the GS1 text that the classic clients run, and the
classic path stays GS1.

Source: the open-source compiler [gs2-parser](https://github.com/xtjoeytx/gs2-parser)
(GPLv3), from the 2026-10-31 source tarball. It has no `.git`, so the commit
is unknown. The build reproduces the repository's own baselines byte for byte
(`tests/baselines/expressions/01_arithmetic.bytecode`, sha256 `dca1a6d1…`).
Read from `src/parser/gs2parser.y` (bison) and `src/parser/gs2scanner.l`
(flex). The rules below restate those files in this repo's notation. No
source is copied. The opcodes the rules compile to are in `gs2-opcodes.md`.

Notation: `a | b` is a choice, `[a]` is optional, `{a}` is zero or more,
quoted text is a literal token.

## Tokens

- Whitespace and newlines are skipped. Comments are `// …` to the end of the
  line and `/* … */`.
- Identifiers: a letter, `_` or `$`, then letters, digits or `_`. A run joined
  by `::` (`a::b::c`) is one identifier.
- Integers are decimal or `0x` hex. A float is `digits . digits` or
  `. digits`. There is no exponent form.
- Strings are `"…"` with backslash escapes. A one-character `'x'` is also a
  string.
- `NL`, `SPC` and `TAB` are the `@` operator with a newline, a space or a tab
  between the two sides (`a SPC b` is `a @ " " @ b`).
- `xor` is the bitwise-xor operator. `_` is the translate cast. `int` and
  `float` are cast keywords, so `int(x)` is a cast and not a call.
- `:=` is the same as `=`. `<>` is `!=`. `=<` and `=>` are `<=` and `>=`.
- Keywords: `public if else elseif for while break continue return function
  new with switch case default const enum int float in`.

## Program

```
program    = { decl }
decl       = stmt | fndecl | constdecl | enumdecl
constdecl  = "const" ident "=" ( constant | "-" int | "-" float | ident ) ";"
enumdecl   = "enum" [ ident ] "{" enumitem { "," enumitem } "}"
enumitem   = ident [ "=" [ "-" ] int ]
fndecl     = [ "public" ] "function" ident [ "." ident ] "(" [ exprlist ] ")" [ stmt ]
```

A function with no body (`function f();`) is allowed by the grammar.
`public function obj.f()` is a function on an object. Constants and enum
values are folded in at compile time and leave no opcode.

## Statements

```
stmt     = block | ifstmt | forstmt | whilestmt | withstmt | switchstmt
         | newstmt | "return" [ expr ] ";" | "break" ";" | "continue" ";"
         | [ expr ] ";"
block    = "{" { stmt } "}"
ifstmt   = "if" "(" expr ")" stmt [ "else" stmt | "elseif" ifrest ]
ifrest   = "(" expr ")" stmt [ "else" stmt | "elseif" ifrest ]
forstmt  = "for" "(" [ expr ] ";" expr ";" expr ")" stmt
         | "for" "(" expr ":" expr ")" stmt
whilestmt  = "while" "(" expr ")" stmt
withstmt   = "with" "(" expr ")" stmt
switchstmt = "switch" "(" expr ")" "{" caseblock { caseblock } "}"
caseblock  = ( "case" expr | "default" ) ":" ( stmt { stmt } | caseblock )
newstmt    = "new" ident "(" [ exprlist ] ")" block
```

`for (x : list)` is the for-each form. There is no `do … while`. A
`newstmt` (`new GuiWindowCtrl("name") { … }`) makes the object and runs its
block inside a `with` on it. After the block, the compiler emits one call by
name to `addcontrol` for each `newstmt` made inside it.

## Expressions

```
expr     = postfix | cast | arraylit | unary | binary | compare | inexpr
         | expr "?" expr ":" expr
postfix  = primary { "[" exprlist "]" | "(" [ exprlist ] ")" | "." primary }
primary  = constant | ident | "(" expr ")"
constant = int | float | string
cast     = ( "int" | "float" | "_" ) "(" expr ")"
arraylit = "{" exprlist [ "," ] "}"
unary    = ( "-" | "@" | "!" | "~" | "++" | "--" ) expr | expr ( "++" | "--" )
binary   = expr op expr
compare  = expr ( "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&" | "||" ) expr
inexpr   = expr "in" "|" expr "," expr "|"
         | expr "in" "<" expr "," expr ">"
         | expr "in" expr
exprlist = ( expr | fnobj ) { "," ( expr | fnobj ) }
fnobj    = "function" "(" [ exprlist ] ")" stmt
```

`op` is one of `+ - * / % ^ & | xor << >> @` or an assignment: `= += -= *=
/= ^= %= <<= >>= @=`. The right side of `=` can also be `new ident(args)`,
`new [n]{[n]}` (an array of fixed size, the sizes being integer literals),
`fnobj` (a function value) or `{}` (an empty object).

Calls are the postfix `(…)` form. `a.b(…)` calls `b` with `a` as the object.
The compiler maps some names to their own opcodes (the built-ins in
`gs2-opcodes.md`). Every other call, which covers all the game commands
(`say2`, `setTimer`, `showimg`, …), is a call by name.

Reserved names that compile to their own opcodes, not to a variable: `this`,
`thiso`, `player`, `playero`, `level`, `temp`, `true`, `false`, `null`, `pi`.

## Precedence

From lowest to highest, as declared in the `.y` file. Rows marked *right*
group right to left. The rest group left to right.

| Level | Operators |
|---|---|
| 1 (*right*) | `=` and the compound assignments (`\|=` and `&=` do not parse, see below) |
| 2 | `[`, `?`, `:` |
| 3 | `\|\|` |
| 4 | `&&` |
| 5 | `&` `\|` `xor` `<<` `>>` |
| 6 | `@` (and `NL` `SPC` `TAB`) |
| 7 | `<` `<=` |
| 8 | `>` `>=` |
| 9 | `==` `!=` `in` |
| 10 | `+` `-` |
| 11 | `*` `/` `^` `%` |
| 12 (*right*) | `!` `++` `--` `~` |
| 13 | `.` |

`<` and `>` sit on different levels, and both sit below `==`. `^` is power
and has the same precedence as `*`. The unary rules carry no `%prec`, so by
bison's rule each takes the level of its operator token: unary `-` sits with
binary `-`, and unary `@` with `@`. These are what the `.y` file says. Whether the real compiler that
the official server used agrees has not been checked.

## Not covered by the `.y` file

- The scanner returns tokens for `|=` and `&=`, but the grammar has no rule
  for them, so a script that uses them does not parse. The built compiler
  rejects `temp.a |= 1;` with "missing semicolon".
- `elseif` and `else` share one precedence level, which settles the
  dangling else. `else if` written as two words is `else` followed by an
  `if` statement, and it compiles.
- The meaning of the GS1-style event names (`onCreated`, `onTimeout`,
  `onPlayerEnters`, …) is not in the grammar. They are ordinary function
  names. What calls them is the server or client (`GServer-v2`
  `GS2ScriptManager`, Stage D).
- `join("class")` with a string literal is recorded by the compiler as a
  class the script needs. It is still compiled as a call by name.
