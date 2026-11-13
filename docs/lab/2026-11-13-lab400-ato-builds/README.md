# Lab #400: Ato 3.x, 4.x and 5.x builds against the lab server stack (2026-11-13)

This lab ran in Mirae's Linux RE Lab VM, headless, inside a private network namespace with loopback only.

**Builds:**
- `seihai-ato`: seihai main `b31c2d5`, release build.
- Server stack:
  - seihai-listserver `ae213fc`, with accounts open.
  - Three seihai-gserver-v2 `101a2cce` gservers with packet logging, run on copies of the lab worlds:
    - **Lab Modern** on 14805, with the #465 link to `lab465_b.nw`.
    - **Enigma (2005)** on 14806.
    - **Login** on 14815: a copy of Modern with `serverside = true` and a level NPC that runs `serverwarp Lab Modern;` when the player enters.
  - In these copies only, `localhost = true` was changed to `false`. With `localhost = true` the gservers never register with the listserver, so steps 2 and 4b could not run.

Every login used a made-up `lab400*` account, with a random password passed through `--password-env`. Step 1's runs happened before that switch, so their gserver lines read "Localhost mode: accepting …". All later logins went through the listserver.

**How the GUI steps ran:** the GUI steps (2, 3, 4) ran Ato's window on Xvfb, with clicks and held keys from xdotool. The times are in `gui/keys.txt`. Ato logs nothing about other players, so step 3's evidence is screenshots from two displays.

Step 5 (listserver.graal.in) is out of scope here; another thread owns it.

## Results

