# Ato Stage 4: the 3D path, held

Date: 2026-10-31. Laundry list item 4.

## Status

The precondition is met. On 2026-10-31 at 02:19 UTC, Ato 3.153 (main
`7203690`) logged in to the patched lab GServer-v2 and received
`gs2test.nw` from the server's board, with NPC 300 and its GS2 script
(`3x-session.md`, "Lab run, 2026-10-31"). The 3.04 run is still owed.

The 3D path itself is **not started**, because nothing on the wire backs it.
The flat path (Stages 2 and 3) is enough for now: it shows that a 3.x login
and level receive work, and it is what a lab run can check.

## Why it waits

- **No 3.x Torque capture.** A 3.x client opens Torque only for a server
  whose listed name starts with `3 `, on UDP at the listed port + 1, beside
  the Graal TCP session (`torque.rs`, `is_3d_server`, `udp_port`). The only
  live Torque handshake on record is 5.12's. For 3.04 (TGE protocol 9) and
  3.153 (Graal3D protocol 19), the connect request, the accept and the
  class table come from the binaries only.
- **No server to capture against.** GServer-v2 (the lab server) has no
  Torque side, so the lab cannot produce a 3.x Torque exchange. A public 3D
  server would be an outward action, and none is known to answer a 3.x
  build.
- The laundry list says not to invent wire bytes. A connect request built
  from `TorqueProfile` alone would be exactly that.

## What would unblock it

1. A pcap of a real 3.04 or 3.153 client connecting to a server that
   speaks Torque on port + 1. Even a refused connect gives the request
   bytes.
2. Or a Torque-speaking lab server whose behaviour is itself taken from a
   capture.

## The shape, once there is a capture

- **The gate.** The same one as the Stage 2 fields: `torque::of(version)`.
  It applies only when the listed server is a 3D server. A 2D server keeps
  the flat path.
- **Connect only first.** Open the UDP socket and send the connect request
  for the build's profile. Read the accept and check its protocol number
  against `TorqueProfile::protocol`, then log the outcome. No ghosts, moves
  or drawing until the accept is matched byte for byte against the capture.
- **The flat view stays** as the fallback and as the view for 2D servers.
