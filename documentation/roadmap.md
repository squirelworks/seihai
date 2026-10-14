## Phase 1 — Protocol & Minimal Client
- [x] Packet framing notes + classic login line
- [x] Provisional packet types (login, level request, player update, board modify)
- [ ] Listserver login against a real server
- [ ] Gserver connection + version handshake
- [ ] Level name / basic tile data reception
- [x] Player position codecs (classic ×2, precise ×16)
- [ ] Simple chat

## Phase 2 — Assets & Rendering
- [x] Text level loader (GLEVNW01)
- [x] Tile-type table (text or arrays.dat)
- [x] ASCII / PPM board view
- [ ] Sprite sheet / tileset loading
- [ ] Basic tile renderer (SDL2)
- [ ] 8-direction player sprite compositing
- [x] Camera follow (ASCII)

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
- [ ] GUI controls (basic)
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