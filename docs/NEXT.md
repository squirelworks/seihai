# Focused implementation tasks

The overall roadmap is [issue 353](https://github.com/squirelworks/seihai/issues/353).
The proposed engine architecture and remaining owner decisions are in
[issue 380](https://github.com/squirelworks/seihai/issues/380).

- [x] Validate known GS2 operands, string references and branch/function bounds
  before returning a parsed script, with synthetic regression tests
  ([PR 381](https://github.com/squirelworks/seihai/pull/381)).

This task covers Seihai input validation. It does not complete the GS2 runtime
integration or establish original-client compatibility.
