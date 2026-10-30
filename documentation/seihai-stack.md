# Seihai Stack

Seihai is the ground-up client. Once the classic and Torque intricacies are solid, the rest of the stack is built to it. Legacy and Torque stay as compatibility modes.

## Seihai Version ID

A token of its own, not a GNW##### borrowed from a Delphi build. The listserver and gserver recognize it as a first-class client. Profiles for 1.x–2.31 and the two Torque builds stay as they are for people who want those. A Seihai session has a cleaner login and a cleaner prop set.

## Seihai GServer

Once the client side of levels, NPCs, combat, and scripts is solid, a server that speaks the same protocol and runs the same script engine is the next piece. It can still accept classic and Torque clients on their profiles, so existing worlds keep working. New worlds can use the Seihai path without the old packet quirks.

## Seihai Listserver

It answers the old shapes (A/B/C, gen-2/3/5) and also the Seihai shape. A player who only has classic clients still gets a list. A player on Seihai gets the list plus whatever extra the Seihai client asks for.

## Seihai Script

GraalScript is the existing language. Seihai Script is a cleaned-up version that still runs the old scripts (so playerworlds do not break) and adds the things the old engine makes painful: better string handling, clearer scoping, a module system, and a way to say “this script is Seihai-only.” The current GS1 engine in seihai-core is the place to grow that from.

## Order

Finish Ato Stages 2–4 and the remaining classic gaps first, so the client really does speak every version it claims. Then:

1. Seihai Version ID and the login shape that goes with it.
2. Seihai Listserver that answers both the old shapes and the new one.
3. Seihai GServer that runs levels and scripts for both.
4. Seihai Script extensions on top of the existing engine, gated so old scripts stay unchanged.

That keeps the “if someone wants legacy or Torque, it is there” promise, and makes the new client the thing that drives the rest of the stack.