| Build | 1 Headless login | 2 Server list, Connect, level | 3 Walk, other player | 4 Level link | 4b Login warp (178) |
|---|---|---|---|---|---|
| 3.04 | pass | pass | pass (with 3.153) | pass | not applicable (4.x/5.x step) |
| 3.153 | pass | pass | pass (with 3.04) | pass | not applicable |
| 4.04r2 | pass | pass ([#465](https://github.com/squirelworks/seihai/issues/465)) | pass (with 5.00r7) | pass, both ways (#465) | not in the issue's list |
| 4.20r8 | pass | pass | pass (with 5.12), see finding 1 | pass, both ways (#465) | pass |
| 5.00r7 | pass | not in the issue's list | pass (with 4.04r2), see finding 1 | pass, both ways | pass |
| 5.12 | pass | pass | pass (with 4.20r8), see finding 1 | pass, both ways | pass |

5.12's Graal3D login is covered separately by [#458](../2026-11-13-lab458-graal3d-run3/README.md), which is on branch `claude/lab-458-graal3d`.

### Step 1: headless login (`--no-window --first-level`, `step1/`)

| Build | Port | Ato | Gserver |
|---|---|---|---|
| 3.04 | 14806 | `logged in at 127.0.0.1:14806 as lab400s1 (3.04)` / `sent the post-login frame (5 messages)` / `level onlinestartlocal.graal: from the server's board, start (23.5, 30.5)` | `New login: lab400s1 [New Client (2.22+)] [G3D03014 (3.041)]` |
| 3.153 | 14806 | `logged in … (3.153)` / `sent the post-login frame (6 messages)` / `level onlinestartlocal.graal …` | `[G3D28015 (3.153)]` |
| 4.04r2 | 14805 | `logged in … (4.04r2)` / `sent the post-login frames (4)` / `level onlinestartlocal.nw … start (30, 30.5)` | `[G3D17026 (4.042)]` |
| 4.20r8 | 14805 | `logged in … (4.20r8)` / `sent the post-login frames (4)` / `level onlinestartlocal.nw …` | `[G3D20126 (4.208)]` |
| 5.00r7 | 14805 | `logged in … (5.00r7)` / `level onlinestartlocal.nw …` | `[G3D22067 (5.007)]` |
| 5.12 | 14805 | `logged in … (5.12)` / `level onlinestartlocal.nw …` | `[G3D14097 (5.12)]` |

- Every build's token was accepted.
- No run logged "connection closed" before its level.

### Step 2: server list (`gui/g-l*-stdout.txt`, `gui/l*.png`)

- **Listing:** Start lists the three lab servers.
  - 3.x: `server list from 127.0.0.1:14922: 3 servers. Welcome to Seihai Lab, lab400a.`
  - 4.20r8 and 5.12 (the 2.05 list login): `… Welcome to Seihai Lab, player.`
- **Connect:**
  - 3.04 and 3.153 connected to Enigma: `logged in at 127.0.0.1:14806 as lab400a (3.04)`, then the level, drawn in `l304-2.png`.
  - 4.20r8 and 5.12 connected to Lab Modern: `logged in at 127.0.0.1:14805 as lab400d (4.20r8)` and `… as lab400e (5.12)`, drawn in `l420-2.png` and `l512-2.png`.

### Step 3: walk with a second client (`gui/s3-*.png`)

Each pair shared one level on two displays. One client walked, then the other.
- **3.153 and 3.04 on Enigma:** 3.153 drew `*lab400c` after 3.04 moved right. 3.04 drew `*lab400b` after 3.153 moved left.
- **4.20r8 and 5.12 on Modern:** each drew the other at its new spot. 5.12 labels 4.20r8 `*lab400d`, but 4.20r8 labels 5.12 `default`.
- **5.00r7 and 4.04r2:** the same result. 5.00r7 labels 4.04r2 `*lab400p404`, and 4.04r2 labels 5.00r7 `default`.

### Step 4: level links

**Enigma** (3.04 and 3.153), walking south onto the start level's link:
```
link to egk-10.graal at 54.5 19
level egk-10.graal: from the server's board, start (54.5, 19), 4 NPCs (4 from the server, 0 in the file)
```
The gserver (`enigma-networkdump.txt`) logged `In Packet from lab400a [ID 3]: [0] PLI_LEVELWARP (15 bytes) … egk-10.graal`, answered with `PLO_LEVELNAME egk-10.graal`.

**Modern** (5.12 and 5.00r7), walking north into the #465 link and back:
```
link to lab465_b.nw at 30 32
level lab465_b.nw: from the server's board, start (30, 32), ...
link to onlinestartlocal.nw at 30 30.5
level onlinestartlocal.nw: from the server's board, start (30, 30.5), ...
```
The gserver logged:
- `In Packet from lab400k512 [ID 3]: [0] PLI_LEVELWARP` (lab465_b.nw), then `[30] PLI_LEVELWARPMOD` (onlinestartlocal.nw)
- the same pair for `lab400k500r7`

4.04r2 and 4.20r8 passed both ways in #465.

### Step 4b: Login warp (`step4b/`)

Each client was pointed at the Login gserver: `--online 127.0.0.1:14815 --no-window`, run for 25 s.

**4.20r8:**
```
logged in at 127.0.0.1:14815 as lab400w420r8 (4.20r8)
level onlinestartlocal.nw: from the server's board, start (30, 30.5), 1 NPCs (1 from the server, 0 in the file)
server warp to "Lab Modern": 127.0.0.1:14805
logged in at 127.0.0.1:14805 as lab400w420r8 (4.20r8)
sent the post-login frames (4)
level onlinestartlocal.nw: from the server's board, start (30, 30.5), ...
```
5.00r7 and 5.12 logged the same lines.

The Login gserver sent the warp three times (`login-networkdump.txt`), once per client:
`< Out Packet to lab400w420r8 [ID 3]: [178] PLO_SERVERWARP (29 bytes)  0,Lab Modern,127.0.0.1,14805`

Modern then logged a second `New login` for each client: `[G3D20126 (4.208)]`, `[G3D22067 (5.007)]` and `[G3D14097 (5.12)]`.

So a lab server can now send 178, and 4b passes against a real gserver, not just the loopback test.

**Kicks and unknown packets:** there were none in any gserver log.

## Finding 1: a 5.x player shows as "default" to others

**What happens:** 4.04r2 and 4.20r8 label a 5.00r7 or 5.12 player `default`. The 5.x clients label 4.x players correctly (`*lab400d`).

**Gserver evidence:**
- The gserver itself holds `PlayerProp::NICKNAME: value: default` for the 5.x players, and that is what it hands to the other clients.
- The 4.x players carry their real nickname (`*lab400d`).
- I found no `NICKNAME` in any player-props packet from the clients' side.

**Guess at the cause:** the nickname reaches the gserver in 4.x's post-login frames, and Ato's 5.x login sends no post-login frames. 4.x logs `sent the post-login frames (4)`; 5.x logs no such line. So a 5.x player keeps the account's default nickname. The 5.x viewers' own labels for 4.x players are right, so the drawing side is fine.

**What to check:** whether the real 5.00r7/5.12 clients send `NICKNAME` after login.

This run used main `b31c2d5`. Main has since gained #503 ("nicknames" in the 3D world); I have not checked whether it changes this. I have not changed any code.

## Files

- `step1/`: headless login stdout, one file per build
- `step4b/`: Login-warp stdout, one file per build
- `gui/`:
  - `g-*-stdout.txt`: each GUI run's stdout (`l*` = server list, `w*`/`p*` = second clients, `k*` = link walks)
  - screenshots
  - `keys.txt`: when each key was held
- `gserver/`:
  - console logs and packet dumps of `modern`, `enigma` and `login`. The dumps start at the switch to `localhost = false`, because the gserver resets its dump on each start; step 1's lines are in the consoles. Login passwords are masked by the gserver.
  - `listserver-console.txt`
