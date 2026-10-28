# Server and listserver reference materials

Seihai is a client. Anyone expanding it into the gserver or listserver space (or writing a companion server) will find the following useful.

## Archived local research (decommissioned 2026-10-28)

A complete local research workspace containing original gserver source dumps, the Reborn 2.4.0 Phoenix binary package, a Python reference rewrite (Ash), an architecture atlas (Ember), and a 39-chapter narrative walkthrough (GServerStory) was archived when that project was decommissioned.

- **Google Drive:** `/[Grok/[Projects/Graal gServers.zip`
- **Direct link:** https://drive.google.com/file/d/1XtT0MEltbmApGG5_cL_TmR8QnvgPYsAS/view?usp=drivesdk

The archive is ~27 MB and contains the primary sources for the server side of the same protocol Seihai speaks.

## Ash — readable Python gserver

Inside the archive is **Ash**, a Python 3 rewrite of the gs2emu / Reborn protocol surface. It is deliberately cut down to what a v2 client actually needs:

- 2-byte length framing
- encryption generations 3 / 4 / 5
- login (client, RC, NPC-Server)
- listserver register / ping / VERIACC2 (works offline)
- GRACC001 accounts and GLEVNW01 levels (including baddies)
- props, chat, PM, warp, file send
- RC file browser
- gmaps / bigmaps / groupmaps
- trigger-action hacks

Ash listens on port 14812 by default and ships a complete world under `servers/ash/`. It is a practical, readable reference when implementing or debugging a server that speaks the same wire protocol as Seihai.

The companion analysis repository (`squirelworks/Graal_analysis`) now also contains extracted server-side notes (lineage, protocol, algorithms) under `server-notes/` and an `ARCHIVAL.md` that points at the same Drive archive.

## Protocol surface that is stable

The packing (`gchar` / `gshort` / `gint*`), the two-byte big-endian frame, the login type-shift that selects the encryption generation, the `servers.txt` multi-world layout, and the GRACC001 / GLEVNW01 on-disk formats are the load-bearing contracts. Everything else (specific baddy modes, horses, weapon bytecode, the exact listserver discovery mechanism) can be modernized.

---

*Added when the local server research workspace was decommissioned. The Drive archive remains the authoritative source for the original dumps and the full Ash / Ember / GServerStory materials.*
