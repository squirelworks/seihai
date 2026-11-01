# 1.39's built-in animations, walk sprite and sword (2026-10-31)

Seihai's 1.39 profile drew the player as a still figure: walking, the sword,
pushing, pulling, carrying, swimming and the hurt pose were all missing, and
other players saw only the direction. 1.39 has no GANI files (`ui.ganis` is
false), and Seihai only animated from GANI files, so for 1.39 it found none.
This note says what changed and what it is based on.

## Built-in poses

1.39's player animations are compiled into the executable. NewMain 1.41.1,
the first GANI client, plays the same poses from files, and those stock
files are the 1.39 tables written out: Graal_analysis
`client-internals/player-mannerisms-live.md` ("Other builds") compares 1.39,
1.41, 2.05 and 2.22 frame by frame.

- `AssetResolver::pose_gani` finds a player pose. A GANI build reads it as
  any GANI (cache first). A build without GANIs takes only the names in
  `pose::BUILTIN_ANIMS` (`idle`, `walk`, `swim`, `sword`, `spin`, `push`,
  `grab`, `pull`, `carrystill`, `carry`, `hurt`, `sit`, `sleep`, `dead`), and
  only from the user's asset folder or Seihai's built-in set, never from the
  per-server cache: 1.39 downloads no GANIs. There is no `lift` (1.39 goes
  straight to the carry pose) and no `hidden`/`hiddenstill` (1.39 keeps the
  carry poses while hidden).
- `pose::fit_builtin_139` fits the stock file to 1.39: push frames last
  about 2 ticks, not 4 (video); the hurt pose lasts about 10 ticks, not 31,
  and the blink follows it (`Blink::AfterHurtPose`, already in the
  profile); the footstep cue is `steps.wav` (1.39 has no `steps2.wav`), and a
  cue for a sound 1.39 does not name (`lift.wav` on the hurt pose) is
  dropped.
- Character NPCs whose animation is one of those names are drawn with it too.
- With a folder that has no GANI files (a bare 1.39 install), Seihai's own
  default set stands in, as it does for the 2.x builds.

Seihai ships no Graal GANI file: the poses come from the user's own folder
(the 2.22 folder the launcher uses by default has `levels/ganis/`).

## Walk sprite

A real 1.39 sends its walk frame in the SPRITE prop (17, `dir + 4 * S`) while
walking. The lab capture of 1.39 (`item-drops/b4_139.txt` in the live-trace
project files) walking down sends 18 (S 4) and 10 (S 2), then 22, 14 and 6,
every 100 ms, and the direction alone once it stops. Seihai's 1.39 sent the
direction only, so others saw it glide. `OfflineSession::walk_sprite` (set
for builds without GANIs) now sends S from the walk frame through the 2.05
step table (`pose::WALK_S`), which those values fit. GANI builds send the
name `walk` instead and are unchanged.

## Sword and hurt mode

The 1.39 profile has `COMBAT_139`: the 2.05 sword box (0 to 4 tiles ahead,
under 2 sideways) with one swing step per 50 ms tick. The capture shows
S 6, 8 and 10 at 100 ms intervals and then the idle sprite, and the video
gives a 5-tick swing that cuts the bush in front and a 40-tick hurt
lock-out. Sword power, AP immunity and the hurt sprite are still the 2.05
template. Knockback is five tiles. 1.33 to 1.38 still have no combat row.

Hurt mode also keeps one blast from hurting more than once. Without it, a
1.39 on hastur lost three half hearts to its own bomb (run of 2026-10-31
15:00); the 40-tick hurt mode now takes the later pieces off.

## Checked

- `crates/seihai-mae/tests/builtin_139.rs`: the poses come from the user's
  folder and never the cache, are fitted to 1.39, the default set stands in;
  the walk sprite sequence; the swing S 6 to 10 one a tick.
- Lab (loopback, GServer-v2 beta4 "Lab Classic" on 14803):
  `cargo run -p seihai-mae --example walk139_probe -- <prefix>` logs two 1.39
  players in. A walks and swings; B saw A's sprites `2, 18, 10, 18, 2, 26, 30,
  34, 38, 42, 2`.
- The game under Xvfb as 1.39 offline (`start1.graal`, a 1.39 install with
  the NewMain stock GANIs in `levels/ganis/`): the idle, two walk frames and
  the sword swing are drawn.

## Still open

- Emotion icons on Ctrl+letter (1.38+) are not drawn.
- The 1.39 profile window (F7 Profile, `PLI_PROFILEGET`/`SET`, `PLO_PROFILE`
  is parsed but not shown), and the 1.39 player list options window.
- A live 1.39 death on a lab server (the profile now has `DEATH_139`: timer 30, dead bit after 27, respawn hearts to the maximum).
- Whether 1.39's spin attack exists; `spin` is offered when the status has
  the spin bit, as in 2.05.
- What a 1.39 does with a character NPC's animation prop is not traced; the
  built-in poses are used for the names above.
