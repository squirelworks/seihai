# Walking collision (cross-link)

Date: 2026-10-28

The real clients (2.05 through 2.31) use a six-probe body-box routine with corner nudges and flush snaps. This supersedes the single-point wall test recovered from the SDL reimplementation.

The full description lives in `documentation/walking-collision.md` (when present) and in the Graal_analysis client notes. The earlier SDL single-point measurement is preserved under `squirelworks/Graal_analysis` `archival/foot-test-sdl.md`.

This note exists so the roadmap and protocol docs can point at a stable location while the detailed write-up settles.
