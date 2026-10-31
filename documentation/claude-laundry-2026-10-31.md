# Claude Laundry List — 2026-10-31

Continue from main at `7203690`. Last night landed GS2 Stages A–E and a batch of classic wire fixes. Work the items in order. When one is done, open the PR and start the next. If a lab run is needed and the lab is busy, note it and move to the next item that does not need the lab.

## 1. Ato Stage 2 — board and props, log only

Receive a 3.x board and the props that come with it. Log the fields. Do not draw yet. Gate by the Torque profile. Add tests from any capture that already exists.

## 2. Ato Stage 3 — flat 2D so a 3.x level is visible

Draw the board that Stage 2 received. Tiles, links, and the player position are enough. No 3D. The goal is a visible level so a lab run can confirm login and level receive.

## 3. Move the 3.x start window out of mae

Picking 3.x in the GUI should launch ato. The 3.x start window is already in `seihai-ato` (#222). mae does not hand off to it yet. Do that hand-off.

## 4. Ato Stage 4 — 3D path

Hold until a lab run shows 3.x login and level receive. Once that run exists, add the 3D path behind the same profile gate. If the flat path is enough for now, leave a note and move on.

## 5. GS2 Stage F — remaining shared commands

Stages A–E are in. Finish the shared command set that the executor does not yet cover. List any command that is still a stub. Do not invent bytes.

## 6. Classic leftovers that are still open

- 1.34–1.38 send the head image name in their login props, as 1.39 does. Seihai only does it for 1.39.
- Lab run of a link into a level that is not local, to confirm first-load warps on the wire.
- Differential-testing gaps that were skipped: walk.gani request, some walk-update rules, part of the login frame bundling.
- Whether any public server enforces the tamper value. Read-only check only, and only if the network can reach the listserver.

## 7. Seihai Version ID

A token of its own, not a GNW#####. Add a profile row that the listserver and gserver can recognise. Keep every existing profile. The Seihai session has a cleaner login and a cleaner prop set. Document the shape.

## 8. Seihai Listserver shape

Answer the old shapes (A/B/C, gen-2/3/5) and the Seihai shape. A classic client still gets a list. A Seihai client gets the list plus whatever extra the Seihai profile asks for. This can be a lab-only listserver first.

## 9. Seihai GServer skeleton

A server that speaks the same protocol and runs the same script engine. Accept classic and Torque clients on their profiles. New worlds can use the Seihai path. Start with login, level load, and walk. Combat and scripts come after.

## 10. Seihai Script extensions

Keep the GS1 and GS2 engines that already run old scripts. Add the things the old engine makes painful: better string handling, clearer scoping, a module system, and a way to say “this script is Seihai-only.” Gate the new pieces so old scripts stay unchanged.

## Order and notes

- Items 1–3 are the next client work. Do them first.
- Item 4 waits on a lab run.
- Items 5–6 are classic and GS2 leftovers. Do them as the lab is free.
- Items 7–10 are the Seihai stack from `documentation/seihai-stack.md`. Do them after the client speaks every version it claims.
- Do not invent wire bytes. If a capture is missing, leave a note and move on.
- Keep the “if someone wants legacy or Torque, it is there” promise.
