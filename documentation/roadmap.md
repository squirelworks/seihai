# Seihai Roadmap

Five layers. The first four are largely understood; the fifth is where the unknowns are. Start with gmaps, then the script engine, then differential testing.

## 1. Data-driven client core

Single code path. Versions are profile rows, not branches.

- [ ] Profile rows set framing, login token, id sets, field layouts and timers. All 26 rows come from the Graal_analysis TOML.
- [ ] CI check that fails when the tables and the code drift apart.
- [ ] Per-version replay tests. Each profile gets the real captured frames from the lab and the decompiles. Sixty-odd real frames already exist; every profile must decode and re-encode them byte for byte.

## 2. Wire parity, by version

- [ ] Gmaps. Ids 49, 185 and 186, cell props 43/44, and the crossing behaviour captured in the lab.
- [ ] Cached level links. The 30-plus-46 form.
- [ ] Script-initiated warps. Needs the script engine below.
- [ ] Remaining fields. The unresolved id 24 and 46 meanings, and the field order for id 178 when a real service sends it.
- [ ] Early builds. A real 2.00-era listserver reply, which is still unobserved.

## 3. Gameplay the client decides on its own

- [ ] Health and death. The 1.5 s timer, input lock, dead status bit, respawn hearts by AP, and the victim-sent claim packet.
- [ ] Combat. Sword hits are judged by the victim's client, with a reach of 0–4 tiles, a lateral limit under 2, and AP 100 immune.
- [ ] Swimming, emotes and chat. Head-only swim blit is done; emotes are local only. Chat popups and the buddy, ignore and mute files remain.
- [ ] Unresolved rules. The damage formula and the respawn "same name group" rule.

## 4. Content and scripting

- [ ] Script engine. The NPC and weapon script vocabulary, with a per-version delta table already mined from the builds.
- [ ] Assets. Loaders for levels, tiles, gani animations, images and sounds, from the 2.22 data folder that works for every client.
- [ ] File transfer and caching. The downloaded-file rules the real clients follow.

## 5. Behaviour over time

- [ ] Version chain. Each profile plus a per-step handler-change table, so adding a version means adding a row and a small delta, not new logic.
- [ ] Differential testing. Run the same scripted session against the real Wine client and Seihai in the lab, and diff the wire frames. Finds every remaining gap automatically.
- [ ] Public-server checks. Read-only first, and only with approval each time.
- [ ] Tooling. A capture viewer and a layout explorer fed by the .tsv tables.

## Completed (prior phases)

### Phase 1 — Protocol & Minimal Client
- [x] Packet framing notes + classic login line
- [x] Packet ids locked against lab captures
- [x] Listserver login against a real server
- [x] Gserver connection + version handshake
- [x] Level name / basic tile data reception
- [x] Player position codecs (classic ×2, precise ×16)
- [x] Simple chat

### Phase 2 — Assets & Rendering
- [x] Text level loader (GLEVNW01)
- [x] Binary loaders (.graal, .zelda)
- [x] Tile-type table (text or arrays.dat)
- [x] ASCII / PPM board view
- [x] Sprite sheet / tileset loading
- [x] Tile renderer (software, minifb window)
- [x] Player sprite compositing (4 directions)
- [x] Camera follow

### Phase 3 — Seihaiscript GS1 Subset
- [x] Bytecode parser + walker
- [x] Minimal interpreter (arithmetic, comparison, jump)
- [x] Command stubs (`message`, `say`, `setimg`, `move`, …)

### Phase 4 — Polish & Compatibility
- [x] Gani parser + stepper
