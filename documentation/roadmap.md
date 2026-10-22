## Phase 1 — Protocol & Minimal Client
- [x] Packet framing notes + classic login line
- [x] Packet ids locked against lab captures
- [x] Listserver login against a real server
- [x] Gserver connection + version handshake
- [x] Level name / basic tile data reception
- [x] Player position codecs (classic ×2, precise ×16)
- [x] Simple chat

## Phase 2 — Assets & Rendering
- [x] Text level loader (GLEVNW01)
- [x] Binary loaders (.graal, .zelda)
- [x] Tile-type table (text or arrays.dat)
- [x] ASCII / PPM board view
- [x] Sprite sheet / tileset loading
- [x] Tile renderer (software, minifb window)
- [x] Player sprite compositing (4 directions)
- [x] Camera follow

## Phase 3 — Seihaiscript GS1 Subset
- [x] Bytecode parser + walker
- [x] Minimal interpreter (arithmetic, comparison, jump)
- [x] Command stubs (`message`, `say`, `setimg`, `move`, …)
- [ ] Core commands with session side effects
- [ ] Event dispatch (`onCreated`, `onPlayerTouchsme`, …)
- [ ] Client-side script execution for NPCs and weapons

## Phase 4 — Polish & Compatibility
- [x] Gani parser + stepper
- [ ] Sound
- [ ] Smooth movement flags (`gr.x` / `gr.y`)
- [x] GUI controls (basic)
- [ ] Broader packet coverage

## Phase 5 — Tooling
- [x] Level editor CLI (ASCII preview + re-export)
- [ ] Gani / animation editor
- [ ] Terrain / dungeon / world generators
- [ ] Format converters

## Phase 6 — Ecosystem
- [x] Documentation
- [ ] Packaging (Windows / Linux / macOS)
- [ ] Optional web / WASM build
- [ ] Community contribution guidelines